use uuid::Uuid;
use serde::Serialize;

#[derive(Clone, Serialize, Debug)]
pub struct Credentials {
    pub key_id: Uuid,
    pub secret_key: Uuid,
}

impl Credentials {
    pub fn new() -> Self {
        Self {
            key_id: Uuid::new_v4(), 
            secret_key: Uuid::new_v4(),
        }
    }
}
