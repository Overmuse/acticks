use crate::asset::types::Asset;
use crate::order::{Order, OrderType, Side};
use actix::prelude::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
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

    pub fn transmit_order(&mut self, o: Order) -> Option<TradeFill> {
        match (&self.market_status, o.extended_hours) {
            (MarketStatus::Open, _)
            | (MarketStatus::PreOpen, true)
            | (MarketStatus::PostClose, true) => match o.order_type {
                OrderType::Market => {
                    let price = self.get_price(&o.symbol);
                    Some(self.execute(o, price))
                }
                _ => self.execute_or_store(o),
            },
            (MarketStatus::Maintenance, _) => todo!(),
            _ => {
                self.store(o);
                None
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

    pub fn execute_or_store(&mut self, o: Order) -> Option<TradeFill> {
        let price = self.get_price(&o.symbol);
        if is_marketable(&o, price) {
            Some(self.execute(o, price))
        } else {
            self.store(o);
            None
        }
    }

    pub fn store(&mut self, o: Order) {
        self.stored_orders.push(o);
    }

    pub fn get_price(&self, symbol: &str) -> f64 {
        *self.prices.get(symbol).unwrap()
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
