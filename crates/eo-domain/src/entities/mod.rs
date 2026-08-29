//! Business entities: plain structs with no ORM attributes, no `async` and no I/O.

pub mod audit;
pub mod categoria;
pub mod movimiento;
pub mod tipo_movimiento;

pub use audit::Audit;
pub use categoria::Categoria;
pub use movimiento::Movimiento;
pub use tipo_movimiento::TipoMovimiento;
