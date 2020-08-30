use crate::order::{Order, OrderType, Side};
use chrono::{DateTime, Utc};
use std::collections::{hash_map::RandomState, HashSet};

#[derive(Clone)]
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
    pub assets: HashSet<String, RandomState>,
    pub stored_orders: Vec<Order>,
    pub market_status: MarketStatus,
}

impl Exchange {
    pub fn new() -> Self {
        Self {
            assets: HashSet::new(),
            stored_orders: vec![],
            market_status: MarketStatus::PreOpen,
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

    pub fn execute(&mut self, order: Order, price: f64) -> TradeFill {
        TradeFill {
            time: Utc::now(),
            qty: order.qty as i32,
            price,
            order: order.clone(),
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

    pub fn get_price(&self, _symbol: &str) -> f64 {
        100.0
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
