use crate::errors::{Error, Result};
use crate::order;
use actix::SystemService;
use actors::{GetPositionBySymbol, GetPositions, PositionManager};
use std::collections::HashMap;
pub use types::{Position, Side};

pub mod actors;
pub mod types;

pub async fn get_positions() -> Result<HashMap<String, Position>> {
    PositionManager::from_registry()
        .send(GetPositions {})
        .await
        .map_err(|e| Error::from(e))
}

pub async fn get_position(symbol: String) -> Result<Position> {
    PositionManager::from_registry()
        .send(GetPositionBySymbol { symbol })
        .await?
        .ok_or(Error::Other)
}

pub async fn close_positions() -> Result<()> {
    let positions = get_positions().await?;
    for position in positions.values() {
        let order_side = match position.side {
            Side::Long => order::types::Side::Sell,
            Side::Short => order::types::Side::Buy,
        };
        let order_intent = order::types::OrderIntent::new(&position.symbol)
            .qty(position.qty.abs() as u32)
            .side(order_side);
        order::post_order(order_intent).await?;
    }
    Ok(())
}

pub async fn close_position(symbol: String) -> Result<()> {
    let position = get_position(symbol.clone()).await?;
    let order_side = match position.side {
        Side::Long => order::types::Side::Sell,
        Side::Short => order::types::Side::Buy,
    };
    let order_intent = order::types::OrderIntent::new(&symbol)
        .qty(position.qty.abs() as u32)
        .side(order_side);
    order::post_order(order_intent).await?;
    Ok(())
}
