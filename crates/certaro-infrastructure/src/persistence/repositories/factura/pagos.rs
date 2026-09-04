use chrono::{DateTime, Utc};use certaro_application::{AppError, AppResult};
use certaro_domain::entities::PagoFactura;
use certaro_domain::{time, RowVersion};
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use crate::persistence::mappers::factura as mapper;
use crate::persistence::models::pago_factura;

use super::{SeaOrmFacturaRepository, ENTITY_PAGO};

impl SeaOrmFacturaRepository {
    pub(super) async fn impl_find_pago(&self, id: Uuid) -> AppResult<Option<PagoFactura>> {
        let found = pago_factura::Entity::find_by_id(id.to_string())
            .filter(pago_factura::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(mapper::pago_to_domain).transpose()
    }

    pub(super) async fn impl_pagos_de(&self, factura_id: Uuid) -> AppResult<Vec<PagoFactura>> {
        let rows = pago_factura::Entity::find()
            .filter(pago_factura::Column::FacturaId.eq(factura_id.to_string()))
            .filter(pago_factura::Column::IsDeleted.eq(false))
            .order_by_asc(pago_factura::Column::Fecha)
            .order_by_asc(pago_factura::Column::Id)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::pago_to_domain).collect()
    }

    pub(super) async fn impl_insert_pago(&self, entity: &PagoFactura) -> AppResult<()> {
        pago_factura::Entity::insert(mapper::pago_to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    pub(super) async fn impl_update_pago(
        &self,
        entity: &PagoFactura,
        esperado: RowVersion,
    ) -> AppResult<()> {
        let result = pago_factura::Entity::update_many()
            .set(mapper::pago_to_active(entity))
            .filter(pago_factura::Column::Id.eq(entity.id.to_string()))
            .filter(pago_factura::Column::RowVersion.eq(esperado.as_bytes().to_vec()))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if result.rows_affected == 0 {
            return Err(AppError::Concurrency {
                entity: ENTITY_PAGO,
            });
        }
        Ok(())
    }

    pub(super) async fn impl_soft_delete_pago(
        &self,
        id: Uuid,
        esperado: RowVersion,
        at: DateTime<Utc>,
    ) -> AppResult<()> {
        let result = pago_factura::Entity::update_many()
            .col_expr(pago_factura::Column::IsDeleted, Expr::value(true))
            .col_expr(
                pago_factura::Column::DeletedAt,
                Expr::value(time::to_storage(at)),
            )
            .col_expr(
                pago_factura::Column::UpdatedAt,
                Expr::value(time::to_storage(at)),
            )
            .col_expr(
                pago_factura::Column::RowVersion,
                Expr::value(esperado.next().as_bytes().to_vec()),
            )
            .filter(pago_factura::Column::Id.eq(id.to_string()))
            .filter(pago_factura::Column::RowVersion.eq(esperado.as_bytes().to_vec()))
            .filter(pago_factura::Column::IsDeleted.eq(false))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if result.rows_affected == 0 {
            return Err(AppError::Concurrency {
                entity: ENTITY_PAGO,
            });
        }
        Ok(())
    }
}
