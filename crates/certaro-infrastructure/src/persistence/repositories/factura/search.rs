use chrono::NaiveDate;
use certaro_application::ports::repositories::{FacturaConResumen, FacturaFiltro, SortDir};
use certaro_application::{AppError, AppResult, PageRequest, PagedResult};
use certaro_domain::entities::Factura;
use certaro_domain::Money;
use sea_orm::sea_query::{Alias, Expr, ExprTrait, Func, Query, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, FromQueryResult, JoinType, Order, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::persistence::mappers::factura as mapper;
use crate::persistence::mappers::{self as common};
use crate::persistence::models::factura::{self as model, Column, Entity};
use crate::persistence::models::{cliente, pago_factura};
use crate::persistence::repositories::estado_deuda_ids;

use super::SeaOrmFacturaRepository;

#[derive(Debug, FromQueryResult)]
pub(super) struct RowConResumen {
    pub id: String,
    pub numero: String,
    pub fecha: String,
    pub fecha_vencimiento: Option<String>,
    pub cliente_id: String,
    pub estado: i32,
    pub subtotal: i64,
    pub iva: i64,
    pub total: i64,
    pub observaciones: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
    pub cliente_nombre: String,
    pub pagado: i64,
}

impl TryFrom<RowConResumen> for FacturaConResumen {
    type Error = AppError;

    fn try_from(row: RowConResumen) -> Result<Self, Self::Error> {
        let model = model::Model {
            id: row.id,
            numero: row.numero,
            fecha: row.fecha,
            fecha_vencimiento: row.fecha_vencimiento,
            cliente_id: row.cliente_id,
            estado: row.estado,
            subtotal: row.subtotal,
            iva: row.iva,
            total: row.total,
            observaciones: row.observaciones,
            created_at: row.created_at,
            updated_at: row.updated_at,
            row_version: row.row_version,
            is_deleted: row.is_deleted,
            deleted_at: row.deleted_at,
        };
        let pagado = Money::from_raw(row.pagado);
        let factura = mapper::to_domain(model)?;
        let saldo = factura.total.checked_sub(pagado).map_err(AppError::from)?;
        Ok(Self {
            factura,
            cliente_nombre: row.cliente_nombre,
            pagado,
            saldo,
        })
    }
}

pub(super) fn alive() -> Condition {
    Condition::all().add(Column::IsDeleted.eq(false))
}

fn lower(column: Column) -> SimpleExpr {
    Func::lower(Expr::col((Entity, column))).into()
}

fn cliente_join() -> sea_orm::RelationDef {
    Entity::belongs_to(cliente::Entity)
        .from(Column::ClienteId)
        .to(cliente::Column::Id)
        .into()
}

pub(super) fn pagado_expr() -> SimpleExpr {
    Func::coalesce([
        SimpleExpr::SubQuery(
            None,
            Box::new(
                Query::select()
                    .expr(Func::sum(Expr::col(pago_factura::Column::Monto)))
                    .from(pago_factura::Entity)
                    .and_where(
                        Expr::col((pago_factura::Entity, pago_factura::Column::FacturaId))
                            .equals((Entity, Column::Id)),
                    )
                    .and_where(
                        Expr::col((pago_factura::Entity, pago_factura::Column::IsDeleted))
                            .eq(false),
                    )
                    .take()
                    .into_sub_query_statement(),
            ),
        ),
        Expr::value(0i64),
    ])
    .into()
}

fn saldo_expr() -> SimpleExpr {
    Expr::col((Entity, Column::Total)).sub(pagado_expr())
}

fn vencimiento_expr(dias_default: u32) -> SimpleExpr {
    Func::coalesce([
        Expr::col((Entity, Column::FechaVencimiento)).into(),
        Func::cust(Alias::new("date"))
            .args([
                Expr::col((Entity, Column::Fecha)).into(),
                Expr::value(format!("+{dias_default} days")),
            ])
            .into(),
    ])
    .into()
}

fn filtro_condition(filtro: &FacturaFiltro, hoy: NaiveDate, dias_default: u32) -> Condition {
    let mut c = alive();

    if let Some(texto) = filtro.texto.as_deref() {
        let patron = format!("%{}%", texto.trim().to_lowercase());
        c = c.add(
            Condition::any()
                .add(lower(Column::Numero).like(patron.clone()))
                .add(
                    SimpleExpr::from(Func::lower(Expr::col((
                        cliente::Entity,
                        cliente::Column::Nombre,
                    ))))
                    .like(patron),
                ),
        );
    }
    if let Some(id) = filtro.cliente_id {
        c = c.add(Column::ClienteId.eq(id.to_string()));
    }
    if !filtro.estados.is_empty() {
        c = c.add(
            Column::Estado.is_in(
                filtro
                    .estados
                    .iter()
                    .map(|e| e.as_i32())
                    .collect::<Vec<_>>(),
            ),
        );
    }
    if let Some(date) = filtro.fecha_desde {
        c = c.add(Column::Fecha.gte(common::civil_to_storage(date)));
    }
    if let Some(date) = filtro.fecha_hasta {
        c = c.add(Column::Fecha.lte(common::civil_to_storage(date)));
    }
    if filtro.solo_impagas {
        c = c
            .add(Column::Estado.is_in(estado_deuda_ids()))
            .add(saldo_expr().gt(0i64));
    }
    if filtro.solo_vencidas {
        c = c
            .add(Column::Estado.is_in(estado_deuda_ids()))
            .add(saldo_expr().gt(0i64))
            .add(vencimiento_expr(dias_default).lt(common::civil_to_storage(hoy)));
    }
    c
}

pub(super) fn base_query() -> sea_orm::Select<Entity> {
    Entity::find()
        .join(JoinType::InnerJoin, cliente_join())
        .column_as(
            Expr::col((cliente::Entity, cliente::Column::Nombre)),
            "cliente_nombre",
        )
        .column_as(pagado_expr(), "pagado")
}

impl SeaOrmFacturaRepository {
    pub(super) async fn impl_search(
        &self,
        filtro: &FacturaFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
        hoy: NaiveDate,
        dias_vencimiento_default: u32,
    ) -> AppResult<PagedResult<FacturaConResumen>> {
        let condition = filtro_condition(filtro, hoy, dias_vencimiento_default);
        let order = match (sort_by, sort_dir) {
            (None, SortDir::Asc) => Order::Desc,
            (_, SortDir::Asc) => Order::Asc,
            (_, SortDir::Desc) => Order::Desc,
        };

        let mut query = base_query().filter(condition.clone());

        query = match sort_by {
            Some("numero") => query.order_by(lower(Column::Numero), order),
            Some("total") => query.order_by(Column::Total, order),
            Some("estado") => query.order_by(Column::Estado, order),
            Some("pagado") => query.order_by(Expr::col(Alias::new("pagado")), order),
            Some("saldo") => query.order_by(saldo_expr(), order),
            Some("fechaVencimiento") => {
                query.order_by(vencimiento_expr(dias_vencimiento_default), order)
            }
            Some("clienteNombre") => query.order_by(
                SimpleExpr::from(Func::lower(Expr::col((
                    cliente::Entity,
                    cliente::Column::Nombre,
                )))),
                order,
            ),
            Some("createdAt") => query.order_by(Column::CreatedAt, order),
            _ => query.order_by(Column::Fecha, order),
        }
        .order_by_desc(Expr::col((Entity, Column::Id)));

        let total = Entity::find()
            .join(JoinType::InnerJoin, cliente_join())
            .filter(condition)
            .count(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if let Some(limit) = page.limit() {
            query = query.limit(limit).offset(page.offset());
        }

        let rows = query
            .into_model::<RowConResumen>()
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        let items = rows
            .into_iter()
            .map(FacturaConResumen::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PagedResult::new(items, total, page))
    }

    pub(super) async fn impl_lookup(
        &self,
        cliente_id: Option<Uuid>,
        solo_impagas: bool,
        texto: Option<&str>,
        limite: u64,
    ) -> AppResult<Vec<Factura>> {
        let mut condition = alive();
        if let Some(texto) = texto {
            condition = condition
                .add(lower(Column::Numero).like(format!("%{}%", texto.trim().to_lowercase())));
        }
        if let Some(id) = cliente_id {
            condition = condition.add(Column::ClienteId.eq(id.to_string()));
        }
        if solo_impagas {
            condition = condition
                .add(Column::Estado.is_in(estado_deuda_ids()))
                .add(saldo_expr().gt(0i64));
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

    pub(super) async fn impl_de_cliente_con_pagos(
        &self,
        cliente_id: Uuid,
        incluir_pagadas: bool,
    ) -> AppResult<Vec<Factura>> {
        let mut condition = alive()
            .add(Column::ClienteId.eq(cliente_id.to_string()))
            .add(Column::Estado.is_in(estado_deuda_ids()));
        if !incluir_pagadas {
            condition = condition.add(saldo_expr().gt(0i64));
        }

        let rows = Entity::find()
            .filter(condition)
            .order_by_asc(Column::Fecha)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        let ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
        let pagos = if ids.is_empty() {
            Vec::new()
        } else {
            pago_factura::Entity::find()
                .filter(pago_factura::Column::FacturaId.is_in(ids))
                .filter(pago_factura::Column::IsDeleted.eq(false))
                .order_by_asc(pago_factura::Column::Fecha)
                .all(self.conn())
                .await
                .map_err(AppError::persistence)?
        };

        let mut facturas = rows
            .into_iter()
            .map(mapper::to_domain)
            .collect::<Result<Vec<_>, _>>()?;

        for pago in pagos {
            let factura_id = common::uuid(&pago.factura_id)?;
            let pago = mapper::pago_to_domain(pago)?;
            if let Some(factura) = facturas.iter_mut().find(|f| f.id == factura_id) {
                factura.pagos.push(pago);
            }
        }

        Ok(facturas)
    }
}
