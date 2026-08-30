//! Business entities: plain structs with no ORM attributes, no `async` and no I/O.

pub mod asistencia_empleado;
pub mod audit;
pub mod categoria;
pub mod certificado;
pub mod cliente;
pub mod empleado;
pub mod factura;
pub mod feriado;
pub mod liquidacion;
pub mod movimiento;
pub mod obra;
pub mod orden_trabajo;
pub mod tipo_movimiento;
pub mod trabajo;

pub use asistencia_empleado::{AsistenciaEmpleado, ResumenAsistencia};
pub use audit::Audit;
pub use categoria::Categoria;
pub use certificado::{Certificado, CertificadoItem};
pub use cliente::{Cliente, ClienteContacto};
pub use empleado::Empleado;
pub use factura::{Factura, PagoFactura};
pub use feriado::{Feriado, OrigenFeriado};
pub use liquidacion::{Liquidacion, LiquidacionAdelanto, ReglasLiquidacion};
pub use movimiento::Movimiento;
pub use obra::Obra;
pub use orden_trabajo::{OrdenTrabajo, OrdenTrabajoItem};
pub use tipo_movimiento::TipoMovimiento;
pub use trabajo::Trabajo;
