use crate::asset::types::{AssetClass, Exchange};
use crate::utils::{from_str, to_string};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all(serialize = "lowercase", deserialize = "lowercase"))]
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

impl Position {
    pub fn update_with_price(&mut self, price: f64) {
        self.market_value = self.qty as f64 * price;
        self.current_price = price;
        self.change_today = (self.current_price - self.lastday_price) / self.lastday_price;
        self.unrealized_pl = self.market_value - self.cost_basis;
        self.unrealized_plpc = self.unrealized_pl / self.cost_basis;
        self.unrealized_intraday_pl = self.qty as f64 * (self.current_price - self.lastday_price);
        self.unrealized_intraday_plpc =
            self.unrealized_intraday_pl / (self.qty as f64 * self.lastday_price);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;
    use uuid::Uuid;
    #[test]
    fn update_position() {
        let mut pos = Position {
            asset_id: Uuid::new_v4(),
            symbol: "Test".into(),
            exchange: Exchange::NYSE,
            asset_class: AssetClass::UsEquity,
            avg_entry_price: 80.0,
            qty: 1,
            side: Side::Long,
            market_value: 100.0,
            cost_basis: 80.0,
            unrealized_pl: 20.0,
            unrealized_plpc: 0.25,
            unrealized_intraday_pl: 0.0,
            unrealized_intraday_plpc: 0.0,
            current_price: 100.0,
            lastday_price: 100.0,
            change_today: 0.0,
        };
        pos.update_with_price(105.0);
        assert!((pos.market_value - 105.0).abs() < 0.001);
        assert!((pos.current_price - 105.0).abs() < 0.001);
        assert!((pos.change_today - 0.05).abs() < 0.001);
        assert!((pos.unrealized_pl - 25.0).abs() < 0.001);
        assert!((pos.unrealized_plpc - 0.3125).abs() < 0.001);
        assert!((pos.unrealized_intraday_pl - 5.0).abs() < 0.001);
        assert!((pos.unrealized_intraday_plpc - 0.05).abs() < 0.001);
    }

    #[test]
    fn serde() {
        let json = r#"
			{
  				"asset_id": "904837e3-3b76-47ec-b432-046db621571b",
  				"symbol": "AAPL",
 	 			"exchange": "NASDAQ",
  				"asset_class": "us_equity",
  				"avg_entry_price": "100.0",
  				"qty": "5",
  				"side": "long",
  				"market_value": "600.0",
  				"cost_basis": "500.0",
  				"unrealized_pl": "100.0",
  				"unrealized_plpc": "0.20",
  				"unrealized_intraday_pl": "10.0",
  				"unrealized_intraday_plpc": "0.0084",
  				"current_price": "120.0",
  				"lastday_price": "119.0",
  				"change_today": "0.0084"
			}"#;
        let deserialized: Position = serde_json::from_str(json).unwrap();
        let _serialized = serde_json::to_string(&deserialized).unwrap();
    }
}
