//! SeaORM models, one per table of `docs/03-modelo-de-datos.md` §3.
//!
//! These are a transcription of the schema and nothing else: an amount is the scaled `i64` the
//! column holds, a date is the ISO-8601 text it holds, and no field is interpreted here. Turning
//! those into `Money`, `Decimal4` and `DateTime<Utc>` is the mappers' job, so that a change in how
//! a value is stored never leaks into the domain.
//!
//! `Relation` is empty in most models. Navigating a foreign key is done by the repositories with
//! explicit queries rather than by SeaORM's relations, because every read has to filter
//! `is_deleted = 0` and an implicit join would silently skip that.

#![allow(clippy::panic)]

pub mod adjunto;
pub mod app_metadata;
pub mod asistencia_empleado;
pub mod auth_externo;
pub mod categoria;
pub mod certificado;
pub mod certificado_item;
pub mod cliente;
pub mod cliente_contacto;
pub mod empleado;
pub mod factura;
pub mod feriado;
pub mod liquidacion;
pub mod liquidacion_adelanto;
pub mod movimiento;
pub mod orden_trabajo;
pub mod orden_trabajo_item;
pub mod pago_factura;
pub mod permiso;
pub mod proyecto;
pub mod rol;
pub mod rol_permiso;
pub mod sesion;
pub mod tipo_concepto_pago;
pub mod tipo_movimiento;
pub mod trabajo;
pub mod usuario;
pub mod usuario_rol;
