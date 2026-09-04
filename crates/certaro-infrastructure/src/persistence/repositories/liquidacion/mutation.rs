use chrono::{DateTime, Utc};
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::{Liquidacion, LiquidacionAdelanto};
use certaro_domain::{time, RowVersion};
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::persistence::mappers::liquidacion as mapper;
use crate::persistence::models::liquidacion::{Column, Entity};
use crate::persistence::models::liquidacion_adelanto;

pub(super) const ENTITY: &str = "Liquidacion";

pub(super) async fn insert(
    conn: &DatabaseTransaction,
    entity: &Liquidacion,
) -> AppResult<()> {
    Entity::insert(mapper::to_active(entity))
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;
    Ok(())
}

pub(super) async fn update(
    conn: &DatabaseTransaction,
    entity: &Liquidacion,
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

pub(super) async fn insert_adelanto(
    conn: &DatabaseTransaction,
    entity: &LiquidacionAdelanto,
) -> AppResult<()> {
    // The partial unique index on `movimiento_id` is the real guard of INV-05; the read the use
    // case does first only makes the failure legible.
    liquidacion_adelanto::Entity::insert(mapper::adelanto_to_active(entity))
        .exec(conn)
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

pub(super) async fn marcar_pdf_generado(
    conn: &DatabaseTransaction,
    id: Uuid,
    at: DateTime<Utc>,
) -> AppResult<()> {
    Entity::update_many()
        .col_expr(Column::PdfGeneradoAt, Expr::value(time::to_storage(at)))
        .filter(Column::Id.eq(id.to_string()))
        .filter(Column::PdfGeneradoAt.is_null())
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
        .exec(conn)
        .await
        .map_err(AppError::persistence)?;

    Ok(())
}

fn es_violacion_de_unicidad(error: &sea_orm::DbErr) -> bool {
    let texto = error.to_string().to_lowercase();
    texto.contains("unique") || texto.contains("2067")
}
