use crate::account::Account;
use crate::exchange::Exchange;
use crate::order::{Order, OrderIntent, OrderStatus};
use crate::position::{Position, Side};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use uuid::Uuid;

#[derive(Clone)]
pub struct Simulator {
    account: Arc<RwLock<Account>>,
    orders: Arc<RwLock<HashMap<Uuid, Order>>>,
    positions: Arc<RwLock<HashMap<String, Position>>>,
    exchange: Arc<RwLock<Exchange>>,
}

impl Simulator {
    pub fn new(cash: f64) -> Self {
        let account = Arc::new(RwLock::new(Account::new(cash)));
        let orders = Arc::new(RwLock::new(HashMap::new()));
        let positions = Arc::new(RwLock::new(HashMap::new()));
        let exchange = Arc::new(RwLock::new(Exchange::new()));
        Simulator {
            account,
            orders,
            positions,
            exchange,
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

    pub fn get_orders(&self) -> HashMap<Uuid, Order> {
        self.orders.read().unwrap().clone()
    }

    pub fn get_order(&self, id: Uuid) -> Option<Order> {
        self.orders.write().unwrap().get_mut(&id).cloned()
    }

    pub fn modify_orders<F>(&self, f: F)
    where
        F: FnOnce(&mut HashMap<Uuid, Order>),
    {
        f(&mut self.orders.write().unwrap())
    }

    pub fn get_positions(&self) -> HashMap<String, Position> {
        self.positions.read().unwrap().clone()
    }

    pub fn get_position(&self, symbol: String) -> Option<Position> {
        self.positions.write().unwrap().get_mut(&symbol).cloned()
    }

    pub fn modify_positions<F>(&self, f: F)
    where
        F: FnOnce(&mut HashMap<String, Position>),
    {
        f(&mut self.positions.write().unwrap())
    }

    pub fn schedule_order(&self, o: Order) {}

    pub fn post_order(&self, o: OrderIntent) -> Order {
        let order: Order = Order::from_intent(o);
        let o2 = order.clone();
        let s = self.clone();
        thread::spawn(move || {
            s.modify_orders(|o| {
                o.insert(order.id, order.clone());
            });
            let executed = s.exchange.write().unwrap().transmit_order(order);
            if let Some(o) = executed {
                s.modify_orders(|os| {
                    os.insert(o.id, o.clone());
                });
                s.modify_positions(|ps| {
                    ps.insert(
                        o.symbol.clone(),
                        Position {
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
                        },
                    );
                })
            }
        });
        o2
    }
}
