//! End-to-end exercise of the personnel modules against a real database: employees, the attendance
//! grid with its click cycle, the holiday table and the settlement with its frozen advances.

#[path = "personal/common.rs"]
mod common;
#[path = "personal/empleados.rs"]
mod empleados;
#[path = "personal/asistencias.rs"]
mod asistencias;
#[path = "personal/feriados.rs"]
mod feriados;
#[path = "personal/liquidaciones.rs"]
mod liquidaciones;
