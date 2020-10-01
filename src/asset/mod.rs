use crate::asset::{
    actors::{AssetManager, GetAssetById, GetAssetBySymbol, GetAssets},
    types::Asset,
};
use crate::errors::{Error, Result};
use actix::SystemService;
use std::collections::HashMap;
use uuid::Uuid;

pub mod actors;
pub mod types;

pub async fn get_assets() -> Result<HashMap<String, Asset>> {
    AssetManager::from_registry()
        .send(GetAssets {})
        .await
        .map_err(|e| Error::from(e))
}

pub async fn get_asset(symbol: &str) -> Result<Asset> {
    AssetManager::from_registry()
        .send(GetAssetBySymbol {
            symbol: symbol.to_string(),
        })
        .await?
        .ok_or(Error::NotFound)
}

pub async fn get_asset_by_id(id: &Uuid) -> Result<Asset> {
    AssetManager::from_registry()
        .send(GetAssetById { id: *id })
        .await?
        .ok_or(Error::NotFound)
}
