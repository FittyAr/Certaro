use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use certaro_application::ports::repositories::{OrdenTrabajoConRelaciones, OrdenTrabajoRepository};
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::{OrdenTrabajo, OrdenTrabajoItem};
use certaro_domain::{time, Decimal4, RowVersion};
use sea_orm::sea_query::{Expr, Func, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, FromQueryResult, JoinType,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::persistence::mappers::orden_trabajo as mapper;
use crate::persistence::mappers::{self as common};
use crate::persistence::models::orden_trabajo::{self as model, Column, Entity};
use crate::persistence::models::{
    certificado, certificado_item, cliente, proyecto, orden_trabajo_item, trabajo,
};

const ENTITY: &str = "OrdenTrabajo";

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

#[derive(Debug, FromQueryResult)]
struct RowConRelaciones {
    id: String,
    trabajo_id: String,
    titulo: String,
    numero_certificado: Option<String>,
    fecha: String,
    observaciones: Option<String>,
    ajuste_uocra_porcentaje: i64,
    otros_descuentos: i64,
    created_at: String,
    updated_at: Option<String>,
    row_version: Vec<u8>,
    is_deleted: bool,
    deleted_at: Option<String>,
    trabajo_descripcion: String,
    proyecto_id: String,
    proyecto_numero: i32,
    proyecto_nombre: String,
    cliente_id: String,
    cliente_nombre: String,
}

impl RowConRelaciones {
    fn into_relaciones(self, items: Vec<OrdenTrabajoItem>) -> AppResult<OrdenTrabajoConRelaciones> {
        let model = model::Model {
            id: self.id,
            trabajo_id: self.trabajo_id,
            titulo: self.titulo,
            numero_certificado: self.numero_certificado,
            fecha: self.fecha,
            observaciones: self.observaciones,
            ajuste_uocra_porcentaje: self.ajuste_uocra_porcentaje,
            otros_descuentos: self.otros_descuentos,
            created_at: self.created_at,
            updated_at: self.updated_at,
            row_version: self.row_version,
            is_deleted: self.is_deleted,
            deleted_at: self.deleted_at,
        };
        let mut orden = mapper::to_domain(model)?;
        orden.items = items;
        Ok(OrdenTrabajoConRelaciones {
            orden,
            trabajo_descripcion: self.trabajo_descripcion,
            proyecto_id: common::uuid(&self.proyecto_id)?,
            proyecto_numero: self.proyecto_numero,
            proyecto_nombre: self.proyecto_nombre,
            cliente_id: common::uuid(&self.cliente_id)?,
            cliente_nombre: self.cliente_nombre,
            certificados_count: 0,
        })
    }
}

fn alive() -> Condition {
    Condition::all().add(Column::IsDeleted.eq(false))
}

fn lower(column: Column) -> SimpleExpr {
    Func::lower(Expr::col((Entity, column))).into()
}

fn trabajo_join() -> sea_orm::RelationDef {
    Entity::belongs_to(trabajo::Entity)
        .from(Column::TrabajoId)
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

fn base_query() -> sea_orm::Select<Entity> {
    Entity::find()
        .join(JoinType::InnerJoin, trabajo_join())
        .join(JoinType::InnerJoin, proyecto_join())
        .join(JoinType::InnerJoin, cliente_join())
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
        Entity::insert(mapper::to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &OrdenTrabajo, esperado: RowVersion) -> AppResult<()> {
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

    async fn touch(&self, id: Uuid, at: DateTime<Utc>) -> AppResult<()> {
        // Read-then-write rather than a SQL increment: the version is an opaque eight-byte blob
        // and only `RowVersion` knows how to advance it.
        let found = Entity::find_by_id(id.to_string())
            .filter(alive())
            .one(self.conn())
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
        Ok(())
    }

    async fn insert_item(&self, entity: &OrdenTrabajoItem) -> AppResult<()> {
        orden_trabajo_item::Entity::insert(mapper::item_to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update_item(&self, entity: &OrdenTrabajoItem) -> AppResult<()> {
        orden_trabajo_item::Entity::update_many()
            .set(mapper::item_to_active(entity))
            .filter(orden_trabajo_item::Column::Id.eq(entity.id.to_string()))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update_avance_item(
        &self,
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
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn soft_delete_items_excepto(
        &self,
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
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
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
