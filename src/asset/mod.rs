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

pub async fn get_assets() -> HashMap<String, Asset> {
    AssetManager::from_registry()
        .send(GetAssets {})
        .await
        .unwrap()
        .clone()
}

pub async fn get_asset(symbol: &str) -> Result<Asset> {
    AssetManager::from_registry()
        .send(GetAssetBySymbol {
            symbol: symbol.to_string(),
        })
        .await
        .unwrap()
        .clone()
        .ok_or(Error::NotFound)
}

pub async fn get_asset_by_id(id: &Uuid) -> Result<Asset> {
    AssetManager::from_registry()
        .send(GetAssetById { id: *id })
        .await
        .unwrap()
        .clone()
        .ok_or(Error::NotFound)
}
