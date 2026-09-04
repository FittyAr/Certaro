use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use certaro_application::ports::repositories::{
    AdelantoCandidato, LiquidacionConRelaciones, LiquidacionFiltro, LiquidacionRepository, SortDir,
};
use certaro_application::{AppError, AppResult, PageRequest, PagedResult};
use certaro_domain::constants::tipos_movimiento;
use certaro_domain::entities::{Liquidacion, LiquidacionAdelanto};
use certaro_domain::{Money, RowVersion};
use sea_orm::sea_query::{Alias, Expr, ExprTrait};
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, Order, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::persistence::mappers::{self, liquidacion as mapper};
use crate::persistence::models::liquidacion::{Column, Entity};
use crate::persistence::models::{liquidacion_adelanto, movimiento};

mod mutation;
mod query;

use query::{
    alive, con_empleado, desde, filtro_condition, hasta, liquidacion_del_adelanto_expr,
    nombre_empleado_lower, total_movimiento_expr, RowCandidato, RowConEmpleado,
};

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
        desde_fecha: NaiveDate,
        hasta_fecha: NaiveDate,
        excluir: Option<Uuid>,
    ) -> AppResult<Option<Liquidacion>> {
        let mut query = Entity::find()
            .filter(alive())
            .filter(Column::EmpleadoId.eq(empleado_id.to_string()))
            // Two closed ranges overlap when each starts before the other ends.
            .filter(Column::FechaInicio.lte(mappers::civil_to_storage(hasta_fecha)))
            .filter(Column::FechaFin.gte(mappers::civil_to_storage(desde_fecha)));

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
        desde_fecha: NaiveDate,
        hasta_fecha: NaiveDate,
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
            .filter(movimiento::Column::Fecha.lte(hasta(hasta_fecha)))
            .filter(
                Condition::any()
                    .add(movimiento::Column::Fecha.gte(desde(desde_fecha)))
                    .add(liquidacion_del_adelanto_expr().is_null()),
            )
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
        mutation::insert(self.conn(), entity).await
    }

    async fn update(&self, entity: &Liquidacion, esperado: RowVersion) -> AppResult<()> {
        mutation::update(self.conn(), entity, esperado).await
    }

    async fn insert_adelanto(&self, entity: &LiquidacionAdelanto) -> AppResult<()> {
        mutation::insert_adelanto(self.conn(), entity).await
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
        mutation::marcar_pdf_generado(self.conn(), id, at).await
    }

    async fn soft_delete(
        &self,
        id: Uuid,
        esperado: RowVersion,
        at: DateTime<Utc>,
    ) -> AppResult<()> {
        mutation::soft_delete(self.conn(), id, esperado, at).await
    }
}
