//! Data transfer objects: the shape of the Tauri contract. See `docs/11-contratos-tauri.md`.

pub mod adjuntos;
pub mod asistencias;
pub mod categorias;
pub mod certificados;
pub mod clientes;
pub mod comercial;
pub mod common;
pub mod cotizaciones;
pub mod dashboard;
pub mod empleados;
pub mod facturas;
pub mod feriados;
pub mod liquidaciones;
pub mod movimientos;
pub mod obras;
pub mod ordenes_trabajo;
pub mod reportes;
pub mod tipos_movimiento;
pub mod trabajos;

pub use common::{AuditDto, EstadoInfo, ListQuery, LookupItem, TransicionPermitida};
