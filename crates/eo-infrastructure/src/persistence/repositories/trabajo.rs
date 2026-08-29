use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use eo_application::ports::repositories::{
    SortDir, TrabajoConRelaciones, TrabajoFiltro, TrabajoRepository,
};
use eo_application::{AppError, AppResult, PageRequest, PagedResult};
use eo_domain::entities::Trabajo;
use eo_domain::{time, RowVersion};
use sea_orm::sea_query::{Expr, Func, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, FromQueryResult, JoinType, Order,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::persistence::mappers::trabajo as mapper;
use crate::persistence::mappers::{self as common};
use crate::persistence::models::trabajo::{self as model, Column, Entity};
use crate::persistence::models::{cliente, movimiento, obra, orden_trabajo};

const ENTITY: &str = "Trabajo";

pub struct SeaOrmTrabajoRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmTrabajoRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[derive(Debug, FromQueryResult)]
struct RowConRelaciones {
    id: String,
    obra_id: String,
    descripcion: String,
    fecha_inicio: String,
    fecha_fin: Option<String>,
    presupuesto: i64,
    estado: i32,
    created_at: String,
    updated_at: Option<String>,
    row_version: Vec<u8>,
    is_deleted: bool,
    deleted_at: Option<String>,
    obra_numero: i32,
    obra_nombre: String,
    cliente_id: String,
    cliente_nombre: String,
}

impl TryFrom<RowConRelaciones> for TrabajoConRelaciones {
    type Error = AppError;

    fn try_from(row: RowConRelaciones) -> Result<Self, Self::Error> {
        let model = model::Model {
            id: row.id,
            obra_id: row.obra_id,
            descripcion: row.descripcion,
            fecha_inicio: row.fecha_inicio,
            fecha_fin: row.fecha_fin,
            presupuesto: row.presupuesto,
            estado: row.estado,
            created_at: row.created_at,
            updated_at: row.updated_at,
            row_version: row.row_version,
            is_deleted: row.is_deleted,
            deleted_at: row.deleted_at,
        };
        Ok(Self {
            trabajo: mapper::to_domain(model)?,
            obra_numero: row.obra_numero,
            obra_nombre: row.obra_nombre,
            cliente_id: common::uuid(&row.cliente_id)?,
            cliente_nombre: row.cliente_nombre,
        })
    }
}

fn alive() -> Condition {
    Condition::all().add(Column::IsDeleted.eq(false))
}

fn lower(column: Column) -> SimpleExpr {
    Func::lower(Expr::col((Entity, column))).into()
}

fn obra_join() -> sea_orm::RelationDef {
    Entity::belongs_to(obra::Entity)
        .from(Column::ObraId)
        .to(obra::Column::Id)
        .into()
}

fn cliente_join() -> sea_orm::RelationDef {
    obra::Entity::belongs_to(cliente::Entity)
        .from(obra::Column::ClienteId)
        .to(cliente::Column::Id)
        .into()
}

fn filtro_condition(filtro: &TrabajoFiltro) -> Condition {
    let mut c = alive();
    if let Some(texto) = filtro.texto.as_deref() {
        c = c.add(lower(Column::Descripcion).like(format!("%{}%", texto.trim().to_lowercase())));
    }
    if let Some(id) = filtro.obra_id {
        c = c.add(Column::ObraId.eq(id.to_string()));
    }
    // A job has no customer of its own: the filter goes through the site, which is exactly the
    // denormalised column the legacy schema got wrong.
    if let Some(id) = filtro.cliente_id {
        c = c.add(Expr::col((obra::Entity, obra::Column::ClienteId)).eq(id.to_string()));
    }
    if let Some(estado) = filtro.estado {
        c = c.add(Column::Estado.eq(estado.as_i32()));
    }
    // Civil dates compare as text because the stored format sorts chronologically.
    if let Some(date) = filtro.fecha_desde {
        c = c.add(Column::FechaInicio.gte(common::civil_to_storage(date)));
    }
    if let Some(date) = filtro.fecha_hasta {
        c = c.add(Column::FechaInicio.lte(common::civil_to_storage(date)));
    }
    c
}

fn base_query() -> sea_orm::Select<Entity> {
    Entity::find()
        .join(JoinType::InnerJoin, obra_join())
        .join(JoinType::InnerJoin, cliente_join())
        .column_as(
            Expr::col((obra::Entity, obra::Column::Numero)),
            "obra_numero",
        )
        .column_as(
            Expr::col((obra::Entity, obra::Column::Nombre)),
            "obra_nombre",
        )
        .column_as(
            Expr::col((cliente::Entity, cliente::Column::Id)),
            "cliente_id",
        )
        .column_as(
            Expr::col((cliente::Entity, cliente::Column::Nombre)),
            "cliente_nombre",
        )
}

/// The count query needs the site join too, because the customer filter lives on that table.
fn count_query() -> sea_orm::Select<Entity> {
    Entity::find().join(JoinType::InnerJoin, obra_join())
}

#[async_trait]
impl TrabajoRepository for SeaOrmTrabajoRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Trabajo>> {
        let found = Entity::find_by_id(id.to_string())
            .filter(alive())
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(mapper::to_domain).transpose()
    }

    async fn find_detalle(&self, id: Uuid) -> AppResult<Option<TrabajoConRelaciones>> {
        let found = base_query()
            .filter(alive())
            .filter(Expr::col((Entity, Column::Id)).eq(id.to_string()))
            .into_model::<RowConRelaciones>()
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(TrabajoConRelaciones::try_from).transpose()
    }

    async fn search(
        &self,
        filtro: &TrabajoFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<TrabajoConRelaciones>> {
        let condition = filtro_condition(filtro);
        // Newest first by default: the open job is the one being looked for.
        let order = match (sort_by, sort_dir) {
            (None, SortDir::Asc) => Order::Desc,
            (_, SortDir::Asc) => Order::Asc,
            (_, SortDir::Desc) => Order::Desc,
        };

        let mut query = base_query().filter(condition.clone());

        query = match sort_by {
            Some("descripcion") => query.order_by(lower(Column::Descripcion), order),
            Some("presupuesto") => query.order_by(Column::Presupuesto, order),
            Some("estado") => query.order_by(Column::Estado, order),
            Some("obraNombre") => query.order_by(
                SimpleExpr::from(Func::lower(Expr::col((obra::Entity, obra::Column::Nombre)))),
                order,
            ),
            Some("clienteNombre") => query.order_by(
                SimpleExpr::from(Func::lower(Expr::col((
                    cliente::Entity,
                    cliente::Column::Nombre,
                )))),
                order,
            ),
            Some("createdAt") => query.order_by(Column::CreatedAt, order),
            _ => query.order_by(Column::FechaInicio, order),
        }
        .order_by_desc(Expr::col((Entity, Column::Id)));

        let total = count_query()
            .filter(condition)
            .count(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if let Some(limit) = page.limit() {
            query = query.limit(limit).offset(page.offset());
        }

        let rows = query
            .into_model::<RowConRelaciones>()
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        let items = rows
            .into_iter()
            .map(TrabajoConRelaciones::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PagedResult::new(items, total, page))
    }

    async fn lookup(
        &self,
        obra_id: Option<Uuid>,
        texto: Option<&str>,
        limite: u64,
    ) -> AppResult<Vec<Trabajo>> {
        let mut condition = alive();
        if let Some(texto) = texto {
            condition = condition
                .add(lower(Column::Descripcion).like(format!("%{}%", texto.trim().to_lowercase())));
        }
        if let Some(id) = obra_id {
            condition = condition.add(Column::ObraId.eq(id.to_string()));
        }
        let rows = Entity::find()
            .filter(condition)
            .order_by_desc(Column::FechaInicio)
            .limit(limite)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::to_domain).collect()
    }

    async fn insert(&self, entity: &Trabajo) -> AppResult<()> {
        Entity::insert(mapper::to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &Trabajo, esperado: RowVersion) -> AppResult<()> {
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

    async fn count_ordenes(&self, id: Uuid) -> AppResult<u64> {
        orden_trabajo::Entity::find()
            .filter(orden_trabajo::Column::TrabajoId.eq(id.to_string()))
            .filter(orden_trabajo::Column::IsDeleted.eq(false))
            .count(self.conn())
            .await
            .map_err(AppError::persistence)
    }

    async fn count_movimientos(&self, id: Uuid) -> AppResult<u64> {
        movimiento::Entity::find()
            .filter(movimiento::Column::TrabajoId.eq(id.to_string()))
            .filter(movimiento::Column::IsDeleted.eq(false))
            .count(self.conn())
            .await
            .map_err(AppError::persistence)
    }
}
