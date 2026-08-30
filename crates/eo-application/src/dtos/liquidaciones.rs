//! Contract of the `liquidaciones` module. See `docs/11-contratos-tauri.md` §5.10.

use chrono::{DateTime, NaiveDate, Utc};
use eo_domain::entities::{Liquidacion, LiquidacionAdelanto, ReglasLiquidacion};
use eo_domain::{Decimal4, Money};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dtos::common::AuditDto;
use crate::ports::repositories::{LiquidacionConRelaciones, LiquidacionFiltro};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidacionFiltroDto {
    pub empleado_id: Option<Uuid>,
    pub fecha_desde: Option<NaiveDate>,
    pub fecha_hasta: Option<NaiveDate>,
    #[serde(default)]
    pub solo_sin_pdf: bool,
}

impl From<LiquidacionFiltroDto> for LiquidacionFiltro {
    fn from(dto: LiquidacionFiltroDto) -> Self {
        Self {
            empleado_id: dto.empleado_id,
            fecha_desde: dto.fecha_desde,
            fecha_hasta: dto.fecha_hasta,
            solo_sin_pdf: dto.solo_sin_pdf,
        }
    }
}

/// Where the suggested days came from: the three branches of the algorithm, made visible so the
/// wizard can say why it is proposing a number. See `docs/06-casos-de-uso-y-formulas.md` §6.6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum OrigenLiquidacion {
    Manual,
    Asistencia,
    Calendario,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidacionSugerenciaQuery {
    pub empleado_ids: Vec<Uuid>,
    pub desde: NaiveDate,
    pub hasta: NaiveDate,
    /// Days typed by hand, per employee. A value here forces the manual branch.
    #[serde(default)]
    pub dias_manuales: std::collections::HashMap<Uuid, Decimal4>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidacionDesglose {
    pub jornadas_completas: Decimal4,
    pub jornadas_medias: Decimal4,
    pub faltas: u32,
    pub faltas_justificadas: u32,
    pub dias_sabado: Decimal4,
    pub dias_domingo: Decimal4,
    pub dias_feriado: Decimal4,
    pub multiplicador_sabado: Decimal4,
    pub multiplicador_domingo: Decimal4,
    pub multiplicador_feriado: Decimal4,
    /// How much of the gross comes from weekend and holiday multipliers, so the PDF can show it as
    /// its own line instead of one opaque total.
    pub recargos: Money,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidacionAdelantoSugerido {
    pub movimiento_id: Uuid,
    pub fecha: NaiveDate,
    pub concepto: String,
    pub monto: Money,
    /// Already consumed by another settlement (INV-05): shown struck out and not added.
    pub ya_descontado: bool,
    pub liquidacion_que_lo_desconto: Option<Uuid>,
    pub incluir: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidacionSugerencia {
    pub empleado_id: Uuid,
    pub empleado_nombre: String,
    pub desde: NaiveDate,
    pub hasta: NaiveDate,
    pub dias_trabajados: Decimal4,
    pub tarifa_aplicada: Money,
    pub total_bruto: Money,
    pub total_adelantos: Money,
    pub total_neto: Money,
    pub origen: OrigenLiquidacion,
    pub incluir_sabados: bool,
    pub incluir_domingos: bool,
    pub incluir_feriados: bool,
    pub desglose: LiquidacionDesglose,
    pub adelantos: Vec<LiquidacionAdelantoSugerido>,
    /// True when the period has no holiday at all, which usually means the calendar could not be
    /// synced. The wizard warns rather than silently underpaying.
    pub feriados_no_disponibles: bool,
}

/// One settlement of the batch, as the user confirmed it after editing the preview.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidacionInput {
    pub empleado_id: Uuid,
    pub fecha_inicio: NaiveDate,
    pub fecha_fin: NaiveDate,
    pub dias_trabajados: Decimal4,
    pub tarifa_aplicada: Money,
    pub incluir_sabados: bool,
    pub incluir_domingos: bool,
    pub incluir_feriados: bool,
    pub multiplicador_sabado: Decimal4,
    pub multiplicador_domingo: Decimal4,
    pub multiplicador_feriado: Decimal4,
    pub total_bruto: Money,
    pub total_adelantos: Money,
    pub observaciones: Option<String>,
    /// The advances the user chose to include; each one is frozen into its own row.
    #[serde(default)]
    pub adelantos: Vec<LiquidacionAdelantoInput>,
}

impl LiquidacionInput {
    pub fn reglas(&self) -> ReglasLiquidacion {
        ReglasLiquidacion {
            incluir_sabados: self.incluir_sabados,
            incluir_domingos: self.incluir_domingos,
            incluir_feriados: self.incluir_feriados,
            multiplicador_sabado: self.multiplicador_sabado,
            multiplicador_domingo: self.multiplicador_domingo,
            multiplicador_feriado: self.multiplicador_feriado,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidacionAdelantoInput {
    pub movimiento_id: Uuid,
    pub fecha: NaiveDate,
    pub concepto: String,
    pub monto: Money,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidacionBatchInput {
    pub dtos: Vec<LiquidacionInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidacionBatchResult {
    pub creadas: Vec<Uuid>,
}

/// Only the notes can be rewritten once a settlement exists; the amounts are frozen and, after the
/// PDF is handed over, refused outright.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidacionUpdateInput {
    pub dias_trabajados: Decimal4,
    pub tarifa_aplicada: Money,
    pub total_bruto: Money,
    pub total_adelantos: Money,
    pub observaciones: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidacionAdelantoDto {
    pub id: Uuid,
    pub movimiento_id: Uuid,
    pub fecha: NaiveDate,
    pub concepto: String,
    pub monto: Money,
}

impl From<&LiquidacionAdelanto> for LiquidacionAdelantoDto {
    fn from(a: &LiquidacionAdelanto) -> Self {
        Self {
            id: a.id,
            movimiento_id: a.movimiento_id,
            fecha: a.fecha,
            concepto: a.concepto.clone(),
            monto: a.monto,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidacionListItem {
    pub id: Uuid,
    pub empleado_id: Uuid,
    pub empleado_nombre: String,
    pub fecha_inicio: NaiveDate,
    pub fecha_fin: NaiveDate,
    pub dias_trabajados: Decimal4,
    pub total_bruto: Money,
    pub total_adelantos: Money,
    pub total_neto: Money,
    pub pdf_generado_at: Option<DateTime<Utc>>,
    pub row_version: String,
}

impl From<LiquidacionConRelaciones> for LiquidacionListItem {
    fn from(row: LiquidacionConRelaciones) -> Self {
        let neto = row.liquidacion.total_neto().unwrap_or(Money::ZERO);
        Self {
            id: row.liquidacion.id,
            empleado_id: row.liquidacion.empleado_id,
            empleado_nombre: row.empleado_nombre,
            fecha_inicio: row.liquidacion.fecha_inicio,
            fecha_fin: row.liquidacion.fecha_fin,
            dias_trabajados: row.liquidacion.dias_trabajados,
            total_bruto: row.liquidacion.total_bruto,
            total_adelantos: row.liquidacion.total_adelantos,
            total_neto: neto,
            pdf_generado_at: row.liquidacion.pdf_generado_at,
            row_version: row.liquidacion.audit.row_version.to_hex(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidacionDetalle {
    pub id: Uuid,
    pub empleado_id: Uuid,
    pub empleado_nombre: String,
    pub empleado_cargo: Option<String>,
    pub empleado_dni: Option<String>,
    pub fecha_inicio: NaiveDate,
    pub fecha_fin: NaiveDate,
    pub dias_trabajados: Decimal4,
    pub tarifa_aplicada: Money,
    pub incluir_sabados: bool,
    pub incluir_domingos: bool,
    pub incluir_feriados: bool,
    pub multiplicador_sabado: Decimal4,
    pub multiplicador_domingo: Decimal4,
    pub multiplicador_feriado: Decimal4,
    pub total_bruto: Money,
    pub total_adelantos: Money,
    pub total_neto: Money,
    pub observaciones: Option<String>,
    pub pdf_generado_at: Option<DateTime<Utc>>,
    /// False once the PDF was handed over: the amounts are what the employee received.
    pub admite_cambio_de_importes: bool,
    pub adelantos: Vec<LiquidacionAdelantoDto>,
    pub audit: AuditDto,
}

impl LiquidacionDetalle {
    pub fn build(row: &LiquidacionConRelaciones) -> Self {
        let l: &Liquidacion = &row.liquidacion;
        Self {
            id: l.id,
            empleado_id: l.empleado_id,
            empleado_nombre: row.empleado_nombre.clone(),
            empleado_cargo: row.empleado_cargo.clone(),
            empleado_dni: row.empleado_dni.clone(),
            fecha_inicio: l.fecha_inicio,
            fecha_fin: l.fecha_fin,
            dias_trabajados: l.dias_trabajados,
            tarifa_aplicada: l.tarifa_aplicada,
            incluir_sabados: l.reglas.incluir_sabados,
            incluir_domingos: l.reglas.incluir_domingos,
            incluir_feriados: l.reglas.incluir_feriados,
            multiplicador_sabado: l.reglas.multiplicador_sabado,
            multiplicador_domingo: l.reglas.multiplicador_domingo,
            multiplicador_feriado: l.reglas.multiplicador_feriado,
            total_bruto: l.total_bruto,
            total_adelantos: l.total_adelantos,
            total_neto: l.total_neto().unwrap_or(Money::ZERO),
            observaciones: l.observaciones.clone(),
            pdf_generado_at: l.pdf_generado_at,
            admite_cambio_de_importes: l.admite_cambio_de_importes(),
            adelantos: l.adelantos.iter().map(Into::into).collect(),
            audit: AuditDto::from(&l.audit),
        }
    }
}
