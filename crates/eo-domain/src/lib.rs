//! Pure domain layer.
//!
//! This crate has no I/O: no database, no HTTP, no filesystem, no Tauri. Everything here is
//! deterministic and testable in isolation. See `docs/02-arquitectura.md` §2.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod clock;
pub mod constants;
pub mod decimal4;
pub mod entities;
pub mod enums;
pub mod error;
pub mod ids;
pub mod money;
pub mod row_version;
mod scaled;
pub mod state;
pub mod time;

pub use decimal4::Decimal4;
pub use entities::{
    Audit, Categoria, Cliente, ClienteContacto, Factura, Movimiento, Obra, PagoFactura,
    TipoMovimiento, Trabajo,
};
pub use enums::{EstadoFactura, EstadoObra, EstadoTrabajo, MedioPago, Moneda};
pub use error::DomainError;
pub use money::{Money, SCALE};
pub use row_version::RowVersion;
pub use state::{recalcular_estado_factura, StateMachine};
