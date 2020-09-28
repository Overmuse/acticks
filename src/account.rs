use crate::exchange::TradeFill;
use crate::utils::{from_str, to_string};
use actix::dev::{MessageResponse, ResponseChannel};
use actix::prelude::*;
use chrono::{DateTime, Utc};
use log::debug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "Account")]
pub struct GetAccount;

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
    fn service_started(&mut self, ctx: &mut Context<Self>) {
        debug!("AccountManager service started");
    }
}

impl Handler<GetAccount> for AccountManager {
    type Result = Account;

    fn handle(&mut self, _msg: GetAccount, _ctx: &mut Context<Self>) -> Self::Result {
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
    type Result = ();

    fn handle(&mut self, tf: TradeFill, _ctx: &mut Context<Self>) -> Self::Result {
        println!("TRADEFILL!");
        let cost_basis = tf.price * tf.qty as f64;
        self.account.cash -= cost_basis;
        if tf.qty > 0 {
            self.account.long_market_value += cost_basis
        } else {
            self.account.short_market_value += cost_basis
        };
        self.account.initial_margin += 0.5 * cost_basis;
        self.account.daytrade_count += 1;
        self.account.daytrading_buying_power =
            (self.account.equity - self.account.initial_margin).max(0.0) * self.account.multiplier;
        self.account.regt_buying_power = self.account.buying_power / 2.;
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SetCash(pub f64);

impl Handler<SetCash> for AccountManager {
    type Result = ();

    fn handle(&mut self, cash: SetCash, _ctx: &mut Context<Self>) -> Self::Result {
        debug!("Updating cash: {}", &cash.0);
        self.account = Account::new(cash.0);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountStatus {
    Onboarding,
    SubmissionFailed,
    Submitted,
    AccountUpdate,
    ApprovalPending,
    Active,
    Rejected,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Account {
    pub id: Uuid,
    pub account_number: String,
    pub status: AccountStatus,
    pub currency: String,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub cash: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub portfolio_value: f64,
    pub pattern_day_trader: bool,
    pub trade_suspended_by_user: bool,
    pub trading_blocked: bool,
    pub transfers_blocked: bool,
    pub account_blocked: bool,
    pub created_at: DateTime<Utc>,
    pub shorting_enabled: bool,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub long_market_value: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub short_market_value: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub equity: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub last_equity: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub multiplier: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub buying_power: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub initial_margin: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub maintenance_margin: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub sma: f64,
    pub daytrade_count: u32,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub last_maintenance_margin: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub daytrading_buying_power: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub regt_buying_power: f64,
}

impl Account {
    pub fn new(cash: f64) -> Self {
        let (multiplier, daytrading_buying_power, regt_buying_power) = if cash < 2000.0 {
            (1.0, 0.0, cash)
        } else if cash < 25000.0 {
            (2.0, 0.0, 2.0 * cash)
        } else {
            (4.0, 4.0 * cash, 2.0 * cash)
        };

        Account {
            id: Uuid::new_v4(),
            account_number: "".to_string(),
            status: AccountStatus::Active,
            currency: "USD".to_string(),
            cash,
            portfolio_value: cash,
            pattern_day_trader: false,
            trade_suspended_by_user: false,
            trading_blocked: false,
            transfers_blocked: false,
            account_blocked: false,
            created_at: Utc::now(),
            shorting_enabled: cash >= 2000.0,
            long_market_value: 0.0,
            short_market_value: 0.0,
            equity: cash,
            last_equity: cash,
            multiplier,
            buying_power: multiplier * cash,
            initial_margin: 0.0,
            maintenance_margin: 0.0,
            sma: 0.0,
            daytrade_count: 0,
            last_maintenance_margin: 0.0,
            daytrading_buying_power,
            regt_buying_power,
        }
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
            "account_blocked": false,
            "account_number": "010203ABCD",
            "buying_power": "262113.632",
            "cash": "-23140.2",
            "created_at": "2019-06-12T22:47:07.99658Z",
            "currency": "USD",
            "daytrade_count": 0,
            "daytrading_buying_power": "262113.632",
            "equity": "103820.56",
            "id": "e6fe16f3-64a4-4921-8928-cadf02f92f98",
            "initial_margin": "63480.38",
            "last_equity": "103529.24",
            "last_maintenance_margin": "38000.832",
            "long_market_value": "126960.76",
            "maintenance_margin": "38088.228",
            "multiplier": "4",
            "pattern_day_trader": false,
            "portfolio_value": "103820.56",
            "regt_buying_power": "80680.36",
            "short_market_value": "0",
            "shorting_enabled": true,
            "sma": "0",
            "status": "ACTIVE",
            "trade_suspended_by_user": false,
            "trading_blocked": false,
            "transfers_blocked": false
        }
        "#;
        let deserialized: Account = serde_json::from_str(json).unwrap();
        let _serialized = serde_json::to_string(&deserialized).unwrap();
    }

    #[test]
    fn initialization() {
        let cash_account = Account::new(1000.0);
        assert_eq!(cash_account.multiplier, 1.0);
        assert!(!cash_account.shorting_enabled);
        assert_eq!(cash_account.daytrading_buying_power, 0.0);
        assert_eq!(cash_account.regt_buying_power, cash_account.cash);

        let low_equity_account = Account::new(20_000.0);
        assert_eq!(low_equity_account.multiplier, 2.0);
        assert!(low_equity_account.shorting_enabled);
        assert_eq!(low_equity_account.daytrading_buying_power, 0.0);
        assert_eq!(
            low_equity_account.regt_buying_power,
            2.0 * low_equity_account.cash
        );

        let high_equity_account = Account::new(200_000.0);
        assert_eq!(high_equity_account.multiplier, 4.0);
        assert!(high_equity_account.shorting_enabled);
        assert_eq!(
            high_equity_account.daytrading_buying_power,
            4.0 * high_equity_account.cash
        );
        assert_eq!(
            high_equity_account.regt_buying_power,
            2.0 * high_equity_account.cash
        );
    }
}
