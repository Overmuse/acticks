use actix_web::middleware::Logger;
use actix_web::{
    delete, get, post,
    web::{Data, Json, Path},
    App, HttpResponse, HttpServer, Result,
};
use env_logger::Env;
use simulator::{
    account::Account,
    asset::Asset,
    brokerage::Brokerage,
    //credentials::Credentials,
    order::{Order, OrderIntent, OrderStatus},
    position::Position,
};
use uuid::Uuid;

#[get("/account")]
async fn get_account(brokerage: Data<Brokerage>) -> Result<HttpResponse> {
    HttpResponse::Ok().json(brokerage.get_account()).await
}

#[get("/assets")]
async fn get_assets(brokerage: Data<Brokerage>) -> Result<HttpResponse> {
    let assets: Vec<Asset> = brokerage.get_assets().values().cloned().collect();
    HttpResponse::Ok().json(assets).await
}

#[get("/assets/{symbol}")]
async fn get_asset_by_symbol(
    brokerage: Data<Brokerage>,
    symbol: Path<String>,
) -> Result<HttpResponse> {
    let asset: Asset = brokerage.get_asset(&symbol)?;
    HttpResponse::Ok().json(asset).await
}

#[get("/orders")]
async fn get_orders(brokerage: Data<Brokerage>) -> Result<HttpResponse> {
    let mut orders: Vec<Order> = brokerage.get_orders().values().cloned().collect();
    orders.sort_unstable_by(|a, b| b.created_at.partial_cmp(&a.created_at).unwrap());
    HttpResponse::Ok().json(orders).await
}

#[get("/orders/{id}")]
async fn get_order_by_id(brokerage: Data<Brokerage>, id: Path<Uuid>) -> Result<HttpResponse> {
    let order: Order = brokerage.get_order(*id)?;
    HttpResponse::Ok().json(order).await
}

#[delete("/orders")]
async fn cancel_orders(brokerage: Data<Brokerage>) -> Result<HttpResponse> {
    brokerage.modify_orders(|orders| {
        for order in orders.values_mut() {
            match order.status {
                OrderStatus::Filled | OrderStatus::Expired | OrderStatus::Canceled => (),
                _ => order
                    .cancel()
                    .expect("All other statuses should be cancelable"),
            }
        }
    });
    HttpResponse::Ok().await
}

#[delete("/orders/{id}")]
async fn cancel_order_by_id(brokerage: Data<Brokerage>, id: Path<Uuid>) -> Result<HttpResponse> {
    brokerage.modify_order(*id, |o| o.cancel());
    HttpResponse::Ok().await
}

#[get("/positions")]
async fn get_positions(brokerage: Data<Brokerage>) -> Result<HttpResponse> {
    let positions: Vec<Position> = brokerage.get_positions().values().cloned().collect();
    HttpResponse::Ok().json(positions).await
}

#[get("/positions/{symbol}")]
async fn get_position_by_symbol(
    brokerage: Data<Brokerage>,
    symbol: Path<String>,
) -> Result<HttpResponse> {
    let position = brokerage.get_position(&symbol)?;
    HttpResponse::Ok().json(position).await
}

#[delete("/positions")]
async fn close_positions(brokerage: Data<Brokerage>) -> Result<HttpResponse> {
    let positions = brokerage.get_positions();
    for position in positions.values() {
        brokerage.close_position(&position.symbol)?;
    }
    HttpResponse::Ok().await
}

#[post("/orders")]
async fn post_order(brokerage: Data<Brokerage>, oi: Json<OrderIntent>) -> Result<HttpResponse> {
    let order = brokerage.post_order(oi.into_inner())?;
    HttpResponse::Ok().json(order).await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //let creds = Credentials::new();
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    let cash = 1_000_000.0;
    let symbols = vec!["AAPL".into(), "TSLA".into()];
    let brokerage = Brokerage::new(cash, symbols);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .data(brokerage.clone())
            .service(get_account)
            .service(get_assets)
            .service(get_asset_by_symbol)
            .service(get_orders)
            .service(get_order_by_id)
            .service(post_order)
            .service(cancel_orders)
            .service(cancel_order_by_id)
            .service(get_positions)
            .service(get_position_by_symbol)
            .service(close_positions)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
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
    fn orders() {
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

        let mut response = req.dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string().unwrap(), "[]");
    }
}
