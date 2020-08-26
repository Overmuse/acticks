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
fn new_account(simulator: State<SafeSimulator>) -> Json<Credentials> {
    Json(simulator.inner().simulator.lock().unwrap().create_account())
}

struct SafeSimulator {
    simulator: Mutex<Simulator>
}

impl SafeSimulator {
    fn new() -> Self {
	Self { simulator: Mutex::new(Simulator::new()) }
    }
}

fn main() {
    rocket::ignite()
	.manage(SafeSimulator::new())
	.mount("/", routes![index, new_account])
	.launch();
}
