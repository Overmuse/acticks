use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::asset::{AssetClass, Exchange};
use crate::utils::{from_str, to_string};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all(serialize = "lowercase"))]
pub enum Side {
    Long,
    Short,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position {
    pub asset_id: Uuid,
    pub symbol: String,
    pub exchange: Exchange,
    pub asset_class: AssetClass,
    #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
    pub avg_entry_price: f64,
    #[serde(deserialize_with = "from_str", serialize_with = "to_string")]
    pub qty: u32,
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

impl Position {
    pub fn update_with_price(&mut self, price: f64) {
        self.market_value = self.qty as f64 * price;
        self.current_price = price;
        self.change_today = self.current_price / self.lastday_price - 1.0;
        self.unrealized_pl = self.market_value - self.cost_basis;
        self.unrealized_plpc = self.unrealized_pl / self.cost_basis;
    }
}
