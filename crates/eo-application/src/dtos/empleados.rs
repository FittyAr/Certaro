//! Contract of the `empleados` module. See `docs/11-contratos-tauri.md` §5.9.

use chrono::NaiveDate;
use eo_domain::entities::Empleado;
use eo_domain::{Decimal4, FrecuenciaPago, Money};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dtos::common::AuditDto;
use crate::ports::repositories::EmpleadoFiltro;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmpleadoFiltroDto {
    pub texto: Option<String>,
    /// Absent means every employee; the list screen sends `true`, because a payroll usually cares
    /// about who is still working.
    pub activo: Option<bool>,
    pub cargo: Option<String>,
}

impl Default for EmpleadoFiltroDto {
    fn default() -> Self {
        Self {
            texto: None,
            activo: Some(true),
            cargo: None,
        }
    }
}

impl From<EmpleadoFiltroDto> for EmpleadoFiltro {
    fn from(dto: EmpleadoFiltroDto) -> Self {
        Self {
            texto: dto.texto.filter(|t| !t.trim().is_empty()),
            activo: dto.activo,
            cargo: dto.cargo.filter(|c| !c.trim().is_empty()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmpleadoInput {
    pub nombre: String,
    pub dni: Option<String>,
    pub cargo: Option<String>,
    pub sueldo_base: Money,
    pub pago_frecuencia: FrecuenciaPago,
    pub tarifa_diaria: Money,
    pub multiplicador_sabado: Decimal4,
    pub multiplicador_domingo: Decimal4,
    pub multiplicador_feriado: Decimal4,
    pub email: Option<String>,
    pub telefono: Option<String>,
    pub fecha_ingreso: NaiveDate,
    pub fecha_egreso: Option<NaiveDate>,
    pub activo: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmpleadoListItem {
    pub id: Uuid,
    pub nombre: String,
    pub dni: Option<String>,
    pub cargo: Option<String>,
    pub tarifa_diaria: Money,
    pub sueldo_base: Money,
    pub pago_frecuencia: FrecuenciaPago,
    pub email: Option<String>,
    pub telefono: Option<String>,
    pub fecha_ingreso: NaiveDate,
    pub fecha_egreso: Option<NaiveDate>,
    pub activo: bool,
    pub row_version: String,
}

impl From<Empleado> for EmpleadoListItem {
    fn from(e: Empleado) -> Self {
        Self {
            id: e.id,
            nombre: e.nombre,
            dni: e.dni,
            cargo: e.cargo,
            tarifa_diaria: e.tarifa_diaria,
            sueldo_base: e.sueldo_base,
            pago_frecuencia: e.pago_frecuencia,
            email: e.email,
            telefono: e.telefono,
            fecha_ingreso: e.fecha_ingreso,
            fecha_egreso: e.fecha_egreso,
            activo: e.activo,
            row_version: e.audit.row_version.to_hex(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmpleadoDetalle {
    pub id: Uuid,
    pub nombre: String,
    pub dni: Option<String>,
    pub cargo: Option<String>,
    pub sueldo_base: Money,
    pub pago_frecuencia: FrecuenciaPago,
    pub tarifa_diaria: Money,
    /// What the rate would be if derived from the salary, so the form can offer it.
    pub tarifa_diaria_sugerida: Money,
    pub multiplicador_sabado: Decimal4,
    pub multiplicador_domingo: Decimal4,
    pub multiplicador_feriado: Decimal4,
    pub email: Option<String>,
    pub telefono: Option<String>,
    pub fecha_ingreso: NaiveDate,
    pub fecha_egreso: Option<NaiveDate>,
    pub activo: bool,
    pub puede_eliminarse: bool,
    pub audit: AuditDto,
}

impl EmpleadoDetalle {
    pub fn build(e: &Empleado, puede_eliminarse: bool) -> Self {
        Self {
            id: e.id,
            nombre: e.nombre.clone(),
            dni: e.dni.clone(),
            cargo: e.cargo.clone(),
            sueldo_base: e.sueldo_base,
            pago_frecuencia: e.pago_frecuencia,
            tarifa_diaria: e.tarifa_diaria,
            tarifa_diaria_sugerida: e.tarifa_diaria_sugerida().unwrap_or(Money::ZERO),
            multiplicador_sabado: e.multiplicador_sabado,
            multiplicador_domingo: e.multiplicador_domingo,
            multiplicador_feriado: e.multiplicador_feriado,
            email: e.email.clone(),
            telefono: e.telefono.clone(),
            fecha_ingreso: e.fecha_ingreso,
            fecha_egreso: e.fecha_egreso,
            activo: e.activo,
            puede_eliminarse,
            audit: AuditDto::from(&e.audit),
        }
    }
}
