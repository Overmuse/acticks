use crate::account::{
    actors::{AccountManager, GetAccount},
    types::Account,
};
use crate::errors::{Error, Result};
use actix::registry::SystemService;
use log::trace;

pub mod actors;
pub mod types;

pub async fn get_account() -> Result<Account> {
    trace!("Sending GetAccount to AccountManager");
    AccountManager::from_registry()
        .send(GetAccount {})
        .await
        .map_err(|e| Error::from(e))
}
