use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AssetClass {
    UsEquity,
}

impl Default for AssetClass {
    fn default() -> Self {
        Self::UsEquity
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Exchange {
    AMEX,
    ARCA,
    BATS,
    NYSE,
    NASDAQ,
    NYSEARCA,
    OTC,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Inactive,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Asset {
    pub id: Uuid,
    pub class: AssetClass,
    pub exchange: Exchange,
    pub symbol: String,
    pub status: Status,
    pub tradable: bool,
    pub marginable: bool,
    pub shortable: bool,
    pub easy_to_borrow: bool,
}

impl Asset {
    pub fn from_symbol(symbol: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            class: AssetClass::UsEquity,
            exchange: Exchange::NYSE,
            symbol: symbol.into(),
            status: Status::Active,
            tradable: true,
            marginable: true,
            shortable: true,
            easy_to_borrow: true,
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
	    "class": "us_equity",
	    "exchange": "NASDAQ",
	    "symbol": "AAPL",
	    "status": "active",
	    "tradable": true,
	    "marginable": true,
	    "shortable": true,
	    "easy_to_borrow": true
	}"#;
        let deserialized: Asset = serde_json::from_str(json).unwrap();
        let _serialized = serde_json::to_string(&deserialized).unwrap();
    }
}
