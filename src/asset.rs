use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AssetClass {
    UsEquity,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Exchange {
    AMEX,
    ARCA,
    BATS,
    NYSE,
    NASDAQ,
    NYSEARCA,
    OTC,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Inactive,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Asset {
    id: Uuid,
    class: AssetClass,
    exchange: Exchange,
    symbol: String,
    status: Status,
    tradable: bool,
    marginable: bool,
    shortable: bool,
    easy_to_borrow: bool,
}
