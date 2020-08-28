use std::default::Default;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DtpbCheck {
    Both,
    Entry,
    Exit
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TradeConfirmEmail {
    All,
    None,
}

#[derive(Debug, Serialize, Clone)]
pub struct AccountConfig {
    dtpb_check: DtpbCheck,
    no_shorting: bool,
    suspend_trade: bool,
    trade_confirm_email: TradeConfirmEmail,
}

impl Default for AccountConfig {
    fn default() -> AccountConfig {
	AccountConfig {
	    dtpb_check: DtpbCheck::Both,
	    no_shorting: false,
            suspend_trade: false,
            trade_confirm_email: TradeConfirmEmail::None,
	}
    }
}
