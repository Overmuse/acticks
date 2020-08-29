#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::{response::status::NotFound, State};
use rocket_contrib::{json::Json, uuid::Uuid};
use simulator::{
    account::Account,
    credentials::Credentials,
    order::{Order, OrderIntent},
    position::Position,
    simulator::Simulator,
};

#[get("/account", rank = 1)]
fn get_account(simulator: State<Simulator>, _c: Credentials) -> Json<Account> {
    let account = simulator.inner().get_account();
    Json(account)
}

#[get("/orders")]
fn get_orders(simulator: State<Simulator>, _c: Credentials) -> Json<Vec<Order>> {
    let orders: Vec<Order> = simulator.inner().get_orders();
    Json(orders)
}

#[get("/orders/<id>")]
fn get_order_by_id(simulator: State<Simulator>, _c: Credentials, id: Uuid) -> Json<Order> {
    let order: Order = simulator
        .inner()
        .get_orders()
        .into_iter()
        .find(|o| o.id.to_hyphenated().to_string() == id.to_hyphenated().to_string())
        .unwrap();
    Json(order)
}

#[delete("/orders")]
fn delete_orders(simulator: State<Simulator>, _c: Credentials) {
    simulator.inner().modify_orders(|o| o.clear());
}

#[delete("/orders/<id>")]
fn delete_order_by_id(
    simulator: State<Simulator>,
    _c: Credentials,
    id: Uuid,
) -> Result<(), NotFound<String>> {
    let orders = simulator.inner().get_orders();
    let idx = &orders
        .iter()
        .position(|o| o.id.to_hyphenated().to_string() == id.to_hyphenated().to_string());
    match idx {
        Some(x) => {
            simulator.inner().modify_orders(|o| {
                o.remove(*x);
            });
            Ok(())
        }
        None => Err(NotFound(format!(
            "{{\"code\":40410000,\"message\":order not found for {}}}",
            id
        ))),
    }
}

#[get("/positions")]
fn get_positions(simulator: State<Simulator>, _c: Credentials) -> Json<Vec<Position>> {
    let positions: Vec<Position> = simulator.inner().get_positions();
    Json(positions)
}

#[get("/positions/<symbol>")]
fn get_position_by_symbol(
    simulator: State<Simulator>,
    _c: Credentials,
    symbol: String,
) -> Result<Json<Position>, NotFound<String>> {
    let positions: Vec<Position> = simulator.inner().get_positions();
    let idx = &positions.iter().position(|p| p.symbol == symbol);
    match idx {
        Some(x) => Ok(Json(positions[*x].clone())),
        None => Err(NotFound(
            "{\"code\":40410000,\"message\":position does not exist}".to_string(),
        )),
    }
}

#[post("/orders", format = "json", data = "<oi>")]
fn post_order(simulator: State<Simulator>, _c: Credentials, oi: Json<OrderIntent>) -> Json<Order> {
    let order = simulator.inner().post_order(oi.0);
    Json(order)
}

fn rocket() -> rocket::Rocket {
    let creds = Credentials::new();
    let cash = 10000000.0;
    rocket::ignite()
        .manage(Simulator::new(cash))
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
                get_position_by_symbol,
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
