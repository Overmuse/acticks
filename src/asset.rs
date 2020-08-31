use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AssetClass {
    UsEquity,
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
