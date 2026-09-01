use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use certaro_application::ports::repositories::{
    FacturaConResumen, FacturaFiltro, FacturaRepository, SortDir,
};
use certaro_application::{AppError, AppResult, PageRequest, PagedResult};
use certaro_domain::entities::{Factura, PagoFactura};
use certaro_domain::{time, EstadoFactura, Money, RowVersion};
use sea_orm::sea_query::{Alias, Expr, ExprTrait, Func, Query, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, FromQueryResult, JoinType, Order,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::persistence::mappers::factura as mapper;
use crate::persistence::mappers::{self as common};
use crate::persistence::models::factura::{self as model, Column, Entity};
use crate::persistence::models::{cliente, movimiento, pago_factura};

use super::estado_deuda_ids;

const ENTITY: &str = "Factura";
const ENTITY_PAGO: &str = "PagoFactura";

pub struct SeaOrmFacturaRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmFacturaRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[derive(Debug, FromQueryResult)]
struct RowConResumen {
    id: String,
    numero: String,
    fecha: String,
    fecha_vencimiento: Option<String>,
    cliente_id: String,
    estado: i32,
    subtotal: i64,
    iva: i64,
    total: i64,
    observaciones: Option<String>,
    created_at: String,
    updated_at: Option<String>,
    row_version: Vec<u8>,
    is_deleted: bool,
    deleted_at: Option<String>,
    cliente_nombre: String,
    pagado: i64,
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

fn alive() -> Condition {
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

/// Sum of the live payments of each invoice, correlated so the listing stays one statement.
fn pagado_expr() -> SimpleExpr {
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

/// The effective due date: the stored one, or `fecha + dias_default` when the column is empty.
///
/// Most legacy invoices have no due date, and treating those as never due would empty the arrears
/// report. SQLite's `date()` does the arithmetic on the stored `YYYY-MM-DD` directly.
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
    // Unpaid means: it counts as debt and something is still owed. An overpaid invoice has a
    // negative balance and is not unpaid, hence the strict comparison.
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

fn base_query() -> sea_orm::Select<Entity> {
    Entity::find()
        .join(JoinType::InnerJoin, cliente_join())
        .column_as(
            Expr::col((cliente::Entity, cliente::Column::Nombre)),
            "cliente_nombre",
        )
        .column_as(pagado_expr(), "pagado")
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
        let condition = filtro_condition(filtro, hoy, dias_vencimiento_default);
        // Newest first by default: an invoice screen is read from the last one backwards.
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

    async fn lookup(
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

    async fn de_cliente_con_pagos(
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

        // One query for the payments of the whole set instead of one per invoice: an account
        // statement of a long-standing customer would otherwise be dozens of round trips.
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
        // No version check and no version bump: the recalculation is a consequence of a payment
        // the caller already validated, and bumping would make the user's next save fail.
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

        // The payments go with it: a live payment pointing at a deleted invoice would keep
        // counting in every sum that does not join back to the header.
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
        let found = pago_factura::Entity::find_by_id(id.to_string())
            .filter(pago_factura::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(mapper::pago_to_domain).transpose()
    }

    async fn pagos_de(&self, factura_id: Uuid) -> AppResult<Vec<PagoFactura>> {
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

    async fn insert_pago(&self, entity: &PagoFactura) -> AppResult<()> {
        pago_factura::Entity::insert(mapper::pago_to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update_pago(&self, entity: &PagoFactura, esperado: RowVersion) -> AppResult<()> {
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

    async fn soft_delete_pago(
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
