use crate::account::actors::AccountManager;
use crate::asset::types::Asset;
use crate::errors::{Error, Result};
use crate::market::Trade;
use crate::order::{
    actors::OrderManager,
    types::{Order, OrderType, Side},
};
use crate::position::actors::PositionManager;
use actix::prelude::*;
use chrono::{DateTime, Utc};
use log::debug;
use log::warn;
use std::collections::HashMap;

#[derive(Clone, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct TradeFill {
    pub time: DateTime<Utc>,
    pub qty: i32,
    pub price: f64,
    pub order: Order,
}

pub enum MarketStatus {
    PreOpen,
    Open,
    PostClose,
    Maintenance,
    Closed,
}

pub struct Exchange {
    pub stored_orders: Vec<Order>,
    pub market_status: MarketStatus,
    pub assets: Vec<Asset>,
    pub prices: HashMap<String, f64>,
}

impl Actor for Exchange {
    type Context = Context<Self>;
}

impl actix::Supervised for Exchange {}

impl SystemService for Exchange {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {
        debug!("Exchange service started");
    }
}

#[derive(Message)]
#[rtype(result = "Result<Option<TradeFill>>")]
pub struct TransmitOrder(pub Order);

impl Handler<TransmitOrder> for Exchange {
    type Result = Result<Option<TradeFill>>;

    fn handle(&mut self, msg: TransmitOrder, _ctx: &mut Context<Self>) -> Self::Result {
        self.transmit_order(msg.0)
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SetAssets {
    pub assets: Vec<Asset>,
}

impl Handler<SetAssets> for Exchange {
    type Result = ();

    fn handle(&mut self, msg: SetAssets, _ctx: &mut Context<Self>) -> Self::Result {
        self.assets = msg.assets;
        self.prices = HashMap::new();
    }
}

impl Handler<Trade> for Exchange {
    type Result = ();

    fn handle(&mut self, msg: Trade, _ctx: &mut Context<Self>) {
        self.prices
            .entry(msg.symbol.clone())
            .and_modify(|x| *x = msg.price)
            .or_insert(msg.price);
    }
}

impl Exchange {
    pub fn new(assets: Vec<Asset>) -> Self {
        let mut prices = HashMap::new();
        assets.iter().for_each(|a| {
            prices.insert(a.symbol.clone(), 100.0);
        });
        Self {
            stored_orders: vec![],
            market_status: MarketStatus::Open,
            assets,
            prices,
        }
    }

    pub fn transmit_order(&mut self, o: Order) -> Result<Option<TradeFill>> {
        match (&self.market_status, o.extended_hours) {
            (MarketStatus::Open, _)
            | (MarketStatus::PreOpen, true)
            | (MarketStatus::PostClose, true) => match o.order_type {
                OrderType::Market => {
                    let price = *self.get_price(&o.symbol)?;
                    Ok(Some(self.execute(o, price)))
                }
                _ => self.execute_or_store(o),
            },
            (MarketStatus::Maintenance, _) => todo!(),
            _ => {
                self.store(o);
                Ok(None)
            }
        }
    }

    pub fn is_open(&self) -> bool {
        true
    }

    pub fn market_status(&self) -> MarketStatus {
        todo!()
    }

    pub fn execute(&self, order: Order, price: f64) -> TradeFill {
        let qty = match order.side {
            Side::Buy => order.qty as i32,
            Side::Sell => -(order.qty as i32),
        };
        TradeFill {
            time: Utc::now(),
            qty,
            price,
            order,
        }
    }

    pub fn execute_or_store(&mut self, o: Order) -> Result<Option<TradeFill>> {
        warn!("{:?}", self.get_price(&o.symbol));
        let price = *self.get_price(&o.symbol)?;
        if is_marketable(&o, price) {
            Ok(Some(self.execute(o, price)))
        } else {
            self.store(o);
            Ok(None)
        }
    }

    pub fn store(&mut self, o: Order) {
        self.stored_orders.push(o);
    }

    pub fn get_price(&self, symbol: &str) -> Result<&f64> {
        self.prices
            .get(symbol)
            .ok_or_else(|| Error::UninitializedPrice)
    }

    pub fn update_price(&mut self, symbol: &str, price: f64) -> Vec<TradeFill> {
        self.prices
            .entry(symbol.to_string())
            .and_modify(|e| *e = price)
            .or_insert(price);
        let marketable_orders: Vec<Order> = self
            .stored_orders
            .drain_filter(|o| o.symbol == symbol && is_marketable(o, price))
            .collect();

        marketable_orders
            .iter()
            .map(|o| self.execute(o.clone(), price))
            .collect()
    }
}

impl Default for Exchange {
    fn default() -> Self {
        let assets: Vec<Asset> = vec![];
        Self::new(assets)
    }
}

pub async fn update_from_fill(tf: &TradeFill) -> Result<()> {
    OrderManager::from_registry().send(tf.clone()).await??;
    // Account needs to be update before position as it relies on the previous position
    AccountManager::from_registry().send(tf.clone()).await??;
    PositionManager::from_registry().send(tf.clone()).await??;
    Ok(())
}
fn is_marketable(o: &Order, price: f64) -> bool {
    match (&o.order_type, &o.side) {
        (OrderType::Market, _) => true,
        (OrderType::Limit { limit_price }, Side::Buy) => *limit_price >= price,
        (OrderType::Limit { limit_price }, Side::Sell) => *limit_price <= price,
        (OrderType::Stop { stop_price }, Side::Buy) => *stop_price <= price,
        (OrderType::Stop { stop_price }, Side::Sell) => *stop_price >= price,
        (
            OrderType::StopLimit {
                limit_price,
                stop_price,
            },
            Side::Buy,
        ) => *limit_price >= price && price >= *stop_price,
        (
            OrderType::StopLimit {
                limit_price,
                stop_price,
            },
            Side::Sell,
        ) => *limit_price <= price && price <= *stop_price,
    }
}
