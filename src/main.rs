#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use simulator::{simulator::Simulator, api::Credentials, Account, Order};
use rocket::State;
use rocket_contrib::json::Json;
use rocket::response::status;
use std::sync::{Arc, Mutex};

#[get("/account", rank = 1)]
fn get_account(simulator: State<Arc<Mutex<Simulator>>>, _creds: Credentials) -> Json<Account> {
    let account = Arc::clone(simulator.inner()).lock().unwrap().get_account();
    //let account = guard.get_account();
    Json(account)
} 

#[get("/account", rank = 2)]
fn get_account_unauthorized(_simulator: State<Arc<Mutex<Simulator>>>) -> status::Unauthorized<()> {
    status::Unauthorized::<()>(None)
}

#[get("/orders")]
fn get_orders(simulator: State<Arc<Mutex<Simulator>>>, _creds: Credentials) -> Json<Vec<Order>> {
    let orders: Vec<Order> = simulator
	.inner()
	.lock()
	.unwrap()
	.get_account()
	.get_orders();
    Json(orders)
}

//#[post("/orders")]
//fn post_order<'r>(simulator: State<'r, Simulator>, _creds: Credentials) -> Json<&'r Order> {
//    let account: &mut Account = simulator.inner().get_account().borrow_mut();
//    account.orders.push(Order {});
//    Json(&Order{})
//}

fn main() {
    let creds = Credentials::new();
    rocket::ignite()
	.manage(Arc::new(Mutex::new(Simulator::new(&creds))))
	.attach(creds)
	.mount("/", routes![get_account, get_account_unauthorized, get_orders])
	.launch();
}
