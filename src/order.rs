use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::ops::Neg;
use uuid::Uuid;

use crate::asset::{Asset, AssetClass};
use crate::errors::{Error, Result};
use crate::utils::*;

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
    pub limit_price: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct StopLossSpec {
    pub stop_price: f32,
    pub limit_price: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
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
        stop_loss: StopLossSpec,
    },
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

    pub fn client_order_id(mut self, client_order_id: String) -> Self {
        self.client_order_id = Some(client_order_id);
        self
    }

    pub fn order_class(mut self, order_class: OrderClass) -> Self {
        self.order_class = order_class;
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    pub id: Uuid,
    pub client_order_id: String,
    pub created_at: DateTime<Utc>,
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
    pub fn from_intent(oi: &OrderIntent, a: &Asset) -> Order {
        let client_order_id = match &oi.client_order_id {
            None => Uuid::new_v4().to_hyphenated().to_string(),
            Some(s) => s.into(),
        };
        Order {
            id: Uuid::new_v4(),
            client_order_id,
            created_at: Utc::now(),
            updated_at: None,
            submitted_at: None,
            filled_at: None,
            expired_at: None,
            canceled_at: None,
            failed_at: None,
            replaced_at: None,
            replaced_by: None,
            replaces: None,
            asset_id: a.id,
            symbol: oi.symbol.clone(),
            asset_class: a.class.clone(),
            qty: oi.qty,
            filled_qty: 0,
            order_type: oi.order_type.clone(),
            side: oi.side.clone(),
            time_in_force: oi.time_in_force.clone(),
            filled_avg_price: None,
            status: OrderStatus::New,
            extended_hours: oi.extended_hours,
            legs: None,
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
}
