use uuid::Uuid;
use serde::Serialize;
use crate::position::Position;
use crate::order::{Order, OrderIntent};
use crate::credentials::Credentials;
use crate::account_configurations::AccountConfig;

#[derive(Debug, Serialize, Clone)]
pub struct Account {
    id: Uuid,
    creds: Credentials,
    cash: f64,
    positions: Vec<Position>,
    pub orders: Vec<Order>,
    config: AccountConfig
}

impl Account {
    pub fn from_creds(creds: Credentials) -> Self {
	Account {
	    id: Uuid::new_v4(),
	    creds: creds,
	    cash: 0.0,
	    positions: vec!(),
	    orders: vec!(),
            config: AccountConfig::default(),
	}
    }	
    
    pub fn get_positions(&self) -> Vec<Position> {
        self.positions.clone()
    }

    pub fn get_orders(&self) -> Vec<Order> {
        self.orders.clone()
    }

    pub fn post_order(&mut self, o: OrderIntent) -> Order {
        let order: Order = Order::from_intent(o);
        println!("{:#?}", &order);
        self.orders.push(order.clone());
        order
    }
}
