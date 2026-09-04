use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use certaro_application::ports::repositories::{
    FacturaConResumen, FacturaFiltro, FacturaRepository, SortDir,
};
use certaro_application::{AppError, AppResult, PageRequest, PagedResult};
use certaro_domain::entities::{Factura, PagoFactura};
use certaro_domain::{time, EstadoFactura, RowVersion};
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, PaginatorTrait, QueryFilter};
use uuid::Uuid;

use crate::persistence::mappers::factura as mapper;
use crate::persistence::models::factura::{Column, Entity};
use crate::persistence::models::{movimiento, pago_factura};

mod pagos;
mod search;

use search::{alive, base_query, RowConResumen};

const ENTITY: &str = "Factura";
pub(super) const ENTITY_PAGO: &str = "PagoFactura";

pub struct SeaOrmFacturaRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmFacturaRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    pub(crate) fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[async_trait]
impl FacturaRepository for SeaOrmFacturaRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Factura>> {
        let found = Entity::find_by_id(id.to_string())
            .filter(alive())
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(mapper::to_domain).transpose()
    }

    async fn find_con_pagos(&self, id: Uuid) -> AppResult<Option<Factura>> {
        let Some(mut factura) = self.find_by_id(id).await? else {
            return Ok(None);
        };
        factura.pagos = self.pagos_de(id).await?;
        Ok(Some(factura))
    }

    async fn find_detalle(&self, id: Uuid) -> AppResult<Option<FacturaConResumen>> {
        let found = base_query()
            .filter(alive())
            .filter(Expr::col((Entity, Column::Id)).eq(id.to_string()))
            .into_model::<RowConResumen>()
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        let Some(row) = found else { return Ok(None) };
        let mut detalle = FacturaConResumen::try_from(row)?;
        detalle.factura.pagos = self.pagos_de(id).await?;
        Ok(Some(detalle))
    }

    async fn search(
        &self,
        filtro: &FacturaFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
        hoy: NaiveDate,
        dias_vencimiento_default: u32,
    ) -> AppResult<PagedResult<FacturaConResumen>> {
        self.impl_search(
            filtro,
            page,
            sort_by,
            sort_dir,
            hoy,
            dias_vencimiento_default,
        )
        .await
    }

    async fn lookup(
        &self,
        cliente_id: Option<Uuid>,
        solo_impagas: bool,
        texto: Option<&str>,
        limite: u64,
    ) -> AppResult<Vec<Factura>> {
        self.impl_lookup(cliente_id, solo_impagas, texto, limite)
            .await
    }

    async fn de_cliente_con_pagos(
        &self,
        cliente_id: Uuid,
        incluir_pagadas: bool,
    ) -> AppResult<Vec<Factura>> {
        self.impl_de_cliente_con_pagos(cliente_id, incluir_pagadas)
            .await
    }

    async fn insert(&self, entity: &Factura) -> AppResult<()> {
        Entity::insert(mapper::to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &Factura, esperado: RowVersion) -> AppResult<()> {
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

    async fn update_estado(&self, id: Uuid, estado: EstadoFactura) -> AppResult<()> {
        Entity::update_many()
            .col_expr(Column::Estado, Expr::value(estado.as_i32()))
            .filter(Column::Id.eq(id.to_string()))
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

        pago_factura::Entity::update_many()
            .col_expr(pago_factura::Column::IsDeleted, Expr::value(true))
            .col_expr(
                pago_factura::Column::DeletedAt,
                Expr::value(time::to_storage(at)),
            )
            .col_expr(
                pago_factura::Column::UpdatedAt,
                Expr::value(time::to_storage(at)),
            )
            .filter(pago_factura::Column::FacturaId.eq(id.to_string()))
            .filter(pago_factura::Column::IsDeleted.eq(false))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;

        Ok(())
    }

    async fn count_movimientos(&self, id: Uuid) -> AppResult<u64> {
        movimiento::Entity::find()
            .filter(movimiento::Column::FacturaId.eq(id.to_string()))
            .filter(movimiento::Column::IsDeleted.eq(false))
            .count(self.conn())
            .await
            .map_err(AppError::persistence)
    }

    async fn find_pago(&self, id: Uuid) -> AppResult<Option<PagoFactura>> {
        self.impl_find_pago(id).await
    }

    async fn pagos_de(&self, factura_id: Uuid) -> AppResult<Vec<PagoFactura>> {
        self.impl_pagos_de(factura_id).await
    }

    async fn insert_pago(&self, entity: &PagoFactura) -> AppResult<()> {
        self.impl_insert_pago(entity).await
    }

    async fn update_pago(&self, entity: &PagoFactura, esperado: RowVersion) -> AppResult<()> {
        self.impl_update_pago(entity, esperado).await
    }

    async fn soft_delete_pago(
        &self,
        id: Uuid,
        esperado: RowVersion,
        at: DateTime<Utc>,
    ) -> AppResult<()> {
        self.impl_soft_delete_pago(id, esperado, at).await
    }
}
