use std::default::Default;
use serde::Serialize;
use uuid::Uuid;

use crate::api::Credentials;
use crate::account_configurations::AccountConfig;

pub mod account_configurations;
pub mod api;
pub mod simulator;



#[derive(Debug, Serialize, Clone)]
pub struct Position {

}

#[derive(Debug, Serialize, Clone)]
pub struct Order {

}

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

    pub fn post_order(&mut self, o: Order) {
        self.orders.push(o)
    }
}

