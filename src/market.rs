use chrono::{Date, Utc};
use serde::{Deserialize, Serialize};
use serde_repr::*;
use tokio::stream::StreamExt;
use tokio::time::{DelayQueue, Duration};

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
    timestamp: u64,
    #[serde(rename = "z")]
    tape: Tape,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TickerTrade(String, Trade);

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
    symbols: Vec<String>,
    data: Vec<TickerTrade>,
    queue: DelayQueue<TickerTrade>,
}

impl PolygonMarket {
    pub fn new(symbols: Vec<String>) -> Self {
        Self {
            symbols,
            data: vec![],
            queue: DelayQueue::new(),
        }
    }

    pub async fn initialize(&mut self) {
        for symbol in &self.symbols {
            let mut data = self.download_data(&symbol).await.unwrap();
            self.data.append(&mut data);
        }
        self.data
            .sort_by(|t1, t2| (t2.1.timestamp.partial_cmp(&t1.1.timestamp)).unwrap());

        self.schedule_trades(1).await;
    }

    pub async fn schedule_trades(&mut self, rate: u64) {
        let mut prev_message: Option<TickerTrade> = None;
        for message in &self.data {
            println!("{:?}", &prev_message);
            println!("{:?}", &message);
            if let Some(prev) = prev_message {
                let delay = message.1.timestamp - prev.1.timestamp;
                self.queue
                    .insert(message.clone(), Duration::from_nanos(delay));
            } else {
                self.queue.insert(message.clone(), Duration::from_nanos(0));
            }
            prev_message = Some(message.clone());
        }
    }

    async fn download_data(&self, symbol: &str) -> Result<Vec<TickerTrade>, i32> {
        let client = Client::new();
        let url = format!(
            "https://api.polygon.io/v2/ticks/stocks/trades/{}/2020-09-18?apiKey={}",
            symbol,
            std::env::var("POLYGON_KEY").unwrap()
        );
        println!("{:?}", url);
        let req = client.get(&url).send().await.unwrap().text().await.unwrap();
        let res: PolygonResponse = serde_json::from_str(&req).unwrap();
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
