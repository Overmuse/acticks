use super::*;
use crate::errors::{Error, Result};
use tracing::{debug, info, trace, warn};
use reqwest::Client;
use serde::{Deserialize};
use serde_json;
use std::cmp::Reverse;
use tokio::time::{DelayQueue, Duration, Instant};

#[derive(Deserialize, Clone)]
struct NonTickerTrade {
    #[serde(rename = "i")]
    trade_id: String,
    #[serde(rename = "x")]
    exchange_id: u8,
    #[serde(rename = "p")]
    price: f64,
    #[serde(rename = "s")]
    size: u32,
    #[serde(rename = "c", default = "default_conditions")]
    conditions: Vec<u8>,
    #[serde(rename = "t")]
    timestamp: i64,
    #[serde(rename = "z")]
    tape: Tape,
}

#[derive(Deserialize)]
struct PolygonResponse {
    results_count: i32,
    db_latency: i32,
    success: bool,
    ticker: String,
    results: Vec<NonTickerTrade>,
}

#[derive(Default)]
pub struct PolygonMarket {
    subscribers: Vec<Recipient<Trade>>,
    trades: Vec<Trade>,
}

impl actix::Supervised for PolygonMarket {}

impl SystemService for PolygonMarket {}

impl PolygonMarket {
    pub fn new() -> Self {
        PolygonMarket {
            subscribers: vec![],
            trades: vec![],
        }
    }

    async fn download_data(symbol: &str) -> Result<Vec<Trade>> {
        let client = Client::new();
        let url = format!(
            "https://api.polygon.io/v2/ticks/stocks/trades/{}/2020-09-18?apiKey={}",
            symbol,
            std::env::var("POLYGON_KEY")?
        );
        trace!("Making request: {}", &url);
        let req = client.get(&url).send().await?;
        let res = req.text().await?;
        let res: PolygonResponse = serde_json::from_str(&res)?;
        Ok(res
            .results
            .iter()
            .cloned()
            .map(|t| Trade { 
                symbol: symbol.to_string(), 
                trade_id: t.trade_id,
                exchange_id: t.exchange_id,
                price: t.price,
                size: t.size,
                conditions: t.conditions,
                timestamp: t.timestamp,
                tape: t.tape,     
            } )
            .collect())
    }
}

impl Actor for PolygonMarket {
    type Context = Context<Self>;
}

impl Handler<Subscribe> for PolygonMarket {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _ctx: &mut Context<Self>) {
        self.subscribers.push(msg.0);
    }
}

impl Handler<Start> for PolygonMarket {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: Start, _ctx: &mut Context<Self>) -> Self::Result {
        debug!("Scheduling trades");
        let mut stream = DelayQueue::new();
        if self.trades.is_empty() {
            return Box::pin(actix::fut::ready(()));
        }
        let first_message = self
            .trades
            .pop()
            .expect("Guaranteed to have at least one element");
        let actual_now = Instant::now();
        let simulated_now = first_message.timestamp;
        stream.insert_at(first_message, actual_now + Duration::from_nanos(0));
        while let Some(message) = self.trades.pop() {
            log::trace!("Scheduling message: {:?}", &message);
            let event_time = actual_now
                + Duration::from_nanos((message.timestamp - simulated_now) as u64 / msg.0);
            stream.insert_at(message, event_time);
        }
        let stream = actix::fut::wrap_stream::<_, Self>(stream);
        let fut = stream
            .map(|msg, act, _ctx| {
                let trade = msg.expect("probably will never error").into_inner();
                //info!("{:?}", &trade);
                for subscr in &act.subscribers {
                    let res = subscr.do_send(trade.clone());
                    if let Err(e) = res {
                        warn!("Error received when sending Trade: {:?}", e)
                    }
                }
            })
            .finish();
        Box::pin(fut)
    }
}
impl Handler<Initialize> for PolygonMarket {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: Initialize, _ctx: &mut Context<Self>) -> Self::Result {
        let fut = async {
            info!("Downloading data");
            let trades: Vec<Trade> =
                futures::future::join_all(msg.0.into_iter().map(|symbol| async move {
                    PolygonMarket::download_data(&symbol).await.unwrap()
                }))
                .await
                .into_iter()
                .flatten()
                .collect();
            Ok::<Vec<Trade>, Error>(trades)
        }
        .into_actor(self)
        .map(|trades, act, _ctx| {
            let mut trades = trades?;
            trades.sort_unstable_by_key(|t| Reverse(t.timestamp));
            act.trades = trades;
            Ok(())
        });
        Box::pin(fut)
    }
}

impl Market for PolygonMarket {}
