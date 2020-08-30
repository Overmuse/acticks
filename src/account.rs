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
