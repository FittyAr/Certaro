//! The clock is a port. See `docs/04-dinero-fechas-y-tipos.md` §3.6.
//!
//! No use case calls `Utc::now()` directly, so every test can pin the time and compare exact
//! results instead of asserting on a range.

use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;

pub trait Clock: Send + Sync {
    fn now_utc(&self) -> DateTime<Utc>;

    /// "Today" in the user's timezone. At 21:00 on 29 August in Argentina it is already the 30th
    /// in UTC, so this cannot be derived from `now_utc` without the zone.
    fn today_civil(&self, tz: &Tz) -> NaiveDate {
        self.now_utc().with_timezone(tz).date_naive()
    }
}

/// The real clock. The only implementation that reads the operating system.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// A clock frozen at a given instant, for tests.
#[derive(Debug, Clone, Copy)]
pub struct FixedClock(pub DateTime<Utc>);

impl Clock for FixedClock {
    fn now_utc(&self) -> DateTime<Utc> {
        self.0
    }
}
