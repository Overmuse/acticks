#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::State;
use rocket_contrib::{json::Json, uuid::Uuid};
use simulator::{
    account::Account,
    brokerage::Brokerage,
    credentials::Credentials,
    errors::{Error, Result},
    order::{Order, OrderIntent, OrderStatus},
    position::Position,
};

fn convert_uuid(id: Uuid) -> uuid::Uuid {
    uuid::Uuid::from_bytes(*id.as_bytes())
}

#[get("/account", rank = 1)]
fn get_account(brokerage: State<Brokerage>, _c: Credentials) -> Json<Account> {
    let account = brokerage.inner().get_account();
    Json(account)
}

#[get("/orders")]
fn get_orders(brokerage: State<Brokerage>, _c: Credentials) -> Json<Vec<Order>> {
    let orders: Vec<Order> = brokerage
        .inner()
        .get_orders()
        .values_mut()
        .map(|x| x.to_owned())
        .collect();
    Json(orders)
}

#[get("/orders/<id>")]
fn get_order_by_id(brokerage: State<Brokerage>, _c: Credentials, id: Uuid) -> Json<Order> {
    let id: uuid::Uuid = convert_uuid(id);
    let order: Order = brokerage.inner().get_order(id).unwrap();
    Json(order)
}

#[delete("/orders")]
fn cancel_orders(brokerage: State<Brokerage>, _c: Credentials) {
    brokerage.inner().modify_orders(|orders| {
        for order in orders.values_mut() {
            match order.status {
                OrderStatus::Filled | OrderStatus::Expired | OrderStatus::Canceled => (),
                _ => order
                    .cancel()
                    .expect("All other statuses should be cancelable"),
            }
        }
    })
}

#[delete("/orders/<id>")]
fn cancel_order_by_id(brokerage: State<Brokerage>, _c: Credentials, id: Uuid) -> Result<()> {
    let id: uuid::Uuid = convert_uuid(id);
    brokerage
        .inner()
        .modify_order(id, |o| -> Result<()> { o.cancel() })
}

#[get("/positions")]
fn get_positions(brokerage: State<Brokerage>, _c: Credentials) -> Json<Vec<Position>> {
    let positions: Vec<Position> = brokerage
        .inner()
        .get_positions()
        .values_mut()
        .map(|x| x.to_owned())
        .collect();
    Json(positions)
}

#[get("/positions/<symbol>")]
fn get_position_by_symbol(
    brokerage: State<Brokerage>,
    _c: Credentials,
    symbol: String,
) -> Result<Json<Position>> {
    let position: Option<Position> = brokerage.inner().get_position(symbol);
    match position {
        Some(x) => Ok(Json(x)),
        None => Err(Error::NotFound {
            msg: "{\"code\":40410000,\"message\":position does not exist}".to_string(),
        }),
    }
}

#[post("/orders", format = "json", data = "<oi>")]
fn post_order(brokerage: State<Brokerage>, _c: Credentials, oi: Json<OrderIntent>) -> Json<Order> {
    let order = brokerage.inner().post_order(oi.0);
    Json(order)
}

fn rocket() -> rocket::Rocket {
    let creds = Credentials::new();
    let cash = 10000000.0;
    rocket::ignite()
        .manage(Brokerage::new(cash))
        .attach(creds)
        .mount(
            "/",
            routes![
                get_account,
                get_orders,
                get_order_by_id,
                cancel_orders,
                cancel_order_by_id,
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
