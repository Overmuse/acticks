use actix_web::middleware::Logger;
use actix_web::{
    web::{self, Data, Json, Path, Query},
    App, HttpResponse, HttpServer, Result,
};
use env_logger::Env;
use serde::Deserialize;
use simulator::{
    account,
    asset::Asset,
    brokerage::Brokerage,
    order::{Order, OrderIntent},
    position::Position,
};
use uuid::Uuid;

async fn get_clock(brokerage: Data<Brokerage>) -> Result<HttpResponse> {
    HttpResponse::Ok().json(brokerage.get_clock()).await
}

async fn get_account() -> Result<HttpResponse> {
    HttpResponse::Ok().json(account::get_account().await).await
}

async fn get_assets(brokerage: Data<Brokerage>) -> Result<HttpResponse> {
    let assets: Vec<Asset> = brokerage.get_assets().await.values().cloned().collect();
    HttpResponse::Ok().json(assets).await
}

async fn get_asset(brokerage: Data<Brokerage>, symbol_or_id: Path<String>) -> Result<HttpResponse> {
    let possible_id = Uuid::parse_str(&symbol_or_id);
    let asset = match possible_id {
        Ok(id) => brokerage.get_asset_by_id(&id).await?,
        Err(_) => brokerage.get_asset(&symbol_or_id).await?,
    };
    HttpResponse::Ok().json(asset).await
}

async fn get_orders(brokerage: Data<Brokerage>) -> Result<HttpResponse> {
    let mut orders: Vec<Order> = brokerage.get_orders().await.values().cloned().collect();
    orders.sort_unstable_by(|a, b| b.created_at.partial_cmp(&a.created_at).unwrap());
    HttpResponse::Ok().json(orders).await
}

async fn get_order_by_id(brokerage: Data<Brokerage>, id: Path<Uuid>) -> Result<HttpResponse> {
    let order: Order = brokerage.get_order(*id).await?;
    HttpResponse::Ok().json(order).await
}

#[derive(Deserialize)]
struct OrderQuery {
    client_order_id: Option<String>,
    nested: bool,
}

async fn get_order_by_client_id(
    brokerage: Data<Brokerage>,
    params: Query<OrderQuery>,
) -> Result<HttpResponse> {
    let order: Order = brokerage
        .get_order_by_client_id(&params.client_order_id.as_ref().unwrap(), params.nested)
        .await?;
    HttpResponse::Ok().json(order).await
}

async fn post_order(brokerage: Data<Brokerage>, oi: Json<OrderIntent>) -> Result<HttpResponse> {
    let order = brokerage.post_order(oi.into_inner()).await?;
    HttpResponse::Ok().json(order).await
}

async fn cancel_orders(brokerage: Data<Brokerage>) -> Result<HttpResponse> {
    brokerage.cancel_orders().await;
    HttpResponse::Ok().await
}

async fn cancel_order_by_id(brokerage: Data<Brokerage>, id: Path<Uuid>) -> Result<HttpResponse> {
    brokerage.cancel_order(*id).await;
    HttpResponse::Ok().await
}

async fn get_positions(brokerage: Data<Brokerage>) -> Result<HttpResponse> {
    let positions: Vec<Position> = brokerage.get_positions().values().cloned().collect();
    HttpResponse::Ok().json(positions).await
}

async fn get_position_by_symbol(
    brokerage: Data<Brokerage>,
    symbol: Path<String>,
) -> Result<HttpResponse> {
    let position = brokerage.get_position(&symbol)?;
    HttpResponse::Ok().json(position).await
}

async fn close_positions(brokerage: Data<Brokerage>) -> Result<HttpResponse> {
    let positions = brokerage.get_positions();
    for position in positions.values() {
        brokerage.close_position(&position.symbol).await?;
    }
    HttpResponse::Ok().await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::from_env(Env::default().default_filter_or("debug")).init();
    let cash = 1_000_000.0;
    let symbols = vec![
        "PROP".into(),
        "IDEX".into(),
        "WORK".into(),
        "SUNW".into(),
        "DRD".into(),
    ];
    let brokerage = Brokerage::new(cash, symbols).await;
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .data(brokerage.clone())
            .route("/account", web::get().to(get_account))
            .route("/clock", web::get().to(get_clock))
            .route("/assets", web::get().to(get_assets))
            .route("/assets/{symbol_or_id}", web::get().to(get_asset))
            .route("/orders", web::get().to(get_orders))
            .route("/orders/{id}", web::get().to(get_order_by_id))
            .route(
                "/orders:by_client_order_id",
                web::get().to(get_order_by_client_id),
            )
            .route("/orders", web::post().to(post_order))
            .route("/orders", web::delete().to(cancel_orders))
            .route("/orders/{id}", web::delete().to(cancel_order_by_id))
            .route("/positions", web::get().to(get_positions))
            .route("/positions/{symbol}", web::get().to(get_position_by_symbol))
            .route("/positions", web::delete().to(close_positions))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::{
        http::StatusCode,
        test::{self, TestRequest},
        web, App,
    };

    fn new_brokerage() -> Brokerage {
        Brokerage::new(100.0, vec!["PRPO".into(), "WORK".into()])
    }

    #[actix_rt::test]
    async fn test_get_orders() {
        let mut app = test::init_service(
            App::new()
                .data(new_brokerage())
                .route("/orders", web::get().to(get_orders)),
        )
        .await;
        let req = TestRequest::get().uri("/orders").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
