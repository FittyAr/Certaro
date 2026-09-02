//! Business entities: plain structs with no ORM attributes, no `async` and no I/O.

pub mod adjunto;
pub mod asistencia_empleado;
pub mod audit;
pub mod auth;
pub mod calendario;
pub mod categoria;
pub mod certificado;
pub mod cliente;
pub mod empleado;
pub mod factura;
pub mod feriado;
pub mod kanban;
pub mod liquidacion;
pub mod movimiento;
pub mod proyecto;
pub mod orden_trabajo;
pub mod tipo_movimiento;
pub mod trabajo;

pub use adjunto::{Adjunto, EntidadAdjunto};
pub use asistencia_empleado::{AsistenciaEmpleado, ResumenAsistencia};
pub use audit::Audit;
pub use auth::{AuthProvider, AuthExterno, Permiso, Rol, RolPermiso, Sesion, Usuario, UsuarioRol};
pub use calendario::{
    CalendarioEvento, CalendarioEventoRecurso, CalendarioGrupoRecurso, CalendarioRecurso,
    TipoEvento, TipoRecurso,
};
pub use categoria::Categoria;
pub use certificado::{Certificado, CertificadoItem};
pub use cliente::{Cliente, ClienteContacto};
pub use empleado::Empleado;
pub use factura::{Factura, PagoFactura};
pub use feriado::{Feriado, OrigenFeriado};
pub use kanban::{
    KanbanColumna, KanbanEtiqueta, KanbanTablero, KanbanTarjeta, KanbanTarjetaAsignado,
    KanbanTarjetaChecklist, KanbanTarjetaEtiqueta, PrioridadTarjeta, TipoPresetTablero,
};
pub use liquidacion::{Liquidacion, LiquidacionAdelanto, ReglasLiquidacion};
pub use movimiento::Movimiento;
pub use proyecto::Proyecto;
pub use orden_trabajo::{OrdenTrabajo, OrdenTrabajoItem};
pub use tipo_movimiento::TipoMovimiento;
pub use trabajo::Trabajo;
