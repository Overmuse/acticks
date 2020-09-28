use crate::asset::types::{Asset, AssetClass};
use crate::errors::{Error, Result};
use crate::exchange::TradeFill;
use crate::utils::*;
use actix::prelude::*;
use chrono::{DateTime, Utc};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Neg;
use uuid::Uuid;

pub async fn get_orders() -> HashMap<Uuid, Order> {
    OrderManager::from_registry()
        .send(GetOrders {})
        .await
        .unwrap()
        .clone()
}

#[derive(Message)]
#[rtype(result = "HashMap<Uuid, Order>")]
pub struct GetOrders;

#[derive(Default)]
pub struct OrderManager {
    pub orders: HashMap<Uuid, Order>,
}

impl Actor for OrderManager {
    type Context = Context<Self>;
}

impl actix::Supervised for OrderManager {}

impl SystemService for OrderManager {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {
        debug!("OrderManager service started");
    }
}

impl Handler<GetOrders> for OrderManager {
    type Result = MessageResult<GetOrders>;

    fn handle(&mut self, _msg: GetOrders, _ctx: &mut Context<Self>) -> Self::Result {
        MessageResult(self.orders.clone())
    }
}

#[derive(Message)]
#[rtype(result = "Option<Order>")]
pub struct GetOrderByClientOrderId {
    pub client_order_id: String,
}

impl Handler<GetOrderByClientOrderId> for OrderManager {
    type Result = Option<Order>;

    fn handle(&mut self, msg: GetOrderByClientOrderId, _ctx: &mut Context<Self>) -> Self::Result {
        self.orders
            .values()
            .find(|order| order.client_order_id == msg.client_order_id)
            .cloned()
    }
}

#[derive(Message)]
#[rtype(result = "Option<Order>")]
pub struct GetOrderById {
    pub id: Uuid,
}

impl Handler<GetOrderById> for OrderManager {
    type Result = Option<Order>;

