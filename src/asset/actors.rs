use crate::asset::types::Asset;
use actix::prelude::*;
use std::collections::HashMap;
use tracing::debug;
use uuid::Uuid;

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

#[derive(Message, Debug)]
#[rtype(result = "HashMap<String, Asset>")]
pub struct GetAssets;

impl Handler<GetAssets> for AssetManager {
    type Result = MessageResult<GetAssets>;

    #[tracing::instrument(name = "AssetManager: Handle<GetAssets>", skip(self, _msg, _ctx))]
    fn handle(&mut self, _msg: GetAssets, _ctx: &mut Context<Self>) -> Self::Result {
        MessageResult(self.assets.clone())
    }
}

#[derive(Message, Debug)]
#[rtype(result = "Option<Asset>")]
pub struct GetAssetBySymbol {
    pub symbol: String,
}

impl Handler<GetAssetBySymbol> for AssetManager {
    type Result = Option<Asset>;

    #[tracing::instrument(name = "AssetManager: Handle<GetAssetBySymbol>", skip(self, _ctx))]
    fn handle(&mut self, msg: GetAssetBySymbol, _ctx: &mut Context<Self>) -> Self::Result {
        self.assets.get(&msg.symbol).cloned()
    }
}

#[derive(Message, Debug)]
#[rtype(result = "Option<Asset>")]
pub struct GetAssetById {
    pub id: Uuid,
}

impl Handler<GetAssetById> for AssetManager {
    type Result = Option<Asset>;

    #[tracing::instrument(name = "AssetManager: Handle<GetAssetById>", skip(self, _ctx))]
    fn handle(&mut self, msg: GetAssetById, _ctx: &mut Context<Self>) -> Self::Result {
        self.assets
            .values()
            .find(|asset| asset.id == msg.id)
            .cloned()
    }
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct SetAssets {
    pub symbols: Vec<String>,
}

impl Handler<SetAssets> for AssetManager {
    type Result = ();

    #[tracing::instrument(name = "AssetManager: Handle<SetAssets>", skip(self, _ctx))]
    fn handle(&mut self, msg: SetAssets, _ctx: &mut Context<Self>) -> Self::Result {
        let symbols = msg.symbols;
        let assets: Vec<Asset> = symbols.iter().map(|x| Asset::from_symbol(x)).collect();
        let mapping: HashMap<String, _> = symbols.iter().cloned().zip(assets).collect();
        self.assets = mapping;
    }
}
