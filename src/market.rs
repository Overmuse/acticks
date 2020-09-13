use chrono::{Date, Utc};
use serde::{Deserialize, Serialize};
use serde_repr::*;
use tokio::stream::StreamExt;
use tokio::time::{DelayQueue, Duration};

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
    #[serde(rename = "sym")]
    symbol: String,
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

struct MarketData {
    trades: Vec<Trade>,
    queue: DelayQueue<Trade>,
}

fn get_trades(symbols: Vec<&str>, start_date: Date<Utc>, end_date: Date<Utc>) -> Vec<Trade> {
    todo!()
}

impl MarketData {
    fn new(mut trades: Vec<Trade>) -> Self {
        trades.sort_by(|t1, t2| t2.timestamp.partial_cmp(&t1.timestamp).unwrap());
        Self {
            trades,
            queue: DelayQueue::new(),
        }
    }

    fn initialize_trades(symbols: Vec<&str>, start_date: Date<Utc>, end_date: Date<Utc>) -> Self {
        let trades = get_trades(symbols, start_date, end_date);
        Self::new(trades)
    }

    fn schedule_trades(&mut self, rate: u64) {
        let mut prev_message = self.trades.pop().unwrap();
        self.queue
            .insert(prev_message.clone(), Duration::from_secs(0));
        for message in &self.trades {
            let delay = message.timestamp - prev_message.timestamp;
            self.queue
                .insert(message.clone(), Duration::from_millis(delay));
            prev_message = message.clone();
        }
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
