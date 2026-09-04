use chrono::{DateTime, Utc};
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::{Cliente, ClienteContacto};
use certaro_domain::{time, RowVersion};
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, Condition, DatabaseTransaction, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::persistence::mappers::cliente as mapper;
use crate::persistence::models::cliente::{Column, Entity};
use crate::persistence::models::cliente_contacto;

const ENTITY: &str = "Cliente";
const ENTITY_CONTACTO: &str = "ClienteContacto";

pub(crate) async fn insert_cliente(conn: &DatabaseTransaction, entity: &Cliente) -> AppResult<()> {
    Entity::insert(mapper::to_active(entity))
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;
    Ok(())
}

pub(crate) async fn update_cliente(
    conn: &DatabaseTransaction,
    entity: &Cliente,
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

pub(crate) async fn soft_delete_cliente(
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

pub(crate) async fn insert_contacto(
    conn: &DatabaseTransaction,
    entity: &ClienteContacto,
) -> AppResult<()> {
    cliente_contacto::Entity::insert(mapper::contacto_to_active(entity))
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;
    Ok(())
}

pub(crate) async fn update_contacto(
    conn: &DatabaseTransaction,
    entity: &ClienteContacto,
) -> AppResult<()> {
    let result = cliente_contacto::Entity::update_many()
        .set(mapper::contacto_to_active(entity))
        .filter(cliente_contacto::Column::Id.eq(entity.id.to_string()))
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;

    if result.rows_affected == 0 {
        return Err(AppError::not_found(ENTITY_CONTACTO, entity.id));
    }
    Ok(())
}

pub(crate) async fn soft_delete_contactos_excepto(
    conn: &DatabaseTransaction,
    cliente_id: Uuid,
    conservar: &[Uuid],
    at: DateTime<Utc>,
) -> AppResult<()> {
    let mut condition = Condition::all()
        .add(cliente_contacto::Column::ClienteId.eq(cliente_id.to_string()))
        .add(cliente_contacto::Column::IsDeleted.eq(false));
    if !conservar.is_empty() {
        condition = condition.add(
            cliente_contacto::Column::Id
                .is_not_in(conservar.iter().map(Uuid::to_string).collect::<Vec<_>>()),
        );
    }

    cliente_contacto::Entity::update_many()
        .col_expr(cliente_contacto::Column::IsDeleted, Expr::value(true))
        .col_expr(
            cliente_contacto::Column::DeletedAt,
            Expr::value(time::to_storage(at)),
        )
        .col_expr(
            cliente_contacto::Column::UpdatedAt,
            Expr::value(time::to_storage(at)),
        )
        .filter(condition)
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;
    Ok(())
}
