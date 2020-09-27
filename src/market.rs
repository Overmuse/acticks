use actix_web::web::Data;
use chrono::{Date, Utc};
use log;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::sync::Mutex;
use tokio::time::{DelayQueue, Duration, Instant};

#[cfg(kafka)]
use rdkafka::consumer::stream_consumer::StreamConsumer;

use reqwest::Client;

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone)]
#[repr(u8)]
pub enum Tape {
    A = 1,
    B = 2,
    C = 3,
}

fn default_conditions() -> Vec<u8> {
    Vec::new()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Trade {
    #[serde(rename = "i")]
    pub trade_id: String,
    #[serde(rename = "x")]
    pub exchange_id: u8,
    #[serde(rename = "p")]
    pub price: f64,
    #[serde(rename = "s")]
    pub size: u32,
    #[serde(rename = "c", default = "default_conditions")]
    pub conditions: Vec<u8>,
    #[serde(rename = "t")]
    pub timestamp: u64,
    #[serde(rename = "z")]
    pub tape: Tape,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TickerTrade(pub String, pub Trade);

pub trait Market {
    fn new(symbols: String, start_date: Date<Utc>, end_date: Date<Utc>) -> Self;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PolygonResponse {
    results_count: i32,
    db_latency: i32,
    success: bool,
    ticker: String,
    results: Vec<Trade>,
}

#[derive(Debug)]
pub struct PolygonMarket {
    pub symbols: Vec<String>,
    data: Vec<TickerTrade>,
    pub queue: DelayQueue<TickerTrade>,
}

impl PolygonMarket {
    pub fn new(symbols: Vec<String>) -> Self {
        Self {
            symbols,
            data: vec![],
            queue: DelayQueue::new(),
        }
    }

    pub fn create(symbols: Vec<String>) -> Data<Mutex<Self>> {
        let me = Data::new(Mutex::new(Self::new(symbols.clone())));
        PolygonMarket::initialize(me.clone(), symbols);
        me
    }

    pub fn initialize(me: Data<Mutex<Self>>, symbols: Vec<String>) {
        actix_web::rt::spawn(async move {
            for symbol in symbols {
                log::debug!("Downloading data for {}", &symbol);
                let mut data = me.lock().unwrap().download_data(&symbol).await.unwrap();
                me.lock().unwrap().data.append(&mut data);
            }
            me.lock()
                .unwrap()
                .data
                .sort_by(|t1, t2| t2.1.timestamp.partial_cmp(&t1.1.timestamp).unwrap());

            log::debug!("Scheduling trades");
            me.lock().unwrap().schedule_trades(1000);
        });
    }

    pub fn schedule_trades(&mut self, rate: u64) {
        let first_message: TickerTrade = self.data.pop().unwrap();
        let actual_now = Instant::now();
        let simulated_now = first_message.1.timestamp;
        self.queue
            .insert_at(first_message, actual_now + Duration::from_nanos(0));
        while let Some(message) = self.data.pop() {
            log::trace!("Scheduling message: {:?}", &message);
            let event_time =
                actual_now + Duration::from_nanos((message.1.timestamp - simulated_now) / rate);
            self.queue.insert_at(message, event_time);
        }
    }

    async fn download_data(&self, symbol: &str) -> Result<Vec<TickerTrade>, i32> {
        let client = Client::new();
        let url = format!(
            "https://api.polygon.io/v2/ticks/stocks/trades/{}/2020-09-18?apiKey={}",
            symbol,
            std::env::var("POLYGON_KEY").unwrap()
        );
        log::trace!("Making request: {}", &url);
        let req = client.get(&url).send().await.unwrap();
        let res = req.text().await.unwrap();
        let res: PolygonResponse = serde_json::from_str(&res).unwrap();
        Ok(res
            .results
            .iter()
            .cloned()
            .map(|t| TickerTrade(symbol.to_string(), t))
            .collect())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test() {
        let trades = vec![
            Trade {
                symbol: "AAPL".into(),
                trade_id: "".into(),
                exchange_id: 1,
                price: 100.0,
                size: 1,
                conditions: vec![],
                timestamp: 1599991200000,
                tape: Tape::A,
            },
            Trade {
                symbol: "AAPL".into(),
                trade_id: "".into(),
                exchange_id: 1,
                price: 200.0,
                size: 1,
                conditions: vec![],
                timestamp: 1599991200001,
                tape: Tape::A,
            },
        ];
        let mut md = MarketData::new(trades);
        md.schedule_trades(1);
        let trade = md.queue.next().await.unwrap().unwrap().into_inner();
        assert_eq!(trade.price, 100.0);

        let trade2 = md.queue.next().await.unwrap().unwrap().into_inner();
        assert_eq!(trade2.price, 200.0);

        assert!(md.queue.next().await.is_none());
    }
}
