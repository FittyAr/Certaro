//! End-to-end exercise of `movimientos`: the server-side filter, the summary over the whole
//! filter, the foreign-key checks and the freeze on a settled advance.

#[path = "movimientos/common.rs"]
mod common;
#[path = "movimientos/filtros.rs"]
mod filtros;
#[path = "movimientos/operaciones.rs"]
mod operaciones;
