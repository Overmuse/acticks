use crate::account::{
    actors::{AccountManager, GetAccount},
    types::Account,
};
use actix::registry::SystemService;

pub mod actors;
pub mod types;

pub async fn get_account() -> Account {
    AccountManager::from_registry()
        .send(GetAccount {})
        .await
        .unwrap()
}
