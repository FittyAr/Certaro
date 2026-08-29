//! Business entities: plain structs with no ORM attributes, no `async` and no I/O.

pub mod audit;
pub mod categoria;
pub mod cliente;
pub mod factura;
pub mod movimiento;
pub mod obra;
pub mod tipo_movimiento;
pub mod trabajo;

pub use audit::Audit;
pub use categoria::Categoria;
pub use cliente::{Cliente, ClienteContacto};
pub use factura::{Factura, PagoFactura};
pub use movimiento::Movimiento;
pub use obra::Obra;
pub use tipo_movimiento::TipoMovimiento;
pub use trabajo::Trabajo;
