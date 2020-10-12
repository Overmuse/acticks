use crate::account::types::Account;
use crate::errors::Result;
use crate::exchange::TradeFill;
use crate::position::{self, actors::{PositionManager, GetPositionBySymbol}};
use actix::dev::{MessageResponse, ResponseChannel};
use actix::prelude::*;
use tracing::{debug, trace};

pub struct AccountManager {
    pub account: Account,
}

impl Default for AccountManager {
    fn default() -> Self {
        Self {
            account: Account::new(100000.0),
        }
    }
}

impl Actor for AccountManager {
    type Context = Context<Self>;
}

impl actix::Supervised for AccountManager {}

impl SystemService for AccountManager {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {
        debug!("AccountManager service started");
    }
}

#[derive(Message)]
#[rtype(result = "Account")]
pub struct GetAccount;

impl Handler<GetAccount> for AccountManager {
    type Result = Account;

    fn handle(&mut self, _msg: GetAccount, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Received GetAccount");
        self.account.clone()
    }
}

impl<A, M> MessageResponse<A, M> for Account
where
    A: Actor,
    M: Message<Result = Account>,
{
    fn handle<R: ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
        if let Some(tx) = tx {
            tx.send(self)
        }
    }
}

impl Handler<TradeFill> for AccountManager {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, tf: TradeFill, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Received TradeFill");
        let cost_basis = tf.price * tf.qty as f64;
        self.account.cash -= cost_basis;
        self.account.initial_margin += 0.5 * cost_basis;
        self.account.daytrade_count += 1;
        self.account.daytrading_buying_power =
            (self.account.equity - self.account.initial_margin).max(0.0) * self.account.multiplier;
        self.account.regt_buying_power = self.account.buying_power / 2.;
        let fut = async move {
            let prev_position = PositionManager::from_registry().send(GetPositionBySymbol{ symbol: tf.order.symbol.clone() }).await.ok().unwrap();
            (tf, prev_position)
        }
        .into_actor(self)
            .map(move |(tf, prev_position), act, _ctx| {
                match prev_position {
                     // The implementation here is incorrect, need to update based on market value, not cost
                     // basis
                     Some(pos) => {
                         if let position::Side::Long = pos.side {
                             act.account.long_market_value += cost_basis
                         } else {
                             act.account.short_market_value += cost_basis
                         }
                     }
                     None => {
                         if tf.qty > 0 {
                             act.account.long_market_value += cost_basis
                         } else {
                             act.account.short_market_value += cost_basis
                         }
                     }
                 };
                Ok(())
            });
        Box::pin(fut)
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SetCash(pub f64);

impl Handler<SetCash> for AccountManager {
    type Result = ();

    fn handle(&mut self, cash: SetCash, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Received SetCash");
        debug!("Updating cash: {}", &cash.0);
        self.account = Account::new(cash.0);
    }
}
