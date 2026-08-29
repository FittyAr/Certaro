//! Data transfer objects: the shape of the Tauri contract. See `docs/11-contratos-tauri.md`.

pub mod categorias;
pub mod common;
pub mod movimientos;
pub mod tipos_movimiento;

pub use common::{AuditDto, ListQuery, LookupItem};
