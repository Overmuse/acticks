use super::*;
use crate::errors::{Error, Result};
use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
use serde_json;
use std::cmp::Reverse;
use tokio::time::{DelayQueue, Duration, Instant};
use websocket::{ClientBuilder, r#async::client::Client};

#[derive(Default)]
pub struct PolygonMarket {
    client: Option<Client>,
    subscribers: Vec<Recipient<TickerTrade>>,
}

impl actix::Supervised for PolygonMarket {}

impl SystemService for PolygonMarket {}

impl PolygonMarket {
    pub fn new() -> Self {
        PolygonMarket {
            client: None,
            subscribers: vec![],
        }
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
        let simulated_now = first_message.1.timestamp;
        stream.insert_at(first_message, actual_now + Duration::from_nanos(0));
        while let Some(message) = self.trades.pop() {
            log::trace!("Scheduling message: {:?}", &message);
            let event_time = actual_now
                + Duration::from_nanos((message.1.timestamp - simulated_now) as u64 / msg.0);
            stream.insert_at(message, event_time);
        }
        let stream = actix::fut::wrap_stream::<_, Self>(stream);
        let fut = stream
            .map(|msg, act, _ctx| {
                let trade = msg.expect("probably will never error").into_inner();
                info!("{:?}", &trade);
                for subscr in &act.subscribers {
                    let res = subscr.do_send(trade.clone());
                    if let Err(e) = res {
                        warn!("Error received when sending TickerTrade: {:?}", e)
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
            let trades: Vec<TickerTrade> =
                futures::future::join_all(msg.0.into_iter().map(|symbol| async move {
                    PolygonMarket::download_data(&symbol).await.unwrap()
                }))
                .await
                .into_iter()
                .flatten()
                .collect();
            Ok::<Vec<TickerTrade>, Error>(trades)
        }
        .into_actor(self)
        .map(|trades, act, _ctx| {
            let mut trades = trades?;
            trades.sort_unstable_by_key(|t| Reverse(t.1.timestamp));
            act.trades = trades;
            Ok(())
        });
        Box::pin(fut)
    }
}

impl Market for PolygonMarket {}
