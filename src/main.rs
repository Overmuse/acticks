#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use simulator::{simulator::Simulator, api::Credentials, Account, Order, Position};
use rocket::State;
use rocket_contrib::json::Json;
use rocket::response::status;

#[get("/account", rank = 1)]
fn get_account<'r>(simulator: State<'r, Simulator>, _creds: Credentials) -> Json<&'r Account> {
    Json(simulator.inner().get_account())
} 

#[get("/account", rank = 2)]
fn get_account_unauthorized(_simulator: State<Simulator>) -> status::Unauthorized<()> {
    status::Unauthorized::<()>(None)
}

#[get("/orders")]
fn get_orders<'r>(simulator: State<'r, Simulator>, _creds: Credentials) -> Json<&'r Vec<Order>> {
    Json(&simulator.inner().get_account().get_orders())
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
	.manage(Simulator::new(&creds))
	.attach(creds)
	.mount("/", routes![get_account, get_account_unauthorized, get_orders])
	.launch();
}
