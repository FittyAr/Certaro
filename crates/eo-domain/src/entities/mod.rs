//! Business entities: plain structs with no ORM attributes, no `async` and no I/O.

pub mod audit;
pub mod tipo_movimiento;

pub use audit::Audit;
pub use tipo_movimiento::TipoMovimiento;
