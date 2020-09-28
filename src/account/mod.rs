use actix::registry::SystemService;
use crate::account::{actors::{AccountManager, GetAccount}, types::Account};

pub mod actors;
pub mod types;

pub async fn get_account() -> Account {
    AccountManager::from_registry()
        .send(GetAccount {})
        .await
        .unwrap()
        .clone()
}
