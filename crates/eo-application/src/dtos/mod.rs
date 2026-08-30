//! Data transfer objects: the shape of the Tauri contract. See `docs/11-contratos-tauri.md`.

pub mod asistencias;
pub mod categorias;
pub mod certificados;
pub mod clientes;
pub mod common;
pub mod empleados;
pub mod facturas;
pub mod feriados;
pub mod liquidaciones;
pub mod movimientos;
pub mod obras;
pub mod ordenes_trabajo;
pub mod tipos_movimiento;
pub mod trabajos;

pub use common::{AuditDto, EstadoInfo, ListQuery, LookupItem, TransicionPermitida};
