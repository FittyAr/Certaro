use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use certaro_application::ports::repositories::{
    CertificadoConRelaciones, CertificadoFiltro, CertificadoRepository, SortDir,
};
use certaro_application::{AppError, AppResult, PageRequest, PagedResult};
use certaro_domain::entities::{Certificado, CertificadoItem};
use certaro_domain::{time, Decimal4, RowVersion};
use sea_orm::sea_query::{Expr, Func, Query, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, FromQueryResult, JoinType, Order,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::persistence::mappers::certificado as mapper;
use crate::persistence::mappers::{self as common};
use crate::persistence::models::certificado::{self as model, Column, Entity};
use crate::persistence::models::{certificado_item, cliente, proyecto, orden_trabajo, trabajo};

const ENTITY: &str = "Certificado";

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

#[derive(Debug, FromQueryResult)]
struct RowConRelaciones {
    id: String,
    orden_trabajo_id: String,
    numero: i32,
    fecha: String,
    observaciones: Option<String>,
    total_certificado: i64,
    ajuste_uocra: i64,
    otros_descuentos: i64,
    total_neto: i64,
    created_at: String,
    updated_at: Option<String>,
    row_version: Vec<u8>,
    is_deleted: bool,
    deleted_at: Option<String>,
    orden_titulo: String,
    trabajo_id: String,
    trabajo_descripcion: String,
    proyecto_id: String,
    proyecto_numero: i32,
    proyecto_nombre: String,
    cliente_id: String,
    cliente_nombre: String,
    /// `MAX(numero)` of the order, deleted certificates included: a spent number still counts.
    ultimo_numero: i32,
}

impl RowConRelaciones {
    fn into_relaciones(self, items: Vec<CertificadoItem>) -> AppResult<CertificadoConRelaciones> {
        let es_ultimo = self.numero >= self.ultimo_numero;
        let model = model::Model {
            id: self.id,
            orden_trabajo_id: self.orden_trabajo_id.clone(),
            numero: self.numero,
            fecha: self.fecha,
            observaciones: self.observaciones,
            total_certificado: self.total_certificado,
            ajuste_uocra: self.ajuste_uocra,
            otros_descuentos: self.otros_descuentos,
            total_neto: self.total_neto,
            created_at: self.created_at,
            updated_at: self.updated_at,
            row_version: self.row_version,
            is_deleted: self.is_deleted,
            deleted_at: self.deleted_at,
        };
        let mut certificado = mapper::to_domain(model)?;
        certificado.items = items;
        Ok(CertificadoConRelaciones {
            orden_trabajo_id: certificado.orden_trabajo_id,
            certificado,
            orden_titulo: self.orden_titulo,
            trabajo_id: common::uuid(&self.trabajo_id)?,
            trabajo_descripcion: self.trabajo_descripcion,
            proyecto_id: common::uuid(&self.proyecto_id)?,
            proyecto_numero: self.proyecto_numero,
            proyecto_nombre: self.proyecto_nombre,
            cliente_id: common::uuid(&self.cliente_id)?,
            cliente_nombre: self.cliente_nombre,
            es_ultimo,
        })
    }
}

fn alive() -> Condition {
    Condition::all().add(Column::IsDeleted.eq(false))
}

fn orden_join() -> sea_orm::RelationDef {
    Entity::belongs_to(orden_trabajo::Entity)
        .from(Column::OrdenTrabajoId)
        .to(orden_trabajo::Column::Id)
        .into()
}

fn trabajo_join() -> sea_orm::RelationDef {
    orden_trabajo::Entity::belongs_to(trabajo::Entity)
        .from(orden_trabajo::Column::TrabajoId)
        .to(trabajo::Column::Id)
        .into()
}

fn proyecto_join() -> sea_orm::RelationDef {
    trabajo::Entity::belongs_to(proyecto::Entity)
        .from(trabajo::Column::ProyectoId)
        .to(proyecto::Column::Id)
        .into()
}

fn cliente_join() -> sea_orm::RelationDef {
    proyecto::Entity::belongs_to(cliente::Entity)
        .from(proyecto::Column::ClienteId)
        .to(cliente::Column::Id)
        .into()
}

/// `MAX(numero)` of the certificate's own order, as a correlated subquery so the flag travels with
/// the row and the list does not need a second pass to know which one can be voided.
fn ultimo_numero_expr() -> SimpleExpr {
    let mut sub = Query::select();
    sub.expr(Func::coalesce([
        Func::max(Expr::col((
            certificado_alias::Entity,
            certificado_alias::Column::Numero,
        )))
        .into(),
        Expr::value(0),
    ]))
    .from(certificado_alias::Entity)
    .and_where(
        Expr::col((
            certificado_alias::Entity,
            certificado_alias::Column::OrdenTrabajoId,
        ))
        .equals((Entity, Column::OrdenTrabajoId)),
    );
    SimpleExpr::SubQuery(None, Box::new(sub.into_sub_query_statement()))
}

/// The same table under its own name, for the correlated subquery above.
mod certificado_alias {
    pub use crate::persistence::models::certificado::{Column, Entity};
}

fn filtro_condition(filtro: &CertificadoFiltro) -> Condition {
    let mut c = alive();
    if let Some(id) = filtro.orden_trabajo_id {
        c = c.add(Column::OrdenTrabajoId.eq(id.to_string()));
    }
    if let Some(id) = filtro.trabajo_id {
        c = c.add(
            Expr::col((orden_trabajo::Entity, orden_trabajo::Column::TrabajoId)).eq(id.to_string()),
        );
    }
    if let Some(id) = filtro.proyecto_id {
        c = c.add(Expr::col((trabajo::Entity, trabajo::Column::ProyectoId)).eq(id.to_string()));
    }
    if let Some(id) = filtro.cliente_id {
        c = c.add(Expr::col((proyecto::Entity, proyecto::Column::ClienteId)).eq(id.to_string()));
    }
    // Civil dates compare as text: the stored format sorts chronologically.
    if let Some(date) = filtro.fecha_desde {
        c = c.add(Column::Fecha.gte(common::civil_to_storage(date)));
    }
    if let Some(date) = filtro.fecha_hasta {
        c = c.add(Column::Fecha.lte(common::civil_to_storage(date)));
    }
    c
}

fn joined() -> sea_orm::Select<Entity> {
    Entity::find()
        .join(JoinType::InnerJoin, orden_join())
        .join(JoinType::InnerJoin, trabajo_join())
        .join(JoinType::InnerJoin, proyecto_join())
        .join(JoinType::InnerJoin, cliente_join())
}

fn base_query() -> sea_orm::Select<Entity> {
    joined()
        .column_as(
            Expr::col((orden_trabajo::Entity, orden_trabajo::Column::Titulo)),
            "orden_titulo",
        )
        .column_as(
            Expr::col((trabajo::Entity, trabajo::Column::Id)),
            "trabajo_id",
        )
        .column_as(
            Expr::col((trabajo::Entity, trabajo::Column::Descripcion)),
            "trabajo_descripcion",
        )
        .column_as(Expr::col((proyecto::Entity, proyecto::Column::Id)), "proyecto_id")
        .column_as(
            Expr::col((proyecto::Entity, proyecto::Column::Numero)),
            "proyecto_numero",
        )
        .column_as(
            Expr::col((proyecto::Entity, proyecto::Column::Nombre)),
            "proyecto_nombre",
        )
        .column_as(
            Expr::col((cliente::Entity, cliente::Column::Id)),
            "cliente_id",
        )
        .column_as(
            Expr::col((cliente::Entity, cliente::Column::Nombre)),
            "cliente_nombre",
        )
        .column_as(ultimo_numero_expr(), "ultimo_numero")
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
        Entity::insert(mapper::to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn insert_item(&self, entity: &CertificadoItem) -> AppResult<()> {
        certificado_item::Entity::insert(mapper::item_to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update_observaciones(
        &self,
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
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;

        Ok(())
    }
}
