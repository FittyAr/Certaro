use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use certaro_application::ports::repositories::{
    LiquidacionConRelaciones, LiquidacionFiltro,
};
use certaro_application::AppError;
use certaro_domain::time;
use sea_orm::sea_query::{Expr, Func, Query, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, FromQueryResult, JoinType, QuerySelect, RelationTrait,
};

use crate::persistence::mappers::{self, liquidacion as mapper};
use crate::persistence::models::liquidacion::{self as model, Column, Entity, Relation};
use crate::persistence::models::{empleado, liquidacion_adelanto, movimiento};

pub(super) fn alive() -> Condition {
    Condition::all().add(Column::IsDeleted.eq(false))
}

pub(super) fn filtro_condition(filtro: &LiquidacionFiltro) -> Condition {
    let mut c = alive();
    if let Some(id) = filtro.empleado_id {
        c = c.add(Column::EmpleadoId.eq(id.to_string()));
    }
    // Matched against the period, which is what the user has in mind, and not against `created_at`.
    if let Some(fecha) = filtro.fecha_desde {
        c = c.add(Column::FechaFin.gte(mappers::civil_to_storage(fecha)));
    }
    if let Some(fecha) = filtro.fecha_hasta {
        c = c.add(Column::FechaInicio.lte(mappers::civil_to_storage(fecha)));
    }
    if filtro.solo_sin_pdf {
        c = c.add(Column::PdfGeneradoAt.is_null());
    }
    c
}

#[derive(Debug, FromQueryResult)]
pub(super) struct RowConEmpleado {
    pub id: String,
    pub empleado_id: String,
    pub fecha_inicio: String,
    pub fecha_fin: String,
    pub dias_trabajados: i64,
    pub tarifa_aplicada: i64,
    pub incluir_sabados: bool,
    pub incluir_domingos: bool,
    pub incluir_feriados: bool,
    pub multiplicador_sabado: i64,
    pub multiplicador_domingo: i64,
    pub multiplicador_feriado: i64,
    pub total_bruto: i64,
    pub total_adelantos: i64,
    pub observaciones: Option<String>,
    pub pdf_generado_at: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
    pub empleado_nombre: String,
    pub empleado_cargo: Option<String>,
    pub empleado_dni: Option<String>,
}

impl TryFrom<RowConEmpleado> for LiquidacionConRelaciones {
    type Error = AppError;

    fn try_from(row: RowConEmpleado) -> Result<Self, Self::Error> {
        let model = model::Model {
            id: row.id,
            empleado_id: row.empleado_id,
            fecha_inicio: row.fecha_inicio,
            fecha_fin: row.fecha_fin,
            dias_trabajados: row.dias_trabajados,
            tarifa_aplicada: row.tarifa_aplicada,
            incluir_sabados: row.incluir_sabados,
            incluir_domingos: row.incluir_domingos,
            incluir_feriados: row.incluir_feriados,
            multiplicador_sabado: row.multiplicador_sabado,
            multiplicador_domingo: row.multiplicador_domingo,
            multiplicador_feriado: row.multiplicador_feriado,
            total_bruto: row.total_bruto,
            total_adelantos: row.total_adelantos,
            observaciones: row.observaciones,
            pdf_generado_at: row.pdf_generado_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            row_version: row.row_version,
            is_deleted: row.is_deleted,
            deleted_at: row.deleted_at,
        };
        Ok(Self {
            liquidacion: mapper::to_domain(model)?,
            empleado_nombre: row.empleado_nombre,
            empleado_cargo: row.empleado_cargo,
            empleado_dni: row.empleado_dni,
        })
    }
}

pub(super) fn con_empleado() -> sea_orm::Select<Entity> {
    Entity::find()
        .join(JoinType::InnerJoin, Relation::Empleado.def())
        .column_as(empleado::Column::Nombre, "empleado_nombre")
        .column_as(empleado::Column::Cargo, "empleado_cargo")
        .column_as(empleado::Column::Dni, "empleado_dni")
}

pub(super) fn nombre_empleado_lower() -> SimpleExpr {
    Func::lower(Expr::col((empleado::Entity, empleado::Column::Nombre))).into()
}

pub(super) fn desde(date: NaiveDate) -> String {
    time::to_storage(Utc.from_utc_datetime(&date.and_time(NaiveTime::MIN)))
}

pub(super) fn hasta(date: NaiveDate) -> String {
    let fin = NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap_or(NaiveTime::MIN);
    time::to_storage(Utc.from_utc_datetime(&date.and_time(fin)))
}

/// `monto × cantidad` rescaled: the advance is the total of the movement, not its unit price.
pub(super) fn total_movimiento_expr() -> SimpleExpr {
    Expr::col((movimiento::Entity, movimiento::Column::Monto))
        .mul(Expr::col((
            movimiento::Entity,
            movimiento::Column::Cantidad,
        )))
        .div(Expr::value(certaro_domain::SCALE))
}

/// The settlement that already consumed the movement, or null. This is what turns INV-05 from an
/// index error into something the wizard can show struck out.
pub(super) fn liquidacion_del_adelanto_expr() -> SimpleExpr {
    SimpleExpr::SubQuery(
        None,
        Box::new(
            Query::select()
                .expr(Expr::col((
                    liquidacion_adelanto::Entity,
                    liquidacion_adelanto::Column::LiquidacionId,
                )))
                .from(liquidacion_adelanto::Entity)
                .and_where(
                    Expr::col((
                        liquidacion_adelanto::Entity,
                        liquidacion_adelanto::Column::MovimientoId,
                    ))
                    .equals((movimiento::Entity, movimiento::Column::Id)),
                )
                .and_where(
                    Expr::col((
                        liquidacion_adelanto::Entity,
                        liquidacion_adelanto::Column::IsDeleted,
                    ))
                    .eq(false),
                )
                .limit(1)
                .take()
                .into_sub_query_statement(),
        ),
    )
}

#[derive(Debug, FromQueryResult)]
pub(super) struct RowCandidato {
    pub id: String,
    pub fecha: String,
    pub concepto: String,
    pub total: i64,
    pub liquidacion_id: Option<String>,
}
