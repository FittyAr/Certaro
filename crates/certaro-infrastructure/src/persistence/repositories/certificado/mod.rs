use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use certaro_application::ports::repositories::{
    CertificadoConRelaciones, CertificadoFiltro, CertificadoRepository, SortDir,
};
use certaro_application::{AppError, AppResult, PageRequest, PagedResult};
use certaro_domain::entities::{Certificado, CertificadoItem};
use certaro_domain::{Decimal4, RowVersion};
use sea_orm::sea_query::{Expr, Func, SimpleExpr};
use sea_orm::{
    ColumnTrait, DatabaseTransaction, EntityTrait, FromQueryResult, JoinType, Order,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::persistence::mappers::certificado as mapper;
use crate::persistence::mappers::{self as common};
use crate::persistence::models::certificado::{Column, Entity};
use crate::persistence::models::certificado_item;

mod mutation;
mod query;

use query::{alive, base_query, filtro_condition, joined, RowConRelaciones};

pub struct SeaOrmCertificadoRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmCertificadoRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }

    async fn items_de(&self, certificado_id: Uuid) -> AppResult<Vec<CertificadoItem>> {
        let rows = certificado_item::Entity::find()
            .filter(certificado_item::Column::CertificadoId.eq(certificado_id.to_string()))
            .filter(certificado_item::Column::IsDeleted.eq(false))
            .order_by_asc(certificado_item::Column::Id)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::item_to_domain).collect()
    }
}

#[async_trait]
impl CertificadoRepository for SeaOrmCertificadoRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Certificado>> {
        let found = Entity::find_by_id(id.to_string())
            .filter(alive())
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(mapper::to_domain).transpose()
    }

    async fn find_con_items(&self, id: Uuid) -> AppResult<Option<Certificado>> {
        let Some(mut certificado) = self.find_by_id(id).await? else {
            return Ok(None);
        };
        certificado.items = self.items_de(id).await?;
        Ok(Some(certificado))
    }

    async fn find_detalle(&self, id: Uuid) -> AppResult<Option<CertificadoConRelaciones>> {
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
        Ok(Some(row.into_relaciones(items)?))
    }

    async fn search(
        &self,
        filtro: &CertificadoFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<CertificadoConRelaciones>> {
        let condition = filtro_condition(filtro);
        // Newest first by default: the certificate being looked for is the one just issued.
        let order = match (sort_by, sort_dir) {
            (None, SortDir::Asc) => Order::Desc,
            (_, SortDir::Asc) => Order::Asc,
            (_, SortDir::Desc) => Order::Desc,
        };

        let mut query = base_query().filter(condition.clone());

        query = match sort_by {
            Some("numero") => query.order_by(Column::Numero, order),
            Some("totalNeto") => query.order_by(Column::TotalNeto, order),
            Some("createdAt") => query.order_by(Column::CreatedAt, order),
            _ => query.order_by(Column::Fecha, order),
        }
        .order_by_desc(Expr::col((Entity, Column::Id)));

        let total = joined()
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

        // The list does not carry the lines: the grid shows totals, and the detail loads them.
        let items = rows
            .into_iter()
            .map(|row| row.into_relaciones(Vec::new()))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PagedResult::new(items, total, page))
    }

    async fn de_orden(&self, orden_trabajo_id: Uuid) -> AppResult<Vec<Certificado>> {
        let rows = Entity::find()
            .filter(alive())
            .filter(Column::OrdenTrabajoId.eq(orden_trabajo_id.to_string()))
            .order_by_desc(Column::Numero)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::to_domain).collect()
    }

    async fn ultimo_numero(&self, orden_trabajo_id: Uuid) -> AppResult<i32> {
        #[derive(Debug, FromQueryResult)]
        struct Row {
            numero: Option<i32>,
        }

        // No `alive()` filter here on purpose: a voided certificate keeps its number spent, so the
        // next one continues the sequence rather than reusing a gap (INV-15).
        let row = Entity::find()
            .select_only()
            .column_as(
                SimpleExpr::from(Func::max(Expr::col((Entity, Column::Numero)))),
                "numero",
            )
            .filter(Column::OrdenTrabajoId.eq(orden_trabajo_id.to_string()))
            .into_model::<Row>()
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;

        Ok(row.and_then(|r| r.numero).unwrap_or(0))
    }

    async fn acumulado_por_item(&self, orden_trabajo_id: Uuid) -> AppResult<Vec<(Uuid, Decimal4)>> {
        #[derive(Debug, FromQueryResult)]
        struct Row {
            orden_trabajo_item_id: String,
            total: Option<i64>,
        }

        // Summed from the live certificates: voiding one has to give the percentage back.
        let rows = certificado_item::Entity::find()
            .select_only()
            .column(certificado_item::Column::OrdenTrabajoItemId)
            .column_as(
                Expr::col((
                    certificado_item::Entity,
                    certificado_item::Column::PorcentajeActual,
                ))
                .sum(),
                "total",
            )
            .join(
                JoinType::InnerJoin,
                certificado_item::Entity::belongs_to(Entity)
                    .from(certificado_item::Column::CertificadoId)
                    .to(Column::Id)
                    .into(),
            )
            .filter(certificado_item::Column::IsDeleted.eq(false))
            .filter(Expr::col((Entity, Column::IsDeleted)).eq(false))
            .filter(Expr::col((Entity, Column::OrdenTrabajoId)).eq(orden_trabajo_id.to_string()))
            .group_by(certificado_item::Column::OrdenTrabajoItemId)
            .into_model::<Row>()
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        rows.iter()
            .map(|r| {
                Ok((
                    common::uuid(&r.orden_trabajo_item_id)?,
                    Decimal4::from_raw(r.total.unwrap_or(0)),
                ))
            })
            .collect()
    }

    async fn insert(&self, entity: &Certificado) -> AppResult<()> {
        mutation::insert(self.conn(), entity).await
    }

    async fn insert_item(&self, entity: &CertificadoItem) -> AppResult<()> {
        mutation::insert_item(self.conn(), entity).await
    }

    async fn update_observaciones(
        &self,
        id: Uuid,
        observaciones: Option<&str>,
        esperado: RowVersion,
        at: DateTime<Utc>,
    ) -> AppResult<()> {
        mutation::update_observaciones(self.conn(), id, observaciones, esperado, at).await
    }

    async fn soft_delete(
        &self,
        id: Uuid,
        esperado: RowVersion,
        at: DateTime<Utc>,
    ) -> AppResult<()> {
        mutation::soft_delete(self.conn(), id, esperado, at).await
    }
}
