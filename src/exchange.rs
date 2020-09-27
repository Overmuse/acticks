use crate::market::{PolygonMarket, TickerTrade};
use crate::order::{Order, OrderType, Side};
use actix_web::web::Data;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tokio::stream::StreamExt;

#[derive(Clone, Debug)]
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
    pub prices: HashMap<String, f64>,
}

impl Exchange {
    pub fn new() -> Self {
        let prices = HashMap::new();
        Self {
            stored_orders: vec![],
            prices,
        }
    }

    pub fn create(market: Data<Mutex<PolygonMarket>>) -> Arc<RwLock<Self>> {
        let me = Arc::new(RwLock::new(Exchange::new()));
        let me2 = me.clone();
        actix_web::rt::spawn(async move {
            Exchange::update_loop(me2, market.clone()).await;
        });
        me
    }

    async fn update_loop(me: Arc<RwLock<Self>>, market: Data<Mutex<PolygonMarket>>) {
        let mut x = market.lock().unwrap();
        while let Some(msg) = x.queue.next().await {
            let TickerTrade(ticker, trade) = msg.unwrap().into_inner();
            me.write().unwrap().update_price(&ticker, trade.price);
        }
    }

    pub fn transmit_order(&mut self, o: Order) -> Option<TradeFill> {
        match (&self.market_status(), o.extended_hours) {
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
        println!("{}, {}", &symbol, price);
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

//impl Default for Exchange {
//    fn default() -> Self {
//        let assets: Vec<Asset> = vec![];
//        Self::new(assets)
//    }
//}

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
