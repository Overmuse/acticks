#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use simulator::{simulator::Simulator, api::Credentials, Account, Order, Position};
use rocket::State;
use rocket_contrib::json::Json;
use rocket::response::status;
use std::sync::{Arc, RwLock};

#[get("/account", rank = 1)]
fn get_account(simulator: State<Arc<RwLock<Simulator>>>, _creds: Credentials) -> Json<Account> {
    let account = Arc::clone(simulator.inner()).read().unwrap().get_account();
    //let account = guard.get_account();
    Json(account)
} 

#[get("/account", rank = 2)]
fn get_account_unauthorized(_simulator: State<Arc<RwLock<Simulator>>>) -> status::Unauthorized<()> {
    status::Unauthorized::<()>(None)
}

#[get("/orders")]
fn get_orders(simulator: State<Arc<RwLock<Simulator>>>, _creds: Credentials) -> Json<Vec<Order>> {
    let orders: Vec<Order> = simulator
	.inner()
	.read()
	.unwrap()
	.get_account()
	.get_orders();
    Json(orders)
}

#[get("/positions")]
fn get_positions(simulator: State<Arc<RwLock<Simulator>>>, _creds: Credentials) -> Json<Vec<Position>> {
    let positions: Vec<Position> = simulator
	.inner()
	.read()
	.unwrap()
	.get_account()
	.get_positions();
    Json(positions)
}
#[post("/orders")]
fn post_order(simulator: State<Arc<RwLock<Simulator>>>, _creds: Credentials) -> Json<Order> {
    simulator.inner().write().unwrap().account.post_order(Order {});
    Json(Order {})
}

fn main() {
    let creds = Credentials::new();
    rocket::ignite()
	.manage(Arc::new(RwLock::new(Simulator::new(&creds))))
	.attach(creds)
	.mount("/", routes![get_account, get_account_unauthorized, get_orders, get_positions, post_order])
	.launch();
}
