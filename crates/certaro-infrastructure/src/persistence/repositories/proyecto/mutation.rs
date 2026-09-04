use chrono::{DateTime, Utc};
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::{Proyecto, Trabajo};
use certaro_domain::{time, EstadoTrabajo, RowVersion};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, FromQueryResult, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::persistence::mappers::proyecto as mapper;
use crate::persistence::mappers::trabajo as trabajo_mapper;
use crate::persistence::models::proyecto::{Column, Entity};
use crate::persistence::models::trabajo;

const ENTITY: &str = "Proyecto";

pub(crate) async fn numero_ocupado(
    conn: &DatabaseTransaction,
    numero: i32,
    excluir: Option<Uuid>,
) -> AppResult<bool> {
    let mut condition = Condition::all().add(Column::Numero.eq(numero));
    if let Some(id) = excluir {
        condition = condition.add(Column::Id.ne(id.to_string()));
    }
    let count = Entity::find()
        .filter(condition)
        .count(conn)
        .await
        .map_err(AppError::persistence)?;
    Ok(count > 0)
}

pub(crate) async fn siguiente_numero(conn: &DatabaseTransaction) -> AppResult<i32> {
    #[derive(Debug, FromQueryResult)]
    struct MaxRow {
        maximo: Option<i32>,
    }

    let row = Entity::find()
        .select_only()
        .expr_as(Expr::col(Column::Numero).max(), "maximo")
        .into_model::<MaxRow>()
        .one(conn)
        .await
        .map_err(AppError::persistence)?;

    Ok(row.and_then(|r| r.maximo).unwrap_or(0) + 1)
}

pub(crate) async fn insert_proyecto(conn: &DatabaseTransaction, entity: &Proyecto) -> AppResult<()> {
    Entity::insert(mapper::to_active(entity))
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;
    Ok(())
}

pub(crate) async fn update_proyecto(
    conn: &DatabaseTransaction,
    entity: &Proyecto,
    esperado: RowVersion,
) -> AppResult<()> {
    let result = Entity::update_many()
        .set(mapper::to_active(entity))
        .filter(Column::Id.eq(entity.id.to_string()))
        .filter(Column::RowVersion.eq(esperado.as_bytes().to_vec()))
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;

    if result.rows_affected == 0 {
        return Err(AppError::Concurrency { entity: ENTITY });
    }
    Ok(())
}

pub(crate) async fn soft_delete_proyecto(
    conn: &DatabaseTransaction,
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
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;

    if result.rows_affected == 0 {
        return Err(AppError::Concurrency { entity: ENTITY });
    }
    Ok(())
}

pub(crate) async fn count_trabajos(conn: &DatabaseTransaction, id: Uuid) -> AppResult<u64> {
    trabajo::Entity::find()
        .filter(trabajo::Column::ProyectoId.eq(id.to_string()))
        .filter(trabajo::Column::IsDeleted.eq(false))
        .count(conn)
        .await
        .map_err(AppError::persistence)
}

pub(crate) async fn trabajos_abiertos(
    conn: &DatabaseTransaction,
    id: Uuid,
) -> AppResult<Vec<Trabajo>> {
    let abiertos: Vec<i32> = EstadoTrabajo::ALL
        .iter()
        .filter(|e| e.esta_abierto())
        .map(|e| e.as_i32())
        .collect();

    let rows = trabajo::Entity::find()
        .filter(trabajo::Column::ProyectoId.eq(id.to_string()))
        .filter(trabajo::Column::IsDeleted.eq(false))
        .filter(trabajo::Column::Estado.is_in(abiertos))
        .order_by_asc(trabajo::Column::FechaInicio)
        .all(conn)
        .await
        .map_err(AppError::persistence)?;

    rows.into_iter().map(trabajo_mapper::to_domain).collect()
}
