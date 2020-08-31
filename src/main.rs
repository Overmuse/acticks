#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::State;
use rocket_contrib::{json::Json, uuid::Uuid};
use simulator::{
    account::Account,
    asset::Asset,
    brokerage::Brokerage,
    credentials::Credentials,
    errors::Result,
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

#[get("/assets")]
fn get_assets(brokerage: State<Brokerage>, _c: Credentials) -> Json<Vec<Asset>> {
    let assets: Vec<Asset> = brokerage.inner().get_assets().values().cloned().collect();
    Json(assets)
}

#[get("/assets/<symbol>")]
fn get_asset_by_symbol(
    brokerage: State<Brokerage>,
    _c: Credentials,
    symbol: String,
) -> Result<Json<Asset>> {
    let asset: Asset = brokerage.inner().get_asset(&symbol)?;
    Ok(Json(asset))
}

#[get("/orders")]
fn get_orders(brokerage: State<Brokerage>, _c: Credentials) -> Json<Vec<Order>> {
    let orders: Vec<Order> = brokerage.inner().get_orders().values().cloned().collect();
    Json(orders)
}

#[get("/orders/<id>")]
fn get_order_by_id(brokerage: State<Brokerage>, _c: Credentials, id: Uuid) -> Result<Json<Order>> {
    let id: uuid::Uuid = convert_uuid(id);
    let order: Order = brokerage.inner().get_order(id)?;
    Ok(Json(order))
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
        .values()
        .cloned()
        .collect();
    Json(positions)
}

#[get("/positions/<symbol>")]
fn get_position_by_symbol(
    brokerage: State<Brokerage>,
    _c: Credentials,
    symbol: String,
) -> Result<Json<Position>> {
    let position = brokerage.inner().get_position(&symbol)?;
    Ok(Json(position))
}

#[delete("/positions")]
fn close_positions(brokerage: State<Brokerage>, _c: Credentials) {
    let positions = brokerage.inner().get_positions();
    for position in positions.values() {
        brokerage.inner().close_position(&position.symbol).unwrap();
    }
}

#[post("/orders", format = "json", data = "<oi>")]
fn post_order(
    brokerage: State<Brokerage>,
    _c: Credentials,
    oi: Json<OrderIntent>,
) -> Result<Json<Order>> {
    let order = brokerage.inner().post_order(oi.0)?;
    Ok(Json(order))
}

fn rocket() -> rocket::Rocket {
    let creds = Credentials::new();
    let cash = 10000000.0;
    let symbols = vec!["AAPL".into(), "TSLA".into()];
    rocket::ignite()
        .manage(Brokerage::new(cash, symbols))
        .attach(creds)
        .mount(
            "/",
            routes![
                get_account,
                get_assets,
                get_asset_by_symbol,
                get_orders,
                get_order_by_id,
                cancel_orders,
                cancel_order_by_id,
                get_positions,
                get_position_by_symbol,
                close_positions,
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
