use actix::prelude::*;
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Default)]
pub struct AssetManager {
    pub assets: HashMap<String, Asset>,
}

impl Actor for AssetManager {
    type Context = Context<Self>;
}

impl actix::Supervised for AssetManager {}

impl SystemService for AssetManager {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {
        debug!("AssetManager service started");
    }
}

#[derive(Message)]
#[rtype(result = "HashMap<String, Asset>")]
pub struct GetAssets;

impl Handler<GetAssets> for AssetManager {
    type Result = MessageResult<GetAssets>;

    fn handle(&mut self, _msg: GetAssets, _ctx: &mut Context<Self>) -> Self::Result {
        MessageResult(self.assets.clone())
    }
}

#[derive(Message)]
#[rtype(result = "Option<Asset>")]
pub struct GetAssetBySymbol {
    pub symbol: String,
}

impl Handler<GetAssetBySymbol> for AssetManager {
    type Result = Option<Asset>;

    fn handle(&mut self, msg: GetAssetBySymbol, _ctx: &mut Context<Self>) -> Self::Result {
        self.assets.get(&msg.symbol).cloned()
    }
}

#[derive(Message)]
#[rtype(result = "Option<Asset>")]
pub struct GetAssetById {
    pub id: Uuid,
}

impl Handler<GetAssetById> for AssetManager {
    type Result = Option<Asset>;

    fn handle(&mut self, msg: GetAssetById, _ctx: &mut Context<Self>) -> Self::Result {
        self.assets
            .values()
            .find(|asset| asset.id == msg.id)
            .cloned()
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SetAssets {
    pub symbols: Vec<String>,
}

impl Handler<SetAssets> for AssetManager {
    type Result = ();

    fn handle(&mut self, msg: SetAssets, _ctx: &mut Context<Self>) -> Self::Result {
        let symbols = msg.symbols;
        let assets: Vec<Asset> = symbols.iter().map(|x| Asset::from_symbol(x)).collect();
        let mapping: HashMap<String, _> = symbols.iter().cloned().zip(assets).collect();
        self.assets = mapping;
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
