use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use certaro_application::ports::repositories::{EmpleadoFiltro, EmpleadoRepository, SortDir};
use certaro_application::{AppError, AppResult, PageRequest, PagedResult};
use certaro_domain::entities::Empleado;
use certaro_domain::{time, RowVersion};
use sea_orm::sea_query::{Expr, Func, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, Order, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::persistence::mappers::empleado as mapper;
use crate::persistence::models::empleado::{Column, Entity};
use crate::persistence::models::{asistencia_empleado, liquidacion, movimiento};

const ENTITY: &str = "Empleado";

pub struct SeaOrmEmpleadoRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmEmpleadoRepository {
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

fn lower(column: Column) -> SimpleExpr {
    Func::lower(Expr::col((Entity, column))).into()
}

fn filtro_condition(filtro: &EmpleadoFiltro) -> Condition {
    let mut c = alive();
    if let Some(texto) = filtro.texto.as_deref() {
        let patron = format!("%{}%", texto.trim().to_lowercase());
        c = c.add(
            Condition::any()
                .add(lower(Column::Nombre).like(patron.clone()))
                .add(lower(Column::Dni).like(patron.clone()))
                .add(lower(Column::Cargo).like(patron)),
        );
    }
    if let Some(activo) = filtro.activo {
        c = c.add(Column::Activo.eq(activo));
    }
    if let Some(cargo) = filtro.cargo.as_deref() {
        c = c.add(Column::Cargo.eq(cargo));
    }
    c
}

#[async_trait]
impl EmpleadoRepository for SeaOrmEmpleadoRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Empleado>> {
        let found = Entity::find_by_id(id.to_string())
            .filter(alive())
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(mapper::to_domain).transpose()
    }

    async fn search(
        &self,
        filtro: &EmpleadoFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<Empleado>> {
        let condition = filtro_condition(filtro);
        let order = match sort_dir {
            SortDir::Asc => Order::Asc,
            SortDir::Desc => Order::Desc,
        };

        let mut query = Entity::find().filter(condition.clone());
        query = match sort_by {
            Some("cargo") => query.order_by(lower(Column::Cargo), order),
            Some("tarifaDiaria") => query.order_by(Column::TarifaDiaria, order),
            Some("sueldoBase") => query.order_by(Column::SueldoBase, order),
            Some("fechaIngreso") => query.order_by(Column::FechaIngreso, order),
            _ => query.order_by(lower(Column::Nombre), order),
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
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        let items = rows
            .into_iter()
            .map(mapper::to_domain)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PagedResult::new(items, total, page))
    }

    async fn lookup(
        &self,
        texto: Option<&str>,
        solo_activos: bool,
        limite: u64,
    ) -> AppResult<Vec<Empleado>> {
        let filtro = EmpleadoFiltro {
            texto: texto.map(str::to_owned),
            activo: solo_activos.then_some(true),
            cargo: None,
        };
        let rows = Entity::find()
            .filter(filtro_condition(&filtro))
            .order_by_asc(lower(Column::Nombre))
            .limit(limite)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::to_domain).collect()
    }

    async fn activos(&self) -> AppResult<Vec<Empleado>> {
        let rows = Entity::find()
            .filter(alive())
            .filter(Column::Activo.eq(true))
            .order_by_asc(lower(Column::Nombre))
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::to_domain).collect()
    }

    async fn cargos(&self) -> AppResult<Vec<String>> {
        let rows: Vec<Option<String>> = Entity::find()
            .select_only()
            .column(Column::Cargo)
            .distinct()
            .filter(alive())
            .filter(Column::Cargo.is_not_null())
            .order_by_asc(lower(Column::Cargo))
            .into_tuple()
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(rows
            .into_iter()
            .flatten()
            .filter(|c| !c.trim().is_empty())
            .collect())
    }

    async fn insert(&self, entity: &Empleado) -> AppResult<()> {
        Entity::insert(mapper::to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &Empleado, esperado: RowVersion) -> AppResult<()> {
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
        Ok(())
    }

    async fn count_liquidaciones(&self, id: Uuid) -> AppResult<u64> {
        liquidacion::Entity::find()
            .filter(liquidacion::Column::EmpleadoId.eq(id.to_string()))
            .filter(liquidacion::Column::IsDeleted.eq(false))
            .count(self.conn())
            .await
            .map_err(AppError::persistence)
    }

    async fn count_asistencias(&self, id: Uuid) -> AppResult<u64> {
        asistencia_empleado::Entity::find()
            .filter(asistencia_empleado::Column::EmpleadoId.eq(id.to_string()))
            .filter(asistencia_empleado::Column::IsDeleted.eq(false))
            .count(self.conn())
            .await
            .map_err(AppError::persistence)
    }

    async fn count_movimientos(&self, id: Uuid) -> AppResult<u64> {
        movimiento::Entity::find()
            .filter(movimiento::Column::EmpleadoId.eq(id.to_string()))
            .filter(movimiento::Column::IsDeleted.eq(false))
            .count(self.conn())
            .await
            .map_err(AppError::persistence)
    }
}
