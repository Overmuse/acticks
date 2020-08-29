use crate::account_configurations::AccountConfig;
use crate::credentials::Credentials;
use crate::position::Position;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
pub struct Account {
    id: Uuid,
    creds: Credentials,
    cash: f64,
    positions: Vec<Position>,
    config: AccountConfig,
}

impl Account {
    pub fn from_creds(creds: Credentials) -> Self {
        Account {
            id: Uuid::new_v4(),
            creds: creds,
            cash: 0.0,
            positions: vec![],
            config: AccountConfig::default(),
        }
    }

    pub fn get_positions(&self) -> Vec<Position> {
        self.positions.clone()
    }

}