    fn handle(&mut self, msg: GetOrderById, _ctx: &mut Context<Self>) -> Self::Result {
        self.orders.get(&msg.id).cloned()
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PostOrder {
    pub order: Order,
}

impl Handler<PostOrder> for OrderManager {
    type Result = ();

    fn handle(&mut self, msg: PostOrder, _ctx: &mut Context<Self>) -> Self::Result {
        self.orders.insert(msg.order.id, msg.order);
    }
}

impl Handler<TradeFill> for OrderManager {
    type Result = ();

    fn handle(&mut self, msg: TradeFill, _ctx: &mut Context<Self>) -> Self::Result {
        self.orders.entry(msg.order.id).and_modify(|order| {
            let time = Some(msg.time);
            order.filled_qty = order.qty;
            order.updated_at = time;
            order.filled_at = time;
            order.filled_avg_price = Some(msg.price);
            order.status = OrderStatus::Filled;
        });
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CancelOrders;

impl Handler<CancelOrders> for OrderManager {
    type Result = ();

    fn handle(&mut self, _msg: CancelOrders, _ctx: &mut Context<Self>) -> Self::Result {
        for order in self.orders.values_mut() {
            match order.status {
                OrderStatus::Filled | OrderStatus::Expired | OrderStatus::Canceled => (),
                _ => order
                    .cancel()
                    .expect("All other statuses should be cancelable"),
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CancelOrder(pub Uuid);

impl Handler<CancelOrder> for OrderManager {
    type Result = ();

    fn handle(&mut self, msg: CancelOrder, _ctx: &mut Context<Self>) -> Self::Result {
        self.orders.get_mut(&msg.0).unwrap().cancel().unwrap();
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OrderType {
    Market,
    Limit {
        #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
        limit_price: f64,
    },
    Stop {
        #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
        stop_price: f64,
    },
    StopLimit {
        #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
        limit_price: f64,
        #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
        stop_price: f64,
    },
}

impl Default for OrderType {
    fn default() -> Self {
        OrderType::Market
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TimeInForce {
    DAY,
    GTC,
    OPG,
    CLS,
    IOC,
    FOK,
}

impl Default for TimeInForce {
    fn default() -> Self {
        TimeInForce::DAY
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Accepted,
    AcceptedForBidding,
    Calculated,
    Canceled,
    DoneForDay,
    Expired,
    Filled,
    Held,
    New,
    PartiallyFilled,
    PendingCancel,
    PendingNew,
    PendingReplace,
    Rejected,
    Replaced,
    Stopped,
    Suspended,
}

impl Default for OrderStatus {
    fn default() -> OrderStatus {
        OrderStatus::New
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Buy,
    Sell,
}

impl Default for Side {
    fn default() -> Side {
        Side::Buy
    }
}

impl Neg for Side {
    type Output = Side;

    fn neg(self) -> Self::Output {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TakeProfitSpec {
    pub limit_price: f64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct StopLossSpec {
    pub stop_price: f64,
    pub limit_price: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "order_class", rename_all = "lowercase")]
pub enum OrderClass {
    Simple,
    Bracket {
        take_profit: TakeProfitSpec,
        stop_loss: StopLossSpec,
    },
    OCO {
        take_profit: TakeProfitSpec,
        stop_loss: StopLossSpec,
    },
    OTO {
        take_profit: Option<TakeProfitSpec>,
        stop_loss: Option<StopLossSpec>,
    },
}

impl OrderClass {
    fn to_order(&self, oi: &OrderIntent, a: &Asset) -> Option<Vec<Order>> {
        match self {
            Self::Simple => None,
            Self::Bracket {
                take_profit: _,
                stop_loss: _,
            } => todo!(),
            Self::OCO {
                take_profit: _,
                stop_loss: _,
            } => todo!(),
            Self::OTO {
                take_profit,
                stop_loss,
            } => {
                let order_type = match (take_profit, stop_loss) {
                    (Some(TakeProfitSpec { limit_price }), None) => OrderType::Limit {
                        limit_price: *limit_price,
                    },
                    (
                        None,
                        Some(StopLossSpec {
                            stop_price,
                            limit_price,
                        }),
                    ) => match limit_price {
                        Some(limit_price) => OrderType::StopLimit {
                            stop_price: *stop_price,
                            limit_price: *limit_price,
                        },
                        None => OrderType::Stop {
                            stop_price: *stop_price,
                        },
                    },
                    _ => panic!(),
                };
                Some(vec![Order {
                    id: Uuid::new_v4(),
                    client_order_id: Uuid::new_v4().to_string(),
                    created_at: Some(Utc::now()),
                    submitted_at: Some(Utc::now()),
                    asset_id: a.id,
                    symbol: oi.symbol.clone(),
                    asset_class: a.class.clone(),
                    qty: oi.qty,
                    order_type,
                    side: oi.side.clone().neg(),
                    time_in_force: oi.time_in_force.clone(),
                    status: OrderStatus::Held,
                    ..Default::default()
                }])
            }
        }
    }
}

impl Default for OrderClass {
    fn default() -> Self {
        OrderClass::Simple
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct OrderIntent {
    pub symbol: String,
    #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
    pub qty: u32,
    pub side: Side,
    #[serde(flatten, rename(serialize = "type", deserialize = "type"))]
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,
    pub extended_hours: bool,
    pub client_order_id: Option<String>,
    #[serde(flatten)]
    pub order_class: OrderClass,
}

impl OrderIntent {
    pub fn new(symbol: &str) -> Self {
        OrderIntent {
            symbol: symbol.to_string(),
            ..Default::default()
        }
    }

    pub fn qty(mut self, qty: u32) -> Self {
        self.qty = qty;
        self
    }

    pub fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }

    pub fn order_type(mut self, order_type: OrderType) -> Self {
        self.order_type = order_type;
        self
    }

    pub fn time_in_force(mut self, time_in_force: TimeInForce) -> Self {
        self.time_in_force = time_in_force;
        self
    }

    pub fn extended_hours(mut self, extended_hours: bool) -> Self {
        self.extended_hours = extended_hours;
        self
    }

    pub fn client_order_id(mut self, client_order_id: &str) -> Self {
        self.client_order_id = Some(client_order_id.into());
        self
    }

    pub fn order_class(mut self, order_class: OrderClass) -> Self {
        self.order_class = order_class;
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Order {
    pub id: Uuid,
    pub client_order_id: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub filled_at: Option<DateTime<Utc>>,
    pub expired_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub replaced_at: Option<DateTime<Utc>>,
    pub replaced_by: Option<Uuid>,
    pub replaces: Option<Uuid>,
    pub asset_id: Uuid,
    pub symbol: String,
    pub asset_class: AssetClass,
    #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
    pub qty: u32,
    #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
    pub filled_qty: u32,
    #[serde(flatten, rename(serialize = "type"))]
    pub order_type: OrderType,
    pub side: Side,
    pub time_in_force: TimeInForce,
    #[serde(
        deserialize_with = "from_str_optional",
        serialize_with = "to_string_optional"
    )]
    pub filled_avg_price: Option<f64>,
    pub status: OrderStatus,
    pub extended_hours: bool,
    pub legs: Option<Vec<Order>>,
}

impl Order {
    //pub fn new() -> Self {
    //    Order {
    //        id: Uuid::new_v4(),
    //        client_order_id,
    //        created_at: Utc::now(),
    //        updated_at: None,
    //        submitted_at: None,
    //        filled_at: None,
    //        expired_at: None,
    //        canceled_at: None,
    //        failed_at: None,
    //        replaced_at: None,
    //        replaced_by: None,
    //        replaces: None,
    //        asset_id: a.id,
    //        symbol: oi.symbol.clone(),
    //        asset_class: a.class.clone(),
    //        qty: oi.qty,
    //        filled_qty: 0,
    //        order_type: oi.order_type.clone(),
    //        side: oi.side.clone(),
    //        time_in_force: oi.time_in_force.clone(),
    //        filled_avg_price: None,
    //        status: OrderStatus::New,
    //        extended_hours: oi.extended_hours,
    //        legs,
    //    }
    //}

    pub fn from_intent(oi: &OrderIntent, a: &Asset) -> Order {
        let client_order_id = match &oi.client_order_id {
            None => Uuid::new_v4().to_hyphenated().to_string(),
            Some(s) => s.into(),
        };
        let legs = oi.order_class.to_order(oi, a);

        Order {
            id: Uuid::new_v4(),
            client_order_id,
            created_at: Some(Utc::now()),
            asset_id: a.id,
            symbol: oi.symbol.clone(),
            asset_class: a.class.clone(),
            qty: oi.qty,
            order_type: oi.order_type.clone(),
            side: oi.side.clone(),
            time_in_force: oi.time_in_force.clone(),
            status: OrderStatus::New,
            extended_hours: oi.extended_hours,
            legs,
            ..Default::default()
        }
    }

    pub fn cancel(&mut self) -> Result<()> {
        match self.status {
            OrderStatus::Filled | OrderStatus::Expired | OrderStatus::Canceled => {
                Err(Error::Uncancelable)
            }
            _ => {
                let time = Some(Utc::now());
                self.status = OrderStatus::Canceled;
                self.canceled_at = time;
                self.updated_at = time;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;

    #[test]
    fn serde() {
        let json = r#"
	{
	  "id": "904837e3-3b76-47ec-b432-046db621571b",
	  "client_order_id": "904837e3-3b76-47ec-b432-046db621571b",
	  "created_at": "2018-10-05T05:48:59Z",
	  "updated_at": "2018-10-05T05:48:59Z",
	  "submitted_at": "2018-10-05T05:48:59Z",
	  "filled_at": "2018-10-05T05:48:59Z",
	  "expired_at": "2018-10-05T05:48:59Z",
	  "canceled_at": "2018-10-05T05:48:59Z",
	  "failed_at": "2018-10-05T05:48:59Z",
	  "replaced_at": "2018-10-05T05:48:59Z",
	  "replaced_by": "904837e3-3b76-47ec-b432-046db621571b",
	  "replaces": null,
	  "asset_id": "904837e3-3b76-47ec-b432-046db621571b",
	  "symbol": "AAPL",
	  "asset_class": "us_equity",
	  "qty": "15",
	  "filled_qty": "0",
	  "type": "market",
	  "side": "buy",
	  "time_in_force": "day",
	  "limit_price": "107.00",
	  "stop_price": "106.00",
	  "filled_avg_price": "106.00",
	  "status": "accepted",
	  "extended_hours": false,
	  "legs": null
	}
	"#;
        let deserialize: Order = serde_json::from_str(json).unwrap();
        let _serialize = serde_json::to_string(&deserialize).unwrap();
    }

    #[test]
    fn from_intent() {
        let a: Asset = Asset::from_symbol("TEST");
        let oi: OrderIntent = OrderIntent::new(&a.symbol);
        let o: Order = Order::from_intent(&oi, &a);
        assert_eq!(o.asset_id, a.id);
    }

    #[test]
    fn from_builder() {
        let a: Asset = Asset::from_symbol("TEST");
        let oi: OrderIntent = OrderIntent::new(&a.symbol)
            .qty(100)
            .side(Side::Sell)
            .order_type(OrderType::Limit { limit_price: 100.0 })
            .time_in_force(TimeInForce::FOK)
            .extended_hours(true)
            .client_order_id("TEST");
        let o: Order = Order::from_intent(&oi, &a);
        assert_eq!(o.qty, 100);
        assert_eq!(o.side, Side::Sell);
        assert_eq!(o.order_type, OrderType::Limit { limit_price: 100.0 });
        assert_eq!(o.time_in_force, TimeInForce::FOK);
        assert!(o.extended_hours);
        assert_eq!(&o.client_order_id, "TEST");
    }
}
