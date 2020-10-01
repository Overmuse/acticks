use crate::asset;
use crate::errors::{Error, Result};
use crate::exchange::{self, Exchange, TransmitOrder};
use actix::prelude::*;
use actors::{
    CancelOrder, CancelOrders, GetOrderByClientOrderId, GetOrderById, GetOrders, OrderManager,
    PostOrder,
};
use chrono::Utc;
use std::collections::HashMap;
use types::{Order, OrderIntent};
use uuid::Uuid;

pub mod actors;
pub mod types;

pub async fn get_orders() -> HashMap<Uuid, Order> {
    OrderManager::from_registry()
        .send(GetOrders {})
        .await
        .unwrap()
}

pub async fn get_order(id: Uuid) -> Result<Order> {
    OrderManager::from_registry()
        .send(GetOrderById { id })
        .await
        .unwrap()
        .ok_or(Error::NotFound)
}

pub async fn get_order_by_client_id(client_id: &str, _nested: bool) -> Result<Order> {
    OrderManager::from_registry()
        .send(GetOrderByClientOrderId {
            client_order_id: client_id.to_string(),
        })
        .await
        .unwrap()
        .ok_or(Error::NotFound)
}

pub async fn cancel_orders() {
    OrderManager::from_registry()
        .send(CancelOrders {})
        .await
        .unwrap();
}

pub async fn cancel_order(id: Uuid) {
    OrderManager::from_registry()
        .send(CancelOrder(id))
        .await
        .unwrap();
}

pub async fn post_order(o: OrderIntent) -> Result<Order> {
    let asset = asset::get_asset(&o.symbol).await?;
    let mut order: Order = Order::from_intent(&o, &asset);
    let o2 = order.clone();
    tokio::spawn(async move {
        order.submitted_at = Some(Utc::now());
        order.updated_at = Some(Utc::now());
        OrderManager::from_registry()
            .send(PostOrder {
                order: order.clone(),
            })
            .await
            .unwrap();
        let potential_fill = Exchange::from_registry()
            .send(TransmitOrder(order))
            .await
            .unwrap();
        if let Some(fill) = potential_fill.unwrap() {
            exchange::update_from_fill(&fill).await.unwrap();
        }
    });
    Ok(o2)
}
