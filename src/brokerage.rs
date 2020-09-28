use crate::account::actors::{AccountManager, SetCash};
use crate::asset::{
    self,
    actors::{AssetManager, GetAssetBySymbol, GetAssets, SetAssets},
    types::Asset,
};
use crate::clock::Clock;
use crate::errors::{Error, Result};
use crate::exchange::{Exchange, TradeFill, TransmitOrder};
use crate::order::{
    CancelOrder, CancelOrders, GetOrderByClientOrderId, GetOrderById, GetOrders, Order,
    OrderIntent, OrderManager, PostOrder,
};
use crate::position::{self, actors::PositionManager, types::Position};
use actix::prelude::*;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Clone)]
pub struct Brokerage {}

impl Brokerage {
    pub fn get_clock(&self) -> Clock {
        // TODO: Make this dynamically pull from exchange
        Clock {
            timestamp: Utc::now(),
            is_open: true,
            next_open: Utc::now(),
            next_close: Utc::now(),
        }
    }

    pub async fn get_order(&self, id: Uuid) -> Result<Order> {
        OrderManager::from_registry()
            .send(GetOrderById { id })
            .await
            .unwrap()
            .clone()
            .ok_or(Error::NotFound)
    }

    pub async fn get_order_by_client_id(&self, client_id: &str, _nested: bool) -> Result<Order> {
        OrderManager::from_registry()
            .send(GetOrderByClientOrderId {
                client_order_id: client_id.to_string(),
            })
            .await
            .unwrap()
            .clone()
            .ok_or(Error::NotFound)
    }

    pub async fn cancel_orders(&self) {
        OrderManager::from_registry()
            .send(CancelOrders {})
            .await
            .unwrap();
    }

    pub async fn cancel_order(&self, id: Uuid) {
        OrderManager::from_registry()
            .send(CancelOrder(id))
            .await
            .unwrap();
    }

    //pub async fn update_price(&self, symbol: String, price: f64) {
    //    let potential_fills = self.exchange.write().unwrap().update_price(&symbol, price);
    //    potential_fills
    //        .iter()
    //        .for_each(|fill| self.update_from_fill(fill).await);
    //}

    pub async fn post_order(&self, o: OrderIntent) -> Result<Order> {
        let asset = asset::get_asset(&o.symbol).await?;
        let mut order: Order = Order::from_intent(&o, &asset);
        let o2 = order.clone();
        let s = self.clone();
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
            if let Some(fill) = potential_fill {
                s.update_from_fill(&fill).await.unwrap();
            }
        });
        Ok(o2)
    }

    async fn update_from_fill(&self, tf: &TradeFill) -> Result<()> {
        OrderManager::from_registry()
            .send(tf.clone())
            .await
            .map_err(|_| Error::Other)?;
        let asset = AssetManager::from_registry()
            .send(GetAssetBySymbol {
                symbol: tf.order.symbol.clone(),
            })
            .await
            .unwrap()
            .unwrap();
        PositionManager::from_registry()
            .send(tf.clone())
            .await
            .unwrap();
        AccountManager::from_registry()
            .send(tf.clone())
            .await
            .unwrap();
        Ok(())
    }
}
