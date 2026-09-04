use certaro_application::ports::repositories::{
    CertificadoConRelaciones, CertificadoFiltro,
};
use certaro_application::AppResult;
use certaro_domain::entities::CertificadoItem;
use sea_orm::sea_query::{Expr, Func, Query, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, FromQueryResult, JoinType, QuerySelect,
};

use crate::persistence::mappers::certificado as mapper;
use crate::persistence::mappers::{self as common};
use crate::persistence::models::certificado::{self as model, Column, Entity};
use crate::persistence::models::{cliente, orden_trabajo, proyecto, trabajo};

pub(super) fn alive() -> Condition {
    Condition::all().add(Column::IsDeleted.eq(false))
}

#[derive(Debug, FromQueryResult)]
pub(super) struct RowConRelaciones {
    pub id: String,
    pub orden_trabajo_id: String,
    pub numero: i32,
    pub fecha: String,
    pub observaciones: Option<String>,
    pub total_certificado: i64,
    pub ajuste_uocra: i64,
    pub otros_descuentos: i64,
    pub total_neto: i64,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
    pub orden_titulo: String,
    pub trabajo_id: String,
    pub trabajo_descripcion: String,
    pub proyecto_id: String,
    pub proyecto_numero: i32,
    pub proyecto_nombre: String,
    pub cliente_id: String,
    pub cliente_nombre: String,
    /// `MAX(numero)` of the order, deleted certificates included: a spent number still counts.
    pub ultimo_numero: i32,
}

impl RowConRelaciones {
    pub(super) fn into_relaciones(self, items: Vec<CertificadoItem>) -> AppResult<CertificadoConRelaciones> {
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

pub(super) fn orden_join() -> sea_orm::RelationDef {
    Entity::belongs_to(orden_trabajo::Entity)
        .from(Column::OrdenTrabajoId)
        .to(orden_trabajo::Column::Id)
        .into()
}

pub(super) fn trabajo_join() -> sea_orm::RelationDef {
    orden_trabajo::Entity::belongs_to(trabajo::Entity)
        .from(orden_trabajo::Column::TrabajoId)
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

/// `MAX(numero)` of the certificate's own order, as a correlated subquery so the flag travels with
/// the row and the list does not need a second pass to know which one can be voided.
pub(super) fn ultimo_numero_expr() -> SimpleExpr {
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

pub(super) fn filtro_condition(filtro: &CertificadoFiltro) -> Condition {
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

pub(super) fn joined() -> sea_orm::Select<Entity> {
    Entity::find()
        .join(JoinType::InnerJoin, orden_join())
        .join(JoinType::InnerJoin, trabajo_join())
        .join(JoinType::InnerJoin, proyecto_join())
        .join(JoinType::InnerJoin, cliente_join())
}

pub(super) fn base_query() -> sea_orm::Select<Entity> {
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
