use crate::market::TickerTrade;
use actix::prelude::*;
use bdays::calendars::us::USSettlement;
use bdays::HolidayCalendar;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

pub fn get_clock() -> Clock {
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

impl Clock {
    fn next_open(dt: DateTime<Utc>) -> DateTime<Utc> {
        let cal = USSettlement {};
        let next_bday = cal.to_bday(dt, true);
        next_bday.date().and_hms(9, 30, 0)
    }

    fn next_close(dt: DateTime<Utc>) -> DateTime<Utc> {
        let cal = USSettlement {};
        todo!()
    }
}

impl Actor for Clock {
    type Context = Context<Self>;
}

impl actix::Supervised for Clock {}

impl Handler<TickerTrade> for Clock {
    type Result = ();

    fn handle(&mut self, msg: TickerTrade, _ctx: &mut Context<Self>) {
        let naive = NaiveDateTime::from_timestamp(msg.1.timestamp, 0);
        self.timestamp = DateTime::from_utc(naive, Utc);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::prelude::*;

    #[test]
    fn next_bday() {
        let dt = Utc.ymd(2020, 9, 30).and_hms(12, 00, 00);
        let next_dt = Clock::next_open(dt);
        assert_eq!(dt.date().succ(), next_dt.date());
    }
}
