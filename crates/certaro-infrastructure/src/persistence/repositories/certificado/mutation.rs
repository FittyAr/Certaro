use chrono::{DateTime, Utc};
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::{Certificado, CertificadoItem};
use certaro_domain::{time, RowVersion};
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::persistence::mappers::certificado as mapper;
use crate::persistence::models::certificado::{Column, Entity};
use crate::persistence::models::certificado_item;

pub(super) const ENTITY: &str = "Certificado";

pub(super) async fn insert(
    conn: &DatabaseTransaction,
    entity: &Certificado,
) -> AppResult<()> {
    Entity::insert(mapper::to_active(entity))
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;
    Ok(())
}

pub(super) async fn insert_item(
    conn: &DatabaseTransaction,
    entity: &CertificadoItem,
) -> AppResult<()> {
    certificado_item::Entity::insert(mapper::item_to_active(entity))
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;
    Ok(())
}

pub(super) async fn update_observaciones(
    conn: &DatabaseTransaction,
    id: Uuid,
    observaciones: Option<&str>,
    esperado: RowVersion,
    at: DateTime<Utc>,
) -> AppResult<()> {
    let result = Entity::update_many()
        .col_expr(
            Column::Observaciones,
            Expr::value(observaciones.map(ToOwned::to_owned)),
        )
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

    // The lines go with the certificate: leaving them alive would keep them in the accumulated
    // sum and the voided percentage would never come back.
    certificado_item::Entity::update_many()
        .col_expr(certificado_item::Column::IsDeleted, Expr::value(true))
        .col_expr(
            certificado_item::Column::DeletedAt,
            Expr::value(time::to_storage(at)),
        )
        .col_expr(
            certificado_item::Column::UpdatedAt,
            Expr::value(time::to_storage(at)),
        )
        .filter(certificado_item::Column::CertificadoId.eq(id.to_string()))
        .filter(certificado_item::Column::IsDeleted.eq(false))
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;

    Ok(())
}
