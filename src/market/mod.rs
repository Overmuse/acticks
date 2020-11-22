use crate::errors::Result;
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use serde_repr::*;

//#[cfg(feature = "polygon")]
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

#[derive(Serialize, Deserialize, Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct Trade {
    #[serde(rename = "sym")]
    pub symbol: String,
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

#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe(pub Recipient<Trade>);

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub struct Initialize(pub Vec<String>);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Start(pub u64);

pub trait Market: Actor + Handler<Subscribe> + Handler<Initialize> + Handler<Start> {}
