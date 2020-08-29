use crate::account::Account;
use crate::order::{Order, OrderIntent, OrderStatus};
use crate::position::{Position, Side};
use chrono::Utc;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde_json;
use std::sync::{Arc, RwLock};
use std::thread;

#[derive(Clone)]
pub struct Simulator {
    account: Arc<RwLock<Account>>,
    orders: Arc<RwLock<Vec<Order>>>,
    positions: Arc<RwLock<Vec<Position>>>,
    producer: FutureProducer,
}

impl Simulator {
    pub fn new(cash: f64) -> Self {
        let account = Arc::new(RwLock::new(Account::new(cash)));
        let orders = Arc::new(RwLock::new(vec![]));
        let positions = Arc::new(RwLock::new(vec![]));
        let producer = ClientConfig::new()
            .set("bootstrap.servers", "localhost:9092")
            .create()
            .unwrap();
        Simulator {
            account,
            orders,
            positions,
            producer,
        }
    }

    pub fn get_account(&self) -> Account {
        self.account.read().unwrap().clone()
    }

    pub fn modify_account<F>(&self, f: F)
    where
        F: FnOnce(&mut Account),
    {
        f(&mut self.account.write().unwrap())
    }

    pub fn get_orders(&self) -> Vec<Order> {
        self.orders.read().unwrap().clone()
    }

    pub fn modify_orders<F>(&self, f: F)
    where
        F: FnOnce(&mut Vec<Order>),
    {
        f(&mut self.orders.write().unwrap())
    }

    pub fn get_positions(&self) -> Vec<Position> {
        self.positions.read().unwrap().clone()
    }

    pub fn modify_positions<F>(&self, f: F)
    where
        F: FnOnce(&mut Vec<Position>),
    {
        f(&mut self.positions.write().unwrap())
    }

    pub fn schedule_order(&self, o: Order) {
        let orders = self.get_orders();
        let idx = &orders.iter().position(|o2| o2.id.to_hyphenated().to_string() == o.id.to_hyphenated().to_string()).unwrap();
        self.modify_orders(|os| {
            let mut order = &mut os[*idx];
            let time = Some(Utc::now());
            order.filled_qty = order.qty;
            order.updated_at = time;
            order.submitted_at = time;
            order.filled_at = time;
            order.filled_avg_price = Some(100.0);
            order.status = OrderStatus::Filled;
        });
        self.modify_positions(|ps| {
            ps.push(Position {
                asset_id: o.asset_id,
                symbol: o.symbol,
                exchange: "NYSE".to_string(),
                asset_class: o.asset_class,
                avg_entry_price: 100.0,
                qty: o.qty as i32,
                side: Side::Long,
                market_value: o.qty as f64 * 100.0,
                cost_basis: o.qty as f64 * 100.0,
                unrealized_pl: 0.0,
                unrealized_plpc: 0.0,
                unrealized_intraday_pl: 0.0,
                unrealized_intraday_plpc: 0.0,
                current_price: 100.0,
                lastday_price: 100.0,
                change_today: 0.0,
            });
        })

    }

    pub fn post_order(&self, o: OrderIntent) -> Order {
        let order: Order = Order::from_intent(o);
        self.modify_orders(|o| o.push(order.clone()));
        let o2 = order.clone();
        let payload = &serde_json::to_string(&order).unwrap();
        self.producer.send(
            FutureRecord::to("new_orders").key(&order.symbol).payload(payload),
            0,
        );
        let s = self.clone();
        thread::spawn(move || {
            s.schedule_order(o2);
        });
        order
    }
}
