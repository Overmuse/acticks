use actix::prelude::*;
use serde::{Deserialize, Serialize};
use serde_repr::*;

pub mod polygon;

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
    pub timestamp: i64,
    #[serde(rename = "z")]
    pub tape: Tape,
}

#[derive(Serialize, Deserialize, Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct TickerTrade(pub String, pub Trade);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe(pub Recipient<TickerTrade>);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Initialize(pub Vec<String>);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Start(pub u64);

pub trait Market: Actor + Handler<Subscribe> + Handler<Initialize> + Handler<Start> {}

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
