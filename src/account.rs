use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::{from_str, to_string};

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
    id: Uuid,
    account_number: String,
    status: AccountStatus,
    currency: String,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    cash: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    portfolio_value: f64,
    pattern_day_trader: bool,
    trade_suspended_by_user: bool,
    trading_blocked: bool,
    transfers_blocked: bool,
    account_blocked: bool,
    created_at: DateTime<Utc>,
    shorting_enabled: bool,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    long_market_value: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    short_market_value: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    equity: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    last_equity: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    multiplier: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    buying_power: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    initial_margin: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    maintenance_margin: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    sma: f64,
    daytrade_count: u32,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    last_maintenance_margin: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    daytrading_buying_power: f64,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    regt_buying_power: f64,
}

impl Account {
	pub fn new(cash: f64) -> Self {
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
			shorting_enabled: true,
			long_market_value: 0.0,
			short_market_value: 0.0,
			equity: cash,
			last_equity: cash,
			multiplier: 4.0,
			buying_power: 4.0 * cash,
			initial_margin: 0.0,
			maintenance_margin: 0.0,
			sma: 0.0,
			daytrade_count: 0,
			last_maintenance_margin: 0.0,
			daytrading_buying_power: 4.0 * cash,
			regt_buying_power: 2.0 * cash,
		}
	}
}

//---
//use crate::account_configurations::AccountConfig;
//use crate::credentials::Credentials;
//use crate::position::Position;
//use serde::Serialize;
//use uuid::Uuid;
//
//#[derive(Debug, Serialize, Clone)]
//pub struct Account {
//    id: Uuid,
//    creds: Credentials,
//    cash: f64,
//    positions: Vec<Position>,
//    config: AccountConfig,
//}
//
//impl Account {
//    pub fn from_creds(creds: Credentials) -> Self {
//        Account {
//            id: Uuid::new_v4(),
//            creds: creds,
//            cash: 0.0,
//            positions: vec![],
//            config: AccountConfig::default(),
//        }
//    }
//
//    pub fn get_positions(&self) -> Vec<Position> {
//        self.positions.clone()
//    }
//
//}
