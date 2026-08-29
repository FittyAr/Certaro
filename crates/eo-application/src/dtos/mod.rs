//! Data transfer objects: the shape of the Tauri contract. See `docs/11-contratos-tauri.md`.

pub mod categorias;
pub mod clientes;
pub mod common;
pub mod facturas;
pub mod movimientos;
pub mod obras;
pub mod tipos_movimiento;
pub mod trabajos;

pub use common::{AuditDto, EstadoInfo, ListQuery, LookupItem, TransicionPermitida};
