use crate::account::Account;
use crate::exchange::{Exchange, TradeFill};
use crate::order::{Order, OrderIntent, OrderStatus};
use crate::position::{Position, Side};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use uuid::Uuid;

#[derive(Clone)]
pub struct Simulator {
    account: Arc<RwLock<Account>>,
    orders: Arc<RwLock<HashMap<Uuid, Order>>>,
    positions: Arc<RwLock<HashMap<String, Position>>>,
    exchange: Arc<RwLock<Exchange>>,
}

impl Simulator {
    pub fn new(cash: f64) -> Self {
        let account = Arc::new(RwLock::new(Account::new(cash)));
        let orders = Arc::new(RwLock::new(HashMap::new()));
        let positions = Arc::new(RwLock::new(HashMap::new()));
        let exchange = Arc::new(RwLock::new(Exchange::new()));
        Simulator {
            account,
            orders,
            positions,
            exchange,
        }
    }

    pub fn get_account(&self) -> Account {
        self.account.read().unwrap().clone()
    }

    pub fn modify_account<F>(&self, f: F)
    where
        F: FnOnce(&mut Account),
    {
        f(&mut self.account.write().unwrap())
    }

    pub fn get_orders(&self) -> HashMap<Uuid, Order> {
        self.orders.read().unwrap().clone()
    }

    pub fn get_order(&self, id: Uuid) -> Option<Order> {
        self.orders.write().unwrap().get_mut(&id).cloned()
    }

    pub fn modify_orders<F>(&self, f: F)
    where
        F: FnOnce(&mut HashMap<Uuid, Order>),
    {
        f(&mut self.orders.write().unwrap())
    }

    pub fn get_positions(&self) -> HashMap<String, Position> {
        self.positions.read().unwrap().clone()
    }

    pub fn get_position(&self, symbol: String) -> Option<Position> {
        self.positions.write().unwrap().get_mut(&symbol).cloned()
    }

    pub fn modify_positions<F>(&self, f: F)
    where
        F: FnOnce(&mut HashMap<String, Position>),
    {
        f(&mut self.positions.write().unwrap())
    }

    pub fn post_order(&self, o: OrderIntent) -> Order {
        let order: Order = Order::from_intent(o);
        let o2 = order.clone();
        let mut s = self.clone();
        thread::spawn(move || {
            s.modify_orders(|o| {
                o.insert(order.id, order.clone());
            });
            let potential_fill = s.exchange.write().unwrap().transmit_order(order);
            if let Some(fill) = potential_fill {
                s.update_from_fill(&fill);
            }
        });
        o2
    }

    fn update_from_fill(&mut self, tf: &TradeFill) {
        self.modify_orders(|os| {
            let order = os.get_mut(&tf.order.id).unwrap();
            let time = Some(tf.time);
            order.filled_qty = order.qty;
            order.updated_at = time;
            order.submitted_at = time;
            order.filled_at = time;
            order.filled_avg_price = Some(tf.price);
            order.status = OrderStatus::Filled;
        });
        self.modify_positions(|ps| {
            ps.insert(
                tf.order.symbol.clone(),
                Position {
                    asset_id: tf.order.asset_id,
                    symbol: tf.order.symbol.clone(),
                    exchange: "NYSE".to_string(),
                    asset_class: tf.order.asset_class.clone(),
                    avg_entry_price: tf.price,
                    qty: tf.qty as i32,
                    side: Side::Long,
                    market_value: tf.qty as f64 * tf.price,
                    cost_basis: tf.qty as f64 * tf.price,
                    unrealized_pl: 0.0,
                    unrealized_plpc: 0.0,
                    unrealized_intraday_pl: 0.0,
                    unrealized_intraday_plpc: 0.0,
                    current_price: tf.price,
                    lastday_price: tf.price,
                    change_today: 0.0,
                },
            );
        });
        self.modify_account(|account| {
            let cost_basis = tf.price * tf.qty as f64;
            account.cash -= cost_basis;
            if tf.qty > 0 {
                account.long_market_value += cost_basis
            } else {
                account.short_market_value -= cost_basis
            };
            account.initial_margin += 0.5 * cost_basis;
            account.daytrade_count += 1;
            account.buying_power =
                (account.equity - account.initial_margin).max(0.0) * account.multiplier;
            account.daytrading_buying_power = account.buying_power;
            account.regt_buying_power = account.buying_power / 2.;
        })
    }
}
