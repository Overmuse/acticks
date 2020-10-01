use super::types::{Position, Side};
use crate::asset::actors::{AssetManager, GetAssetBySymbol};
use crate::exchange::TradeFill;
use crate::market::TickerTrade;
use actix::prelude::*;
use log::debug;
use std::collections::HashMap;

#[derive(Default)]
pub struct PositionManager {
    pub positions: HashMap<String, Position>,
}

impl Actor for PositionManager {
    type Context = Context<Self>;
}

impl actix::Supervised for PositionManager {}

impl SystemService for PositionManager {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {
        debug!("PositionManager service started");
    }
}

impl Handler<TickerTrade> for PositionManager {
    type Result = ();

    fn handle(&mut self, msg: TickerTrade, ctx: &mut Context<Self>) {
        let TickerTrade(ticker, trade) = msg;
        match self.positions.get_mut(&ticker) {
            Some(pos) => {
                pos.update_with_price(trade.price);
            }
            None => (),
        }
    }
}

#[derive(Message)]
#[rtype(result = "HashMap<String, Position>")]
pub struct GetPositions;

impl Handler<GetPositions> for PositionManager {
    type Result = MessageResult<GetPositions>;

    fn handle(&mut self, _msg: GetPositions, _ctx: &mut Context<Self>) -> Self::Result {
        MessageResult(self.positions.clone())
    }
}

#[derive(Message)]
#[rtype(result = "Option<Position>")]
pub struct GetPositionBySymbol {
    pub symbol: String,
}

impl Handler<GetPositionBySymbol> for PositionManager {
    type Result = Option<Position>;

    fn handle(&mut self, msg: GetPositionBySymbol, _ctx: &mut Context<Self>) -> Self::Result {
        self.positions.get(&msg.symbol).cloned()
    }
}

impl Handler<TradeFill> for PositionManager {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: TradeFill, _ctx: &mut Context<Self>) -> Self::Result {
        Box::pin(
            async move {
                let msg = msg.clone();
                let asset = AssetManager::from_registry()
                    .send(GetAssetBySymbol {
                        symbol: msg.order.symbol.clone(),
                    })
                    .await
                    .unwrap()
                    .unwrap();
                (msg, asset)
            }
            .into_actor(self)
            .map(|(msg, asset), act, _ctx| {
                act.positions
                    .entry(msg.order.symbol.clone())
                    .and_modify(|p| {
                        p.qty += msg.qty;
                        if p.qty >= 0 {
                            p.side = Side::Long
                        } else {
                            p.side = Side::Short
                        };
                        p.cost_basis += msg.qty as f64 * msg.price;
                        p.update_with_price(msg.price);
                    })
                    .or_insert(Position {
                        asset_id: asset.id,
                        symbol: msg.order.symbol.clone(),
                        exchange: asset.exchange,
                        asset_class: asset.class,
                        avg_entry_price: msg.price,
                        qty: msg.qty,
                        side: {
                            if msg.qty > 0 {
                                Side::Long
                            } else {
                                Side::Short
                            }
                        },
                        market_value: msg.qty as f64 * msg.price,
                        cost_basis: msg.qty as f64 * msg.price,
                        unrealized_pl: 0.0,
                        unrealized_plpc: 0.0,
                        unrealized_intraday_pl: 0.0,
                        unrealized_intraday_plpc: 0.0,
                        current_price: msg.price,
                        lastday_price: msg.price,
                        change_today: 0.0,
                    });
                act.positions.retain(|_, v| v.qty != 0);
            }),
        )
    }
}
