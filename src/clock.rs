use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub fn get_clock() -> Clock {
    // TODO: Make this dynamically pull from exchange
    Clock {
        timestamp: Utc::now(),
        is_open: true,
        next_open: Utc::now(),
        next_close: Utc::now(),
    }
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Clock {
    pub timestamp: DateTime<Utc>,
    pub is_open: bool,
    pub next_open: DateTime<Utc>,
    pub next_close: DateTime<Utc>,
}
