use crate::account::actors::{AccountManager, SetCash};
use crate::asset::{Asset, AssetManager, GetAssetById, GetAssetBySymbol, GetAssets, SetAssets};
use crate::clock::Clock;
use crate::errors::{Error, Result};
use crate::exchange::{Exchange, TradeFill};
use crate::order::{
    self, CancelOrder, CancelOrders, GetOrderByClientOrderId, GetOrderById, GetOrders, Order,
    OrderIntent, OrderManager, PostOrder,
};
use crate::position::{self, Position};
use actix::prelude::*;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Clone)]
pub struct Brokerage {
    positions: Arc<RwLock<HashMap<String, Position>>>,
    exchange: Arc<RwLock<Exchange>>,
}

impl Brokerage {
    pub async fn new(cash: f64, symbols: Vec<String>) -> Self {
        AccountManager::from_registry()
            .send(SetCash(cash))
            .await
            .unwrap();
        AssetManager::from_registry()
            .send(SetAssets { symbols })
            .await
            .unwrap();
        let assets = AssetManager::from_registry()
            .send(GetAssets {})
            .await
            .unwrap();
        let positions = Arc::new(RwLock::new(HashMap::new()));
        let exchange = Arc::new(RwLock::new(Exchange::new(
            assets.values().cloned().collect(),
        )));
        Brokerage {
            positions,
            exchange,
        }
    }

    pub fn get_clock(&self) -> Clock {
        // TODO: Make this dynamically pull from exchange
        Clock {
            timestamp: Utc::now(),
            is_open: true,
            next_open: Utc::now(),
            next_close: Utc::now(),
        }
    }

    pub async fn get_assets(&self) -> HashMap<String, Asset> {
        AssetManager::from_registry()
            .send(GetAssets {})
            .await
            .unwrap()
            .clone()
    }

    pub async fn get_asset(&self, symbol: &str) -> Result<Asset> {
        AssetManager::from_registry()
            .send(GetAssetBySymbol {
                symbol: symbol.to_string(),
            })
            .await
            .unwrap()
            .clone()
            .ok_or(Error::NotFound)
    }

    pub async fn get_asset_by_id(&self, id: &Uuid) -> Result<Asset> {
        AssetManager::from_registry()
            .send(GetAssetById { id: *id })
            .await
            .unwrap()
            .clone()
            .ok_or(Error::NotFound)
        //let assets = self.assets.read().unwrap();
        //assets
        //    .values()
        //    .find(|asset| &asset.id == id)
        //    .cloned()
        //    .ok_or(Error::NotFound)
    }

    pub async fn get_orders(&self) -> HashMap<Uuid, Order> {
        OrderManager::from_registry()
            .send(GetOrders {})
            .await
            .unwrap()
            .clone()
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

    pub fn get_positions(&self) -> HashMap<String, Position> {
        self.positions.read().unwrap().clone()
    }

    pub fn get_position(&self, symbol: &str) -> Result<Position> {
        self.positions
            .read()
            .unwrap()
            .get(symbol)
            .cloned()
            .ok_or(Error::NotFound)
    }

    pub fn modify_positions<F>(&self, f: F)
    where
        F: FnOnce(&mut HashMap<String, Position>),
    {
        f(&mut self.positions.write().unwrap())
    }

    pub fn modify_position<F>(&self, symbol: &str, f: F)
    where
        F: FnOnce(&mut Position),
    {
        let mut positions = self.positions.write().unwrap();
        let mut position = positions.get_mut(symbol).unwrap();
        f(&mut position)
    }

    pub async fn close_position(&self, symbol: &str) -> Result<Order> {
        let position = self.get_position(symbol)?;
        let order_side = match position.side {
            position::Side::Long => order::Side::Sell,
            position::Side::Short => order::Side::Buy,
        };
        let order_intent = OrderIntent::new(symbol)
            .qty(position.qty.abs() as u32)
            .side(order_side);
        self.post_order(order_intent).await
    }

    //pub async fn update_price(&self, symbol: String, price: f64) {
    //    let potential_fills = self.exchange.write().unwrap().update_price(&symbol, price);
    //    potential_fills
    //        .iter()
    //        .for_each(|fill| self.update_from_fill(fill).await);
    //}

    pub async fn post_order(&self, o: OrderIntent) -> Result<Order> {
        let asset = self.get_asset(&o.symbol).await?;
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
            let potential_fill = s.exchange.write().unwrap().transmit_order(order);
            if let Some(fill) = potential_fill {
                s.update_from_fill(&fill).await.unwrap();
            }
        });
        Ok(o2)
    }

    async fn update_from_fill(&self, tf: &TradeFill) -> Result<()> {
        let asset = self
            .get_asset(&tf.order.symbol)
            .await
            .map_err(|_| Error::Other)?;
        OrderManager::from_registry()
            .send(tf.clone())
            .await
            .map_err(|_| Error::Other)?;
        self.modify_positions(|ps| {
            ps.entry(tf.order.symbol.clone())
                .and_modify(|p| {
                    p.qty += tf.qty;
                    if p.qty >= 0 {
                        p.side = position::Side::Long
                    } else {
                        p.side = position::Side::Short
                    };
                    p.cost_basis += tf.qty as f64 * tf.price;
                    p.update_with_price(tf.price);
                })
                .or_insert(Position {
                    asset_id: asset.id,
                    symbol: tf.order.symbol.clone(),
                    exchange: asset.exchange,
                    asset_class: asset.class,
                    avg_entry_price: tf.price,
                    qty: tf.qty,
                    side: {
                        if tf.qty > 0 {
                            position::Side::Long
                        } else {
                            position::Side::Short
                        }
                    },
                    market_value: tf.qty as f64 * tf.price,
                    cost_basis: tf.qty as f64 * tf.price,
                    unrealized_pl: 0.0,
                    unrealized_plpc: 0.0,
                    unrealized_intraday_pl: 0.0,
                    unrealized_intraday_plpc: 0.0,
                    current_price: tf.price,
                    lastday_price: tf.price,
                    change_today: 0.0,
                });
            ps.retain(|_, v| v.qty != 0);
        });
        AccountManager::from_registry()
            .send(tf.clone())
            .await
            .unwrap();
        Ok(())
    }
}
