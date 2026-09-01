//! Dates and times. Everything is UTC. See `docs/04-dinero-fechas-y-tipos.md` §3.
//!
//! Two kinds of value live here and they are not interchangeable:
//!
//! - an **instant**, which keeps its time of day (`movimientos.fecha` is the only one);
//! - a **civil date**, which is a calendar day and is stored as UTC midnight of that day.
//!
//! Conflating them is what made the legacy system include or exclude movements at the edge of a
//! month depending on the hour they were loaded.

use crate::error::DomainError;
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;

/// Storage format: ISO-8601 UTC with milliseconds and a `Z` suffix, fixed 24 characters, so that
/// lexicographic order matches chronological order and `ORDER BY fecha` works.
pub const STORAGE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.3fZ";

/// Civil dates on the wire: `YYYY-MM-DD`, no time, no zone.
pub const CIVIL_FORMAT: &str = "%Y-%m-%d";

/// A calendar day as the UTC instant of its midnight.
#[must_use]
pub fn civil_to_utc(d: NaiveDate) -> DateTime<Utc> {
    let midnight = d.and_hms_milli_opt(0, 0, 0, 0).unwrap_or_default();
    Utc.from_utc_datetime(&midnight)
}

/// The calendar day of an instant, with no timezone conversion.
///
/// Deliberately not timezone-aware: a civil date that was stored as UTC midnight must come back as
/// the same day, and converting it to a local zone would move it backwards by three hours and land
/// on the previous day.
#[must_use]
pub fn utc_to_civil(dt: DateTime<Utc>) -> NaiveDate {
    dt.date_naive()
}

/// Renders an instant in the storage format.
#[must_use]
pub fn to_storage(dt: DateTime<Utc>) -> String {
    dt.format(STORAGE_FORMAT).to_string()
}

/// Parses the storage format, tolerating a missing fractional part.
pub fn from_storage(s: &str) -> Result<DateTime<Utc>, DomainError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")
                .map(|naive| Utc.from_utc_datetime(&naive))
        })
        .map_err(|_| DomainError::InvalidDate)
}

/// Parses a civil date from the `YYYY-MM-DD` the frontend sends.
pub fn parse_civil(s: &str) -> Result<NaiveDate, DomainError> {
    NaiveDate::parse_from_str(s.trim(), CIVIL_FORMAT).map_err(|_| DomainError::InvalidDate)
}

/// Parses an instant the user typed, which arrives with an offset or already in UTC.
pub fn parse_instant(s: &str) -> Result<DateTime<Utc>, DomainError> {
    from_storage(s)
}

/// Start of a day-range filter: inclusive, at `00:00:00.000Z`.
#[must_use]
pub fn range_start(d: NaiveDate) -> DateTime<Utc> {
    civil_to_utc(d)
}

/// End of a day-range filter: inclusive, at `23:59:59.999Z`.
///
/// Both ends are inclusive on purpose. Using `BETWEEN` with the next day's midnight, the obvious
/// alternative, silently includes anything that happened exactly at that midnight.
#[must_use]
pub fn range_end(d: NaiveDate) -> DateTime<Utc> {
    let end = d.and_hms_milli_opt(23, 59, 59, 999).unwrap_or_default();
    Utc.from_utc_datetime(&end)
}

/// Interprets a naive local date-time in `tz` and normalises it to UTC.
///
/// Ambiguous and non-existent local times (daylight-saving transitions) resolve to the earliest
/// valid instant rather than failing: refusing to save a movement because of a clock change would
/// be worse than a one-hour discrepancy on two days a year.
pub fn local_to_utc(naive: NaiveDateTime, tz: &Tz) -> DateTime<Utc> {
    match tz.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => dt.with_timezone(&Utc),
        chrono::LocalResult::Ambiguous(earliest, _) => earliest.with_timezone(&Utc),
        chrono::LocalResult::None => {
            let shifted = naive + chrono::Duration::hours(1);
            tz.from_local_datetime(&shifted).earliest().map_or_else(
                || Utc.from_utc_datetime(&naive),
                |dt| dt.with_timezone(&Utc),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_round_trip_keeps_the_day() {
        let d = NaiveDate::from_ymd_opt(2026, 3, 15).expect("valid date");
        assert_eq!(utc_to_civil(civil_to_utc(d)), d);
    }

    #[test]
    fn civil_to_utc_is_midnight() {
        let d = NaiveDate::from_ymd_opt(2026, 3, 15).expect("valid date");
        assert_eq!(to_storage(civil_to_utc(d)), "2026-03-15T00:00:00.000Z");
    }

    #[test]
    fn range_end_is_the_last_millisecond() {
        let d = NaiveDate::from_ymd_opt(2026, 8, 31).expect("valid date");
        assert_eq!(to_storage(range_end(d)), "2026-08-31T23:59:59.999Z");
    }

    #[test]
    fn storage_round_trip() {
        let raw = "2026-08-29T15:04:05.123Z";
        assert_eq!(to_storage(from_storage(raw).expect("parses")), raw);
    }

    #[test]
    fn storage_accepts_an_offset_and_normalises_it() {
        let dt = from_storage("2026-08-29T22:00:00.000-03:00").expect("parses");
        assert_eq!(to_storage(dt), "2026-08-30T01:00:00.000Z");
    }
}
