use crate::account::{
    actors::{AccountManager, GetAccount},
    types::Account,
};
use crate::errors::{Error, Result};
use actix::registry::SystemService;

pub mod actors;
pub mod types;

#[tracing::instrument]
pub async fn get_account() -> Result<Account> {
    AccountManager::from_registry()
        .send(GetAccount {})
        .await
        .map_err(|e| Error::from(e))
}
