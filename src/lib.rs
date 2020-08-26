use std::default::Default;
use uuid::Uuid;

use crate::api::Credentials;

pub mod api;
pub mod simulator;


#[derive(Debug)]
struct AccountConfig {
    short_allowed: bool,
}

impl Default for AccountConfig {
    fn default() -> AccountConfig {
	AccountConfig {
	    short_allowed: true
	}
    }
}

#[derive(Debug)]
struct Position {

}

#[derive(Debug)]
struct Order {

}

#[derive(Debug)]
struct Account {
    id: Uuid,
    creds: Credentials,
    cash: f64,
    positions: Vec<Position>,
    orders: Vec<Order>,
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
    
    fn get_positions(self) -> Vec<Position> {
        self.positions
    }

    fn get_orders(self) -> Vec<Order> {
        self.orders
    }
}
