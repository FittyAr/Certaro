use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use certaro_application::ports::repositories::{
    AdelantoCandidato, LiquidacionConRelaciones, LiquidacionFiltro, LiquidacionRepository, SortDir,
};
use certaro_application::{AppError, AppResult, PageRequest, PagedResult};
use certaro_domain::constants::tipos_movimiento;
use certaro_domain::entities::{Liquidacion, LiquidacionAdelanto};
use certaro_domain::{time, Money, RowVersion};
use sea_orm::sea_query::{Alias, Expr, Func, Query, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, FromQueryResult, JoinType, Order,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use uuid::Uuid;

use crate::persistence::mappers::{self, liquidacion as mapper};
use crate::persistence::models::liquidacion::{self as model, Column, Entity, Relation};
use crate::persistence::models::{empleado, liquidacion_adelanto, movimiento};

const ENTITY: &str = "Liquidacion";

pub struct SeaOrmLiquidacionRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmLiquidacionRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

fn alive() -> Condition {
    Condition::all().add(Column::IsDeleted.eq(false))
}

fn filtro_condition(filtro: &LiquidacionFiltro) -> Condition {
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
struct RowConEmpleado {
    id: String,
    empleado_id: String,
    fecha_inicio: String,
    fecha_fin: String,
    dias_trabajados: i64,
    tarifa_aplicada: i64,
    incluir_sabados: bool,
    incluir_domingos: bool,
    incluir_feriados: bool,
    multiplicador_sabado: i64,
    multiplicador_domingo: i64,
    multiplicador_feriado: i64,
    total_bruto: i64,
    total_adelantos: i64,
    observaciones: Option<String>,
    pdf_generado_at: Option<String>,
    created_at: String,
    updated_at: Option<String>,
    row_version: Vec<u8>,
    is_deleted: bool,
    deleted_at: Option<String>,
    empleado_nombre: String,
    empleado_cargo: Option<String>,
    empleado_dni: Option<String>,
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

fn con_empleado() -> sea_orm::Select<Entity> {
    Entity::find()
        .join(JoinType::InnerJoin, Relation::Empleado.def())
        .column_as(empleado::Column::Nombre, "empleado_nombre")
        .column_as(empleado::Column::Cargo, "empleado_cargo")
        .column_as(empleado::Column::Dni, "empleado_dni")
}

fn nombre_empleado_lower() -> SimpleExpr {
    Func::lower(Expr::col((empleado::Entity, empleado::Column::Nombre))).into()
}

fn desde(date: NaiveDate) -> String {
    time::to_storage(Utc.from_utc_datetime(&date.and_time(NaiveTime::MIN)))
}

fn hasta(date: NaiveDate) -> String {
    let fin = NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap_or(NaiveTime::MIN);
    time::to_storage(Utc.from_utc_datetime(&date.and_time(fin)))
}

/// `monto × cantidad` rescaled: the advance is the total of the movement, not its unit price.
fn total_movimiento_expr() -> SimpleExpr {
    Expr::col((movimiento::Entity, movimiento::Column::Monto))
        .mul(Expr::col((
            movimiento::Entity,
            movimiento::Column::Cantidad,
        )))
        .div(Expr::value(certaro_domain::SCALE))
}

/// The settlement that already consumed the movement, or null. This is what turns INV-05 from an
/// index error into something the wizard can show struck out.
fn liquidacion_del_adelanto_expr() -> SimpleExpr {
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
struct RowCandidato {
    id: String,
    fecha: String,
    concepto: String,
    total: i64,
    liquidacion_id: Option<String>,
}

#[async_trait]
impl LiquidacionRepository for SeaOrmLiquidacionRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Liquidacion>> {
        let found = Entity::find_by_id(id.to_string())
            .filter(alive())
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(mapper::to_domain).transpose()
    }

    async fn find_con_adelantos(&self, id: Uuid) -> AppResult<Option<Liquidacion>> {
        let Some(mut entity) = self.find_by_id(id).await? else {
            return Ok(None);
        };
        entity.adelantos = self.adelantos_de(id).await?;
        Ok(Some(entity))
    }

    async fn find_detalle(&self, id: Uuid) -> AppResult<Option<LiquidacionConRelaciones>> {
        let found = con_empleado()
            .filter(alive())
            .filter(Column::Id.eq(id.to_string()))
            .into_model::<RowConEmpleado>()
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(LiquidacionConRelaciones::try_from).transpose()
    }

    async fn search(
        &self,
        filtro: &LiquidacionFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<LiquidacionConRelaciones>> {
        let condition = filtro_condition(filtro);
        let order = match sort_dir {
            SortDir::Asc => Order::Asc,
            SortDir::Desc => Order::Desc,
        };

        let neto = Expr::col((Entity, Column::TotalBruto))
            .sub(Expr::col((Entity, Column::TotalAdelantos)));

        let mut query = con_empleado()
            .filter(condition.clone())
            .column_as(neto.clone(), "total_neto_calc");

        query = match sort_by {
            Some("empleadoNombre") => query.order_by(nombre_empleado_lower(), order),
            Some("fechaInicio") => query.order_by(Column::FechaInicio, order),
            Some("diasTrabajados") => query.order_by(Column::DiasTrabajados, order),
            Some("totalBruto") => query.order_by(Column::TotalBruto, order),
            Some("totalNeto") => query.order_by(Expr::col(Alias::new("total_neto_calc")), order),
            // Newest period first: a payroll is read from the last one settled.
            _ => query.order_by(Column::FechaFin, order),
        }
        .order_by_asc(Column::Id);

        let total = Entity::find()
            .filter(condition)
            .count(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if let Some(limit) = page.limit() {
            query = query.limit(limit).offset(page.offset());
        }

        let rows = query
            .into_model::<RowConEmpleado>()
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        let items = rows
            .into_iter()
            .map(LiquidacionConRelaciones::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PagedResult::new(items, total, page))
    }

    async fn periodo_solapado(
        &self,
        empleado_id: Uuid,
        desde: NaiveDate,
        hasta: NaiveDate,
        excluir: Option<Uuid>,
    ) -> AppResult<Option<Liquidacion>> {
        let mut query = Entity::find()
            .filter(alive())
            .filter(Column::EmpleadoId.eq(empleado_id.to_string()))
            // Two closed ranges overlap when each starts before the other ends.
            .filter(Column::FechaInicio.lte(mappers::civil_to_storage(hasta)))
            .filter(Column::FechaFin.gte(mappers::civil_to_storage(desde)));

        if let Some(id) = excluir {
            query = query.filter(Column::Id.ne(id.to_string()));
        }

        let found = query
            .order_by_asc(Column::FechaInicio)
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(mapper::to_domain).transpose()
    }

    async fn adelantos_candidatos(
        &self,
        empleado_id: Uuid,
        desde: NaiveDate,
        hasta: NaiveDate,
    ) -> AppResult<Vec<AdelantoCandidato>> {
        let rows = movimiento::Entity::find()
            .select_only()
            .column(movimiento::Column::Id)
            .column(movimiento::Column::Fecha)
            .column(movimiento::Column::Concepto)
            .column_as(total_movimiento_expr(), "total")
            .column_as(liquidacion_del_adelanto_expr(), "liquidacion_id")
            .filter(movimiento::Column::IsDeleted.eq(false))
            .filter(movimiento::Column::EmpleadoId.eq(empleado_id.to_string()))
            // Filtered by the seeded identifier, never by the name: renaming the row must not stop
            // the payroll from finding advances.
            .filter(movimiento::Column::TipoMovimientoId.eq(tipos_movimiento::ADELANTO.to_string()))
            .filter(movimiento::Column::Fecha.gte(self::desde(desde)))
            .filter(movimiento::Column::Fecha.lte(self::hasta(hasta)))
            .order_by_asc(movimiento::Column::Fecha)
            .into_model::<RowCandidato>()
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        rows.into_iter()
            .map(|row| {
                Ok(AdelantoCandidato {
                    movimiento_id: mappers::uuid(&row.id)?,
                    fecha: mappers::instant(&row.fecha)?.date_naive(),
                    concepto: row.concepto,
                    monto: Money::from_raw(row.total),
                    liquidacion_id: mappers::uuid_opt(row.liquidacion_id.as_deref())?,
                })
            })
            .collect()
    }

    async fn insert(&self, entity: &Liquidacion) -> AppResult<()> {
        Entity::insert(mapper::to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &Liquidacion, esperado: RowVersion) -> AppResult<()> {
        let result = Entity::update_many()
            .set(mapper::to_active(entity))
            .filter(Column::Id.eq(entity.id.to_string()))
            .filter(Column::RowVersion.eq(esperado.as_bytes().to_vec()))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if result.rows_affected == 0 {
            return Err(AppError::Concurrency { entity: ENTITY });
        }
        Ok(())
    }

    async fn insert_adelanto(&self, entity: &LiquidacionAdelanto) -> AppResult<()> {
        // The partial unique index on `movimiento_id` is the real guard of INV-05; the read the use
        // case does first only makes the failure legible.
        liquidacion_adelanto::Entity::insert(mapper::adelanto_to_active(entity))
            .exec(self.conn())
            .await
            .map_err(|e| {
                if es_violacion_de_unicidad(&e) {
                    AppError::Conflict {
                        code: "ADELANTO_YA_DESCONTADO",
                        message_key: "Validation.Liquidacion.AdelantoYaDescontado",
                        params: [
                            ("concepto".to_owned(), entity.concepto.clone()),
                            ("fecha".to_owned(), entity.fecha.to_string()),
                        ]
                        .into(),
                    }
                } else {
                    AppError::persistence(e)
                }
            })?;
        Ok(())
    }

    async fn adelantos_de(&self, liquidacion_id: Uuid) -> AppResult<Vec<LiquidacionAdelanto>> {
        let rows = liquidacion_adelanto::Entity::find()
            .filter(liquidacion_adelanto::Column::LiquidacionId.eq(liquidacion_id.to_string()))
            .filter(liquidacion_adelanto::Column::IsDeleted.eq(false))
            .order_by_asc(liquidacion_adelanto::Column::Fecha)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::adelanto_to_domain).collect()
    }

    async fn marcar_pdf_generado(&self, id: Uuid, at: DateTime<Utc>) -> AppResult<()> {
        Entity::update_many()
            .col_expr(Column::PdfGeneradoAt, Expr::value(time::to_storage(at)))
            .filter(Column::Id.eq(id.to_string()))
            .filter(Column::PdfGeneradoAt.is_null())
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn soft_delete(
        &self,
        id: Uuid,
        esperado: RowVersion,
        at: DateTime<Utc>,
    ) -> AppResult<()> {
        let result = Entity::update_many()
            .col_expr(Column::IsDeleted, Expr::value(true))
            .col_expr(Column::DeletedAt, Expr::value(time::to_storage(at)))
            .col_expr(Column::UpdatedAt, Expr::value(time::to_storage(at)))
            .col_expr(
                Column::RowVersion,
                Expr::value(esperado.next().as_bytes().to_vec()),
            )
            .filter(Column::Id.eq(id.to_string()))
            .filter(Column::RowVersion.eq(esperado.as_bytes().to_vec()))
            .filter(Column::IsDeleted.eq(false))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if result.rows_affected == 0 {
            return Err(AppError::Concurrency { entity: ENTITY });
        }

        // Deleting the lines is what frees the advances: the unique index is partial on
        // `is_deleted = 0`, so they become available to settle again.
        liquidacion_adelanto::Entity::update_many()
            .col_expr(liquidacion_adelanto::Column::IsDeleted, Expr::value(true))
            .col_expr(
                liquidacion_adelanto::Column::DeletedAt,
                Expr::value(time::to_storage(at)),
            )
            .col_expr(
                liquidacion_adelanto::Column::UpdatedAt,
                Expr::value(time::to_storage(at)),
            )
            .filter(liquidacion_adelanto::Column::LiquidacionId.eq(id.to_string()))
            .filter(liquidacion_adelanto::Column::IsDeleted.eq(false))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;

        Ok(())
    }
}

fn es_violacion_de_unicidad(error: &sea_orm::DbErr) -> bool {
    let texto = error.to_string().to_lowercase();
    texto.contains("unique") || texto.contains("2067")
}
