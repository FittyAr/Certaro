use certaro_application::ports::repositories::OrdenTrabajoConRelaciones;
use certaro_application::AppResult;
use certaro_domain::entities::OrdenTrabajoItem;
use sea_orm::sea_query::{Expr, Func, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, FromQueryResult, JoinType, QuerySelect,
};

use crate::persistence::mappers::orden_trabajo as mapper;
use crate::persistence::mappers::{self as common};
use crate::persistence::models::orden_trabajo::{self as model, Column, Entity};
use crate::persistence::models::{cliente, proyecto, trabajo};

pub(super) fn alive() -> Condition {
    Condition::all().add(Column::IsDeleted.eq(false))
}

pub(super) fn lower(column: Column) -> SimpleExpr {
    Func::lower(Expr::col((Entity, column))).into()
}

pub(super) fn trabajo_join() -> sea_orm::RelationDef {
    Entity::belongs_to(trabajo::Entity)
        .from(Column::TrabajoId)
        .to(trabajo::Column::Id)
        .into()
}

pub(super) fn proyecto_join() -> sea_orm::RelationDef {
    trabajo::Entity::belongs_to(proyecto::Entity)
        .from(trabajo::Column::ProyectoId)
        .to(proyecto::Column::Id)
        .into()
}

pub(super) fn cliente_join() -> sea_orm::RelationDef {
    proyecto::Entity::belongs_to(cliente::Entity)
        .from(proyecto::Column::ClienteId)
        .to(cliente::Column::Id)
        .into()
}

pub(super) fn base_query() -> sea_orm::Select<Entity> {
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

#[derive(Debug, FromQueryResult)]
pub(super) struct RowConRelaciones {
    pub id: String,
    pub trabajo_id: String,
    pub titulo: String,
    pub numero_certificado: Option<String>,
    pub fecha: String,
    pub observaciones: Option<String>,
    pub ajuste_uocra_porcentaje: i64,
    pub otros_descuentos: i64,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
    pub trabajo_descripcion: String,
    pub proyecto_id: String,
    pub proyecto_numero: i32,
    pub proyecto_nombre: String,
    pub cliente_id: String,
    pub cliente_nombre: String,
}

impl RowConRelaciones {
    pub(super) fn into_relaciones(self, items: Vec<OrdenTrabajoItem>) -> AppResult<OrdenTrabajoConRelaciones> {
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
