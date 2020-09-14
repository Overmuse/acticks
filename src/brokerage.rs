use crate::account::Account;
use crate::asset::Asset;
use crate::errors::{Error, Result};
use crate::exchange::{Exchange, TradeFill};
use crate::order::{self, Order, OrderIntent, OrderStatus};
use crate::position::{self, Position};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use uuid::Uuid;

#[derive(Clone)]
pub struct Brokerage {
    account: Arc<RwLock<Account>>,
    assets: Arc<RwLock<HashMap<String, Asset>>>,
    orders: Arc<RwLock<HashMap<Uuid, Order>>>,
    positions: Arc<RwLock<HashMap<String, Position>>>,
    exchange: Arc<RwLock<Exchange>>,
}

impl Brokerage {
    pub fn new(cash: f64, symbols: Vec<String>) -> Self {
        let account = Arc::new(RwLock::new(Account::new(cash)));
        let orders = Arc::new(RwLock::new(HashMap::new()));
        let positions = Arc::new(RwLock::new(HashMap::new()));
        let assets: Vec<Asset> = symbols.iter().map(|x| Asset::from_symbol(x)).collect();
        let exchange = Arc::new(RwLock::new(Exchange::new(assets.clone())));
        let mapping: HashMap<String, _> = symbols.iter().cloned().zip(assets).collect();
        Brokerage {
            account,
            assets: Arc::new(RwLock::new(mapping)),
            orders,
            positions,
            exchange,
        }
    }

    pub fn get_account(&self) -> Account {
        self.account.read().unwrap().clone()
    }

    pub fn modify_account<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut Account) -> T,
    {
        f(&mut self.account.write().unwrap())
    }

    pub fn get_assets(&self) -> HashMap<String, Asset> {
        self.assets.read().unwrap().clone()
    }

    pub fn get_asset(&self, symbol: &str) -> Result<Asset> {
        self.assets
            .read()
            .unwrap()
            .get(symbol)
            .cloned()
            .ok_or(Error::NotFound)
    }

    pub fn get_orders(&self) -> HashMap<Uuid, Order> {
        self.orders.read().unwrap().clone()
    }

    pub fn get_order(&self, id: Uuid) -> Result<Order> {
        self.orders
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(Error::NotFound)
    }

    pub fn modify_orders<F>(&self, f: F)
    where
        F: FnOnce(&mut HashMap<Uuid, Order>),
    {
        f(&mut self.orders.write().unwrap())
    }

    pub fn modify_order<F, T>(&self, id: Uuid, f: F) -> Result<T>
    where
        F: FnOnce(&mut Order) -> Result<T>,
    {
        let mut orders = self.orders.write().unwrap();
        let mut order = orders.get_mut(&id).ok_or(Error::NotFound)?;
        f(&mut order)
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

    pub fn update_price(&self, symbol: String, price: f64) {
        let potential_fills = self.exchange.write().unwrap().update_price(&symbol, price);
        potential_fills
            .iter()
            .for_each(|fill| self.update_from_fill(fill));
    }

    pub async fn post_order(&self, o: OrderIntent) -> Result<Order> {
        let asset = self.get_asset(&o.symbol)?;
        let mut order: Order = Order::from_intent(&o, &asset);
        let o2 = order.clone();
        let s = self.clone();
        tokio::spawn(async move {
            order.submitted_at = Some(Utc::now());
            order.updated_at = Some(Utc::now());
            s.modify_orders(|o| {
                o.insert(order.id, order.clone());
            });
            let potential_fill = s.exchange.write().unwrap().transmit_order(order);
            if let Some(fill) = potential_fill {
                s.update_from_fill(&fill);
            }
        });
        Ok(o2)
    }

    fn update_from_fill(&self, tf: &TradeFill) {
        let asset = self.get_asset(&tf.order.symbol).unwrap();
        self.modify_order(tf.order.id, |order| {
            let time = Some(tf.time);
            order.filled_qty = order.qty;
            order.updated_at = time;
            order.filled_at = time;
            order.filled_avg_price = Some(tf.price);
            order.status = OrderStatus::Filled;
            Ok(())
        })
        .unwrap();
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
        });
        self.modify_account(|account| {
            let cost_basis = tf.price * tf.qty as f64;
            account.cash -= cost_basis;
            if tf.qty > 0 {
                account.long_market_value += cost_basis
            } else {
                account.short_market_value += cost_basis
            };
            account.initial_margin += 0.5 * cost_basis;
            account.daytrade_count += 1;
            account.daytrading_buying_power =
                (account.equity - account.initial_margin).max(0.0) * account.multiplier;
            account.regt_buying_power = account.buying_power / 2.;
        })
    }
}
