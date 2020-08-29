use crate::account::Account;
use crate::order::{OrderIntent, Order};
use crate::position::Position;

#[derive(Clone)]
pub struct Simulator {
    pub account: Account,
    pub orders: Vec<Order>,
    pub positions: Vec<Position>,
}

impl Simulator {
    pub fn new(cash: f64) -> Self {
        let account = Account::new(cash);
        let orders = vec!();
        let positions = vec!();
        Simulator { account, orders, positions }
    }

    pub fn get_account(&self) -> Account {
        self.account.clone()
    }
    
    pub fn get_orders(&self) -> Vec<Order> {
        self.orders.clone()
    }

    pub fn get_positions(&self) -> Vec<Position> {
        self.positions.clone()
    }
    
    pub fn post_order(&mut self, o: OrderIntent) -> Order {
        let order: Order = Order::from_intent(o);
        self.orders.push(order.clone());
        order
    }
}
