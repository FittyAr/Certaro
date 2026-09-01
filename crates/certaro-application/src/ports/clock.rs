//! Re-export of the domain clock port.
//!
//! The trait lives in `eo-domain` because domain code needs it too; the application layer names it
//! here so use cases import all their ports from one place.

pub use certaro_domain::clock::{Clock as ClockPort, FixedClock, SystemClock};
