//! Re-export of the domain identifier port. See `ports::clock` for why it is re-exported.

pub use certaro_domain::ids::{IdGenerator as IdGeneratorPort, SequenceIdGenerator, UuidV7Generator};
