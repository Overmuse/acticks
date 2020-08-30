use crate::order::{Order, OrderStatus, OrderType, Side};
use chrono::{DateTime, Utc};
use std::collections::{hash_map::RandomState, HashSet};

#[derive(Clone)]
pub struct TradeFill {
    pub time: DateTime<Utc>,
    pub qty: i32,
    pub price: f64,
    pub order: Order,
}

pub struct Exchange {
    pub assets: HashSet<String, RandomState>,
    pub stored_orders: Vec<Order>,
}

impl Exchange {
    pub fn new() -> Self {
        Self {
            assets: HashSet::new(),
            stored_orders: vec![],
        }
    }

    pub fn transmit_order(&mut self, o: Order) -> Option<TradeFill> {
        if self.is_open() {
            match o.order_type {
                OrderType::Market => {
                    let price = self.get_price(&o.symbol);
                    Some(self.execute(o, price))
                }
                _ => self.execute_or_store(o),
            }
        } else {
            self.store(o);
            None
        }
    }

    pub fn is_open(&self) -> bool {
        true
    }

    pub fn execute(&mut self, mut order: Order, price: f64) -> TradeFill {
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

    pub fn get_price(&self, symbol: &str) -> f64 {
        100.0
    }
}

fn is_marketable(o: &Order, price: f64) -> bool {
    match o.order_type {
        OrderType::Market => true,
        OrderType::Limit { limit_price } => match o.side {
            Side::Buy => limit_price >= price,
            Side::Sell => limit_price <= price,
        },
        OrderType::Stop { stop_price } => match o.side {
            Side::Buy => stop_price <= price,
            Side::Sell => stop_price >= price,
        },
        _ => false,
    }
}
