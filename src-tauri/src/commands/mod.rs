//! Tauri commands: one thin function per use case. No business logic, no calculation.
//!
//! See `docs/11-contratos-tauri.md`.
//!
//! `generate_handler!` needs the module path of each command, not a re-export: the macro also
//! looks up hidden items that `pub use` does not carry along.

pub mod app;
