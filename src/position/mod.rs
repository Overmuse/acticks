use crate::errors::{Error, Result};
use actix::SystemService;
use actors::{GetPositionBySymbol, GetPositions, PositionManager};
use std::collections::HashMap;
use types::Position;

pub mod actors;
pub mod types;

pub async fn get_positions() -> HashMap<String, Position> {
    PositionManager::from_registry()
        .send(GetPositions {})
        .await
        .unwrap()
}

pub async fn get_position(symbol: String) -> Result<Position> {
    PositionManager::from_registry()
        .send(GetPositionBySymbol { symbol })
        .await
        .unwrap()
        .ok_or(Error::Other)
}
