#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::{State, response::status::NotFound};
use rocket_contrib::{json::Json, uuid::Uuid};
use simulator::{
    account::Account,
    credentials::Credentials,
    order::{Order, OrderIntent},
    position::Position,
    simulator::Simulator,
};
use std::sync::{Arc, RwLock};

#[get("/account", rank = 1)]
fn get_account(simulator: State<Arc<RwLock<Simulator>>>, _c: Credentials) -> Json<Account> {
    let account = Arc::clone(simulator.inner()).read().unwrap().get_account();
    //let account = guard.get_account();
    Json(account)
}

#[get("/orders")]
fn get_orders(simulator: State<Arc<RwLock<Simulator>>>, _c: Credentials) -> Json<Vec<Order>> {
    let orders: Vec<Order> = simulator.read().unwrap().get_account().get_orders();
    Json(orders)
}

#[get("/orders/<id>")]
fn get_order_by_id(
    simulator: State<Arc<RwLock<Simulator>>>,
    _c: Credentials,
    id: Uuid,
) -> Json<Order> {
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

#[delete("/orders")]
fn delete_orders(simulator: State<Arc<RwLock<Simulator>>>, _c: Credentials) {
    simulator.write().unwrap().account.orders.clear();
}

#[delete("/orders/<id>")]
fn delete_order_by_id(
    simulator: State<Arc<RwLock<Simulator>>>,
    _c: Credentials,
    id: Uuid,
) -> Result<(), NotFound<String>>{
    let orders = &mut simulator
        .write()
        .unwrap()
        .account
        .orders;
    let pos = &orders
        .iter()
        .position(|o| o.id.to_hyphenated().to_string() == id.to_hyphenated().to_string());
    match pos {
        Some(x) => {orders.remove(*x); Ok(())},
        None => Err(NotFound(format!("{{\"code\":40410000,\"message\":order not found for {}}}", id)))
    }
}

#[get("/positions")]
fn get_positions(simulator: State<Arc<RwLock<Simulator>>>, _c: Credentials) -> Json<Vec<Position>> {
    let positions: Vec<Position> = simulator.read().unwrap().get_account().get_positions();
    Json(positions)
}
#[post("/orders", format = "json", data = "<oi>")]
fn post_order(
    simulator: State<Arc<RwLock<Simulator>>>,
    _c: Credentials,
    oi: Json<OrderIntent>,
) -> Json<Order> {
    let order = simulator.write().unwrap().account.post_order(oi.0);
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
                get_orders,
                get_order_by_id,
                delete_orders,
                delete_order_by_id,
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
    use rocket::http::{Header, Status};
    use rocket::local::Client;
    use uuid::Uuid;

    #[test]
    fn get_account() {
        let client = Client::new(rocket()).unwrap();
        let mut req = client.get("/account");
        req.add_header(Header::new(
            "APCA-API-KEY-ID",
            Uuid::new_v4().to_hyphenated().to_string(),
        ));
        req.add_header(Header::new(
            "APCA-API-SECRET-KEY",
            Uuid::new_v4().to_hyphenated().to_string(),
        ));

        let response = req.dispatch();
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn get_orders() {
        let client = Client::new(rocket()).unwrap();
        let mut req = client.get("/orders");
        req.add_header(Header::new(
            "APCA-API-KEY-ID",
            Uuid::new_v4().to_hyphenated().to_string(),
        ));
        req.add_header(Header::new(
            "APCA-API-SECRET-KEY",
            Uuid::new_v4().to_hyphenated().to_string(),
        ));

        let response = req.dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
}
