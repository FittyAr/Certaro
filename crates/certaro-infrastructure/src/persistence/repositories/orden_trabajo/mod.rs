use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use certaro_application::ports::repositories::{OrdenTrabajoConRelaciones, OrdenTrabajoRepository};
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::{OrdenTrabajo, OrdenTrabajoItem};
use certaro_domain::{Decimal4, RowVersion};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, DatabaseTransaction, EntityTrait, FromQueryResult, JoinType, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::persistence::mappers::orden_trabajo as mapper;
use crate::persistence::mappers::{self as common};
use crate::persistence::models::orden_trabajo::{Column, Entity};
use crate::persistence::models::{certificado, certificado_item, orden_trabajo_item};

mod mutation;
mod query;

use query::{alive, base_query, lower, RowConRelaciones};

pub struct SeaOrmOrdenTrabajoRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmOrdenTrabajoRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }

    /// The live items of an order, in the order the sheet prints them.
    async fn items_de(&self, orden_trabajo_id: Uuid) -> AppResult<Vec<OrdenTrabajoItem>> {
        let rows = orden_trabajo_item::Entity::find()
            .filter(orden_trabajo_item::Column::OrdenTrabajoId.eq(orden_trabajo_id.to_string()))
            .filter(orden_trabajo_item::Column::IsDeleted.eq(false))
            .order_by_asc(orden_trabajo_item::Column::Orden)
            .order_by_asc(orden_trabajo_item::Column::Id)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::item_to_domain).collect()
    }
}

#[async_trait]
impl OrdenTrabajoRepository for SeaOrmOrdenTrabajoRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<OrdenTrabajo>> {
        let found = Entity::find_by_id(id.to_string())
            .filter(alive())
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(mapper::to_domain).transpose()
    }

    async fn find_con_items(&self, id: Uuid) -> AppResult<Option<OrdenTrabajo>> {
        let Some(mut orden) = self.find_by_id(id).await? else {
            return Ok(None);
        };
        orden.items = self.items_de(id).await?;
        Ok(Some(orden))
    }

    async fn find_detalle(&self, id: Uuid) -> AppResult<Option<OrdenTrabajoConRelaciones>> {
        let found = base_query()
            .filter(alive())
            .filter(Expr::col((Entity, Column::Id)).eq(id.to_string()))
            .into_model::<RowConRelaciones>()
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;

        let Some(row) = found else {
            return Ok(None);
        };
        let items = self.items_de(id).await?;
        let mut relaciones = row.into_relaciones(items)?;
        relaciones.certificados_count = self.count_certificados(id).await?;
        Ok(Some(relaciones))
    }

    async fn de_trabajo(&self, trabajo_id: Uuid) -> AppResult<Vec<OrdenTrabajoConRelaciones>> {
        let rows = base_query()
            .filter(alive())
            .filter(Column::TrabajoId.eq(trabajo_id.to_string()))
            .order_by_desc(Column::Fecha)
            .order_by_desc(Expr::col((Entity, Column::Id)))
            .into_model::<RowConRelaciones>()
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        // One extra round trip per order for its items and its certificate count. A job has a
        // handful of orders, so this is bounded; the alternative is a grouped join whose rows would
        // have to be stitched back together anyway.
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            let id = common::uuid(&row.id)?;
            let items = self.items_de(id).await?;
            let mut relaciones = row.into_relaciones(items)?;
            relaciones.certificados_count = self.count_certificados(id).await?;
            result.push(relaciones);
        }
        Ok(result)
    }

    async fn lookup(
        &self,
        trabajo_id: Option<Uuid>,
        texto: Option<&str>,
        limite: u64,
    ) -> AppResult<Vec<OrdenTrabajo>> {
        let mut condition = alive();
        if let Some(texto) = texto {
            condition = condition
                .add(lower(Column::Titulo).like(format!("%{}%", texto.trim().to_lowercase())));
        }
        if let Some(id) = trabajo_id {
            condition = condition.add(Column::TrabajoId.eq(id.to_string()));
        }
        let rows = Entity::find()
            .filter(condition)
            .order_by_desc(Column::Fecha)
            .limit(limite)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::to_domain).collect()
    }

    async fn insert(&self, entity: &OrdenTrabajo) -> AppResult<()> {
        mutation::insert(self.conn(), entity).await
    }

    async fn update(&self, entity: &OrdenTrabajo, esperado: RowVersion) -> AppResult<()> {
        mutation::update(self.conn(), entity, esperado).await
    }

    async fn touch(&self, id: Uuid, at: DateTime<Utc>) -> AppResult<()> {
        mutation::touch(self.conn(), id, at).await
    }

    async fn soft_delete(
        &self,
        id: Uuid,
        esperado: RowVersion,
        at: DateTime<Utc>,
    ) -> AppResult<()> {
        mutation::soft_delete(self.conn(), id, esperado, at).await
    }

    async fn insert_item(&self, entity: &OrdenTrabajoItem) -> AppResult<()> {
        mutation::insert_item(self.conn(), entity).await
    }

    async fn update_item(&self, entity: &OrdenTrabajoItem) -> AppResult<()> {
        mutation::update_item(self.conn(), entity).await
    }

    async fn update_avance_item(
        &self,
        id: Uuid,
        porcentaje_anterior: Decimal4,
        porcentaje_actual: Decimal4,
        ejecutado: bool,
        at: DateTime<Utc>,
    ) -> AppResult<()> {
        mutation::update_avance_item(self.conn(), id, porcentaje_anterior, porcentaje_actual, ejecutado, at).await
    }

    async fn soft_delete_items_excepto(
        &self,
        orden_trabajo_id: Uuid,
        conservar: &[Uuid],
        at: DateTime<Utc>,
    ) -> AppResult<()> {
        mutation::soft_delete_items_excepto(self.conn(), orden_trabajo_id, conservar, at).await
    }

    async fn items_certificados(&self, orden_trabajo_id: Uuid) -> AppResult<Vec<Uuid>> {
        #[derive(Debug, FromQueryResult)]
        struct Row {
            orden_trabajo_item_id: String,
        }

        // Through the certificate rather than through the item's own percentage: a line whose
        // progress was reverted by a void is no longer certified and must be droppable again.
        let rows = certificado_item::Entity::find()
            .select_only()
            .column(certificado_item::Column::OrdenTrabajoItemId)
            .distinct()
            .join(
                JoinType::InnerJoin,
                certificado_item::Entity::belongs_to(certificado::Entity)
                    .from(certificado_item::Column::CertificadoId)
                    .to(certificado::Column::Id)
                    .into(),
            )
            .filter(certificado_item::Column::IsDeleted.eq(false))
            .filter(Expr::col((certificado::Entity, certificado::Column::IsDeleted)).eq(false))
            .filter(
                Expr::col((certificado::Entity, certificado::Column::OrdenTrabajoId))
                    .eq(orden_trabajo_id.to_string()),
            )
            .into_model::<Row>()
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        rows.iter()
            .map(|r| common::uuid(&r.orden_trabajo_item_id))
            .collect()
    }

    async fn count_certificados(&self, orden_trabajo_id: Uuid) -> AppResult<u64> {
        certificado::Entity::find()
            .filter(certificado::Column::OrdenTrabajoId.eq(orden_trabajo_id.to_string()))
            .filter(certificado::Column::IsDeleted.eq(false))
            .count(self.conn())
            .await
            .map_err(AppError::persistence)
    }
}
