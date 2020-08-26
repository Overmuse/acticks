#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use simulator::{simulator::Simulator, api::Credentials};
use rocket::State;
use rocket_contrib::json::Json;
use std::sync::Mutex;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/account/new")]
fn new_account(simulator: State<Mutex<Simulator>>) -> Json<Credentials> {
    let creds = simulator
	.inner()
	.lock()
	.unwrap()
	.create_account();
    Json(creds)
}

fn main() {
    rocket::ignite()
	.manage(Mutex::new(Simulator::new()))
	.mount("/", routes![index, new_account])
	.launch();
}
