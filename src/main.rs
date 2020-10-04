use acticks::{
    account, asset, clock, exchange,
    market::{self},
    order, position,
};
use actix::registry::SystemService;
use actix_web::middleware::Logger;
use actix_web::{
    web::{self, Json, Path, Query},
    App, HttpResponse, HttpServer, Result,
};
use env_logger;
use serde::Deserialize;
use uuid::Uuid;

async fn get_clock() -> Result<HttpResponse> {
    HttpResponse::Ok().json(clock::get_clock()).await
}

async fn get_account() -> Result<HttpResponse> {
    HttpResponse::Ok().json(account::get_account().await?).await
}

async fn get_assets() -> Result<HttpResponse> {
    let assets: Vec<asset::types::Asset> = asset::get_assets().await?.values().cloned().collect();
    HttpResponse::Ok().json(assets).await
}

async fn get_asset(symbol_or_id: Path<String>) -> Result<HttpResponse> {
    let possible_id = Uuid::parse_str(&symbol_or_id);
    let asset = match possible_id {
        Ok(id) => asset::get_asset_by_id(&id).await?,
        Err(_) => asset::get_asset(&symbol_or_id).await?,
    };
    HttpResponse::Ok().json(asset).await
}

async fn get_orders() -> Result<HttpResponse> {
    let mut orders: Vec<order::types::Order> =
        order::get_orders().await?.values().cloned().collect();
    orders.sort_unstable_by(|a, b| b.created_at.partial_cmp(&a.created_at).unwrap());
    HttpResponse::Ok().json(orders).await
}

async fn get_order_by_id(id: Path<Uuid>) -> Result<HttpResponse> {
    let order: order::types::Order = order::get_order(*id).await?;
    HttpResponse::Ok().json(order).await
}

#[derive(Deserialize)]
struct OrderQuery {
    client_order_id: Option<String>,
    nested: bool,
}

async fn get_order_by_client_id(params: Query<OrderQuery>) -> Result<HttpResponse> {
    let order: order::types::Order =
        order::get_order_by_client_id(&params.client_order_id.as_ref().unwrap(), params.nested)
            .await?;
    HttpResponse::Ok().json(order).await
}

async fn post_order(oi: Json<order::types::OrderIntent>) -> Result<HttpResponse> {
    let order = order::post_order(oi.into_inner()).await?;
    HttpResponse::Ok().json(order).await
}

async fn cancel_orders() -> Result<HttpResponse> {
    order::cancel_orders().await?;
    HttpResponse::Ok().await
}

async fn cancel_order_by_id(id: Path<Uuid>) -> Result<HttpResponse> {
    order::cancel_order(*id).await?;
    HttpResponse::Ok().await
}

async fn get_positions() -> Result<HttpResponse> {
    let positions: Vec<position::types::Position> =
        position::get_positions().await?.values().cloned().collect();
    HttpResponse::Ok().json(positions).await
}

async fn get_position_by_symbol(symbol: Path<String>) -> Result<HttpResponse> {
    let position = position::get_position(symbol.to_string()).await?;
    HttpResponse::Ok().json(position).await
}

async fn close_position(symbol: Path<String>) -> Result<HttpResponse> {
    position::close_position(symbol.to_string()).await?;
    HttpResponse::Ok().await
}

async fn close_positions() -> Result<HttpResponse> {
    position::close_positions().await?;
    HttpResponse::Ok().await
}

async fn initialize_actors(
    cash: f64,
    symbols: Vec<String>,
) -> Result<actix::prelude::Request<market::polygon::PolygonMarket, market::Start>> {
    account::actors::AccountManager::from_registry()
        .send(account::actors::SetCash(cash))
        .await
        .unwrap();
    asset::actors::AssetManager::from_registry()
        .send(asset::actors::SetAssets {
            symbols: symbols.clone(),
        })
        .await
        .unwrap();
    let assets: Vec<asset::types::Asset> = symbols
        .iter()
        .map(|x| asset::types::Asset::from_symbol(x))
        .collect();
    exchange::Exchange::from_registry()
        .send(exchange::SetAssets { assets })
        .await
        .unwrap();
    let market_addr = market::polygon::PolygonMarket::from_registry();
    market_addr
        .send(market::Initialize(symbols))
        .await
        .unwrap()?;
    market_addr.do_send(market::Subscribe(
        exchange::Exchange::from_registry().recipient(),
    ));
    market_addr.do_send(market::Subscribe(
        position::actors::PositionManager::from_registry().recipient(),
    ));
    Ok(market_addr.send(market::Start(3600)))
}

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cash: f64 = 1_000_000.0;
    let symbols = vec![
        "AAPL".into(),
    ];
    let market_fut = initialize_actors(cash, symbols).await?;
    let server_fut = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
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
            .route("/positions/{symbol}", web::delete().to(close_position))
            .route("/positions", web::delete().to(close_positions))
    })
    .bind("127.0.0.1:8000")?
    .run();
    futures::future::select(market_fut, server_fut).await;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::{
        http::StatusCode,
        test::{self, TestRequest},
        web, App,
    };

    #[actix_rt::test]
    async fn test_get_orders() {
        let mut app =
            test::init_service(App::new().route("/orders", web::get().to(get_orders))).await;
        let req = TestRequest::get().uri("/orders").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
