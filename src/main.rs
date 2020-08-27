#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use simulator::{simulator::Simulator, api::Credentials, Account};
use rocket::State;
use rocket_contrib::json::Json;
use std::sync::RwLock;
use std::borrow::Borrow;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/account")]
fn get_account<'r>(simulator: State<'r, Simulator>, creds: Credentials) -> Json<&'r Account> {
    Json(simulator.inner().get_account())
} 

fn main() {
    rocket::ignite()
	.manage(Simulator::new())
	.mount("/", routes![index, get_account])
	.launch();
}
