//! Ports: the traits the application layer depends on. Infrastructure implements them.
//!
//! See `docs/02-arquitectura.md` §5.

pub mod clock;
pub mod id_generator;
pub mod settings;

pub use clock::ClockPort;
pub use id_generator::IdGeneratorPort;
pub use settings::SettingsStore;
