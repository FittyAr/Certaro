//! Contract of the `asistencia` module. See `docs/11-contratos-tauri.md` §5.9.
//!
//! The grid is transported as days plus rows rather than as a sparse map: `celdas` always has the
//! same length as `dias`, so the frontend renders by index and never has to guess what a missing
//! key means.

use chrono::NaiveDate;
use certaro_domain::entities::{AsistenciaEmpleado, ResumenAsistencia};
use certaro_domain::{Decimal4, TipoJornada};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsistenciaGrillaQuery {
    pub desde: NaiveDate,
    pub hasta: NaiveDate,
    /// Empty means every active employee.
    #[serde(default)]
    pub empleado_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsistenciaUpsertInput {
    pub empleado_id: Uuid,
    pub fecha: NaiveDate,
    /// `None` clears the cell: the click cycle has to be able to come back to empty.
    pub tipo_jornada: Option<TipoJornada>,
    pub trabajo_id: Option<Uuid>,
    pub observaciones: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsistenciaRangoInput {
    pub empleado_id: Uuid,
    pub desde: NaiveDate,
    pub hasta: NaiveDate,
    pub tipo_jornada: TipoJornada,
    /// Skips Saturdays, Sundays and holidays.
    pub solo_dias_habiles: bool,
    pub trabajo_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsistenciaDia {
    pub fecha: NaiveDate,
    /// `1` is Monday, `7` is Sunday.
    pub dia_semana: u8,
    pub es_fin_de_semana: bool,
    pub es_feriado: bool,
    pub feriado_nombre: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsistenciaCelda {
    pub fecha: NaiveDate,
    /// `None` means there is no record for that day.
    pub tipo_jornada: Option<TipoJornada>,
    pub trabajo_id: Option<Uuid>,
    pub observaciones: Option<String>,
}

impl AsistenciaCelda {
    pub fn vacia(fecha: NaiveDate) -> Self {
        Self {
            fecha,
            tipo_jornada: None,
            trabajo_id: None,
            observaciones: None,
        }
    }
}

impl From<&AsistenciaEmpleado> for AsistenciaCelda {
    fn from(a: &AsistenciaEmpleado) -> Self {
        Self {
            fecha: a.fecha,
            tipo_jornada: Some(a.tipo_jornada),
            trabajo_id: a.trabajo_id,
            observaciones: a.observaciones.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsistenciaResumenDto {
    pub completas: u32,
    pub medias: u32,
    pub faltas: u32,
    pub faltas_justificadas: u32,
    pub feriados: u32,
    pub jornadas_equivalentes: Decimal4,
}

impl From<ResumenAsistencia> for AsistenciaResumenDto {
    fn from(r: ResumenAsistencia) -> Self {
        Self {
            completas: r.completas,
            medias: r.medias,
            faltas: r.faltas,
            faltas_justificadas: r.faltas_justificadas,
            feriados: r.feriados,
            jornadas_equivalentes: r.jornadas_equivalentes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsistenciaFila {
    pub empleado_id: Uuid,
    pub empleado_nombre: String,
    pub empleado_cargo: Option<String>,
    pub celdas: Vec<AsistenciaCelda>,
    pub resumen: AsistenciaResumenDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsistenciaGrilla {
    pub desde: NaiveDate,
    pub hasta: NaiveDate,
    pub dias: Vec<AsistenciaDia>,
    pub filas: Vec<AsistenciaFila>,
}
