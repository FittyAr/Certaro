use chrono::{DateTime, Utc};
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::{OrdenTrabajo, OrdenTrabajoItem};
use certaro_domain::{time, Decimal4, RowVersion};
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, Condition, DatabaseTransaction, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::persistence::mappers::orden_trabajo as mapper;
use crate::persistence::mappers::{self as common};
use crate::persistence::models::orden_trabajo::{Column, Entity};
use crate::persistence::models::orden_trabajo_item;

pub(super) const ENTITY: &str = "OrdenTrabajo";

pub(super) async fn insert(
    conn: &DatabaseTransaction,
    entity: &OrdenTrabajo,
) -> AppResult<()> {
    Entity::insert(mapper::to_active(entity))
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;
    Ok(())
}

pub(super) async fn update(
    conn: &DatabaseTransaction,
    entity: &OrdenTrabajo,
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

pub(super) async fn touch(
    conn: &DatabaseTransaction,
    id: Uuid,
    at: DateTime<Utc>,
) -> AppResult<()> {
    let found = Entity::find_by_id(id.to_string())
        .filter(Column::IsDeleted.eq(false))
        .one(conn)
        .await
        .map_err(AppError::persistence)?
        .ok_or_else(|| AppError::not_found(ENTITY, id))?;
    let actual = common::row_version(&found.row_version)?;

    Entity::update_many()
        .col_expr(Column::UpdatedAt, Expr::value(time::to_storage(at)))
        .col_expr(
            Column::RowVersion,
            Expr::value(actual.next().as_bytes().to_vec()),
        )
        .filter(Column::Id.eq(id.to_string()))
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;
    Ok(())
}

pub(super) async fn soft_delete(
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

pub(super) async fn insert_item(
    conn: &DatabaseTransaction,
    entity: &OrdenTrabajoItem,
) -> AppResult<()> {
    orden_trabajo_item::Entity::insert(mapper::item_to_active(entity))
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;
    Ok(())
}

pub(super) async fn update_item(
    conn: &DatabaseTransaction,
    entity: &OrdenTrabajoItem,
) -> AppResult<()> {
    orden_trabajo_item::Entity::update_many()
        .set(mapper::item_to_active(entity))
        .filter(orden_trabajo_item::Column::Id.eq(entity.id.to_string()))
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;
    Ok(())
}

pub(super) async fn update_avance_item(
    conn: &DatabaseTransaction,
    id: Uuid,
    porcentaje_anterior: Decimal4,
    porcentaje_actual: Decimal4,
    ejecutado: bool,
    at: DateTime<Utc>,
) -> AppResult<()> {
    orden_trabajo_item::Entity::update_many()
        .col_expr(
            orden_trabajo_item::Column::PorcentajeAnterior,
            Expr::value(porcentaje_anterior.raw()),
        )
        .col_expr(
            orden_trabajo_item::Column::PorcentajeActual,
            Expr::value(porcentaje_actual.raw()),
        )
        .col_expr(
            orden_trabajo_item::Column::Ejecutado,
            Expr::value(ejecutado),
        )
        .col_expr(
            orden_trabajo_item::Column::UpdatedAt,
            Expr::value(time::to_storage(at)),
        )
        .filter(orden_trabajo_item::Column::Id.eq(id.to_string()))
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;
    Ok(())
}

pub(super) async fn soft_delete_items_excepto(
    conn: &DatabaseTransaction,
    orden_trabajo_id: Uuid,
    conservar: &[Uuid],
    at: DateTime<Utc>,
) -> AppResult<()> {
    let mut condition = Condition::all()
        .add(orden_trabajo_item::Column::OrdenTrabajoId.eq(orden_trabajo_id.to_string()))
        .add(orden_trabajo_item::Column::IsDeleted.eq(false));
    if !conservar.is_empty() {
        condition = condition.add(
            orden_trabajo_item::Column::Id.is_not_in(conservar.iter().map(ToString::to_string)),
        );
    }

    orden_trabajo_item::Entity::update_many()
        .col_expr(orden_trabajo_item::Column::IsDeleted, Expr::value(true))
        .col_expr(
            orden_trabajo_item::Column::DeletedAt,
            Expr::value(time::to_storage(at)),
        )
        .col_expr(
            orden_trabajo_item::Column::UpdatedAt,
            Expr::value(time::to_storage(at)),
        )
        .filter(condition)
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;
    Ok(())
}
