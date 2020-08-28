#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::response::status;
use rocket::State;
use rocket_contrib::json::Json;
use simulator::{
    account::Account,
    credentials::Credentials,
    order::{Order, OrderIntent},
    position::Position,
    simulator::Simulator,
};
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
    let orders: Vec<Order> = simulator.read().unwrap().get_account().get_orders();
    Json(orders)
}

#[get("/orders/<id>")]
fn get_order_by_id(
    simulator: State<Arc<RwLock<Simulator>>>,
    _creds: Credentials,
    id: rocket_contrib::uuid::Uuid,
) -> Json<Order> {
    //let id: uuid::Uuid = id.into_inner();
    let order: Order = simulator
        .read()
        .unwrap()
        .get_account()
        .get_orders()
        .into_iter()
        .find(|o| o.id.to_hyphenated().to_string() == id.to_hyphenated().to_string())
        .unwrap();
    Json(order)
}

#[get("/positions")]
fn get_positions(
    simulator: State<Arc<RwLock<Simulator>>>,
    _creds: Credentials,
) -> Json<Vec<Position>> {
    let positions: Vec<Position> = simulator.read().unwrap().get_account().get_positions();
    Json(positions)
}
#[post("/orders", format = "json", data = "<oi>")]
fn post_order(
    simulator: State<Arc<RwLock<Simulator>>>,
    _creds: Credentials,
    oi: Json<OrderIntent>,
) -> Json<Order> {
    let oi = oi.0;
    let order = simulator.write().unwrap().account.post_order(oi);
    Json(order)
}

fn rocket() -> rocket::Rocket {
    let creds = Credentials::new();
    rocket::ignite()
        .manage(Arc::new(RwLock::new(Simulator::new(&creds))))
        .attach(creds)
        .mount(
            "/",
            routes![
                get_account,
                get_account_unauthorized,
                get_orders,
                get_order_by_id,
                get_positions,
                post_order
            ],
        )
}

fn main() {
    rocket().launch();
}

#[cfg(test)]
mod test {
    use super::rocket;
    use crate::Credentials;
    use rocket::local::Client;
    use rocket::http::{Header, Status};
    use uuid::Uuid;

    #[test]
    fn get_account() {
        let client = Client::new(rocket()).expect("valid rocket instance");
        let mut req = client
            .get("/account");
        req.add_header(Header::new("APCA-API-KEY-ID", Uuid::new_v4().to_hyphenated().to_string()));
        req.add_header(Header::new("APCA-API-SECRET-KEY", Uuid::new_v4().to_hyphenated().to_string()));

        let response = req.dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
}
