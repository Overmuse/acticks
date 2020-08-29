use crate::account::Account;
use crate::order::{Order, OrderIntent};
use crate::position::Position;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Simulator {
    pub account: Arc<RwLock<Account>>,
    pub orders: Arc<RwLock<Vec<Order>>>,
    pub positions: Arc<RwLock<Vec<Position>>>,
}

impl Simulator {
    pub fn new(cash: f64) -> Self {
        let account = Arc::new(RwLock::new(Account::new(cash)));
        let orders = Arc::new(RwLock::new(vec![]));
        let positions = Arc::new(RwLock::new(vec![]));
        Simulator {
            account,
            orders,
            positions,
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

    pub fn post_order(&self, o: OrderIntent) -> Order {
        let order: Order = Order::from_intent(o);
        self.modify_orders(|o| o.push(order.clone()));
        order
    }
}
