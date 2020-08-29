use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::{from_str, to_string};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all(serialize = "lowercase"))]
pub enum Side {
    Long,
    Short
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position {
    pub asset_id: Uuid,
    pub symbol: String,
    pub exchange: String,
    pub asset_class: String,
    #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
    pub avg_entry_price: f64,
    #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
    pub qty: i32,
    pub side: Side,
    #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
    pub market_value: f64,
    #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
    pub cost_basis: f64,
    #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
    pub unrealized_pl: f64,
    #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
    pub unrealized_plpc: f64,
    #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
    pub unrealized_intraday_pl: f64,
    #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
    pub unrealized_intraday_plpc: f64,
    #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
    pub current_price: f64,
    #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
    pub lastday_price: f64,
    #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
    pub change_today: f64,
}
