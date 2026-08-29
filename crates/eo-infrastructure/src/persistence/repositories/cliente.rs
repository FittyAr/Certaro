use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use eo_application::ports::repositories::{
    ClienteConResumen, ClienteFiltro, ClienteRepository, SortDir,
};
use eo_application::{AppError, AppResult, PageRequest, PagedResult};
use eo_domain::entities::{Cliente, ClienteContacto};
use eo_domain::{time, Money, RowVersion};
use sea_orm::sea_query::{Alias, Expr, ExprTrait, Func, Query, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, FromQueryResult, Order,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::persistence::mappers::cliente as mapper;
use crate::persistence::models::cliente::{self as model, Column, Entity};
use crate::persistence::models::{cliente_contacto, factura, movimiento, obra, pago_factura};

use super::estado_deuda_ids;

const ENTITY: &str = "Cliente";
const ENTITY_CONTACTO: &str = "ClienteContacto";

pub struct SeaOrmClienteRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmClienteRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }

    async fn contactos_de(&self, cliente_id: Uuid) -> AppResult<Vec<ClienteContacto>> {
        let rows = cliente_contacto::Entity::find()
            .filter(cliente_contacto::Column::ClienteId.eq(cliente_id.to_string()))
            .filter(cliente_contacto::Column::IsDeleted.eq(false))
            // The main contact leads, then alphabetically: the form shows them in this order and
            // so does every place that prints the first one.
            .order_by_desc(cliente_contacto::Column::EsPrincipal)
            .order_by_asc(cliente_contacto::Column::Etiqueta)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::contacto_to_domain).collect()
    }
}

#[derive(Debug, FromQueryResult)]
struct RowConResumen {
    id: String,
    nombre: String,
    cuit: Option<String>,
    direccion: Option<String>,
    telefono: Option<String>,
    email: Option<String>,
    condicion_iva: Option<String>,
    created_at: String,
    updated_at: Option<String>,
    row_version: Vec<u8>,
    is_deleted: bool,
    deleted_at: Option<String>,
    obras_count: i64,
    facturas_count: i64,
    deuda: i64,
}

impl TryFrom<RowConResumen> for ClienteConResumen {
    type Error = AppError;

    fn try_from(row: RowConResumen) -> Result<Self, Self::Error> {
        let model = model::Model {
            id: row.id,
            nombre: row.nombre,
            cuit: row.cuit,
            direccion: row.direccion,
            telefono: row.telefono,
            email: row.email,
            condicion_iva: row.condicion_iva,
            created_at: row.created_at,
            updated_at: row.updated_at,
            row_version: row.row_version,
            is_deleted: row.is_deleted,
            deleted_at: row.deleted_at,
        };
        Ok(Self {
            cliente: mapper::to_domain(model)?,
            obras_count: row.obras_count.max(0) as u64,
            facturas_count: row.facturas_count.max(0) as u64,
            deuda: Money::from_raw(row.deuda),
        })
    }
}

fn alive() -> Condition {
    Condition::all().add(Column::IsDeleted.eq(false))
}

fn lower(column: Column) -> SimpleExpr {
    Func::lower(Expr::col((Entity, column))).into()
}

fn filtro_condition(filtro: &ClienteFiltro) -> Condition {
    let mut c = alive();
    if let Some(texto) = filtro.texto.as_deref() {
        let patron = format!("%{}%", texto.trim().to_lowercase());
        c = c.add(
            Condition::any()
                .add(lower(Column::Nombre).like(patron.clone()))
                .add(lower(Column::Cuit).like(patron.clone()))
                .add(lower(Column::Email).like(patron)),
        );
    }
    if let Some(condicion) = filtro.condicion_iva.as_deref() {
        c = c.add(Column::CondicionIva.eq(condicion));
    }
    if filtro.solo_con_deuda {
        c = c.add(deuda_expr().gt(0i64));
    }
    c
}

fn obras_count_expr() -> SimpleExpr {
    SimpleExpr::SubQuery(
        None,
        Box::new(
            Query::select()
                .expr(Expr::col(obra::Column::Id).count())
                .from(obra::Entity)
                .and_where(Expr::col((obra::Entity, obra::Column::ClienteId)).equals((
                    Entity,
                    Column::Id,
                )))
                .and_where(Expr::col((obra::Entity, obra::Column::IsDeleted)).eq(false))
                .take()
                .into_sub_query_statement(),
        ),
    )
}

fn facturas_count_expr() -> SimpleExpr {
    SimpleExpr::SubQuery(
        None,
        Box::new(
            Query::select()
                .expr(Expr::col(factura::Column::Id).count())
                .from(factura::Entity)
                .and_where(Expr::col((factura::Entity, factura::Column::ClienteId)).equals((
                    Entity,
                    Column::Id,
                )))
                .and_where(Expr::col((factura::Entity, factura::Column::IsDeleted)).eq(false))
                .take()
                .into_sub_query_statement(),
        ),
    )
}

/// Sum of `total - paid` over the invoices of the customer that count as debt.
///
/// It is computed in SQL rather than in Rust because the column is sortable and filterable: paging
/// on a figure the database cannot see would return the wrong rows.
pub(super) fn deuda_expr() -> SimpleExpr {
    let pagado = SimpleExpr::SubQuery(
        None,
        Box::new(
            Query::select()
                .expr(Func::coalesce([
                    Func::sum(Expr::col(pago_factura::Column::Monto)).into(),
                    Expr::value(0i64),
                ]))
                .from(pago_factura::Entity)
                .and_where(
                    Expr::col((pago_factura::Entity, pago_factura::Column::FacturaId))
                        .equals((factura::Entity, factura::Column::Id)),
                )
                .and_where(
                    Expr::col((pago_factura::Entity, pago_factura::Column::IsDeleted)).eq(false),
                )
                .take()
                .into_sub_query_statement(),
        ),
    );

    Func::coalesce([
        SimpleExpr::SubQuery(
            None,
            Box::new(
                Query::select()
                    .expr(Func::sum(
                        Expr::col((factura::Entity, factura::Column::Total)).sub(pagado),
                    ))
                    .from(factura::Entity)
                    .and_where(Expr::col((factura::Entity, factura::Column::ClienteId)).equals((
                        Entity,
                        Column::Id,
                    )))
                    .and_where(Expr::col((factura::Entity, factura::Column::IsDeleted)).eq(false))
                    .and_where(
                        Expr::col((factura::Entity, factura::Column::Estado))
                            .is_in(estado_deuda_ids()),
                    )
                    .take()
                    .into_sub_query_statement(),
            ),
        ),
        Expr::value(0i64),
    ])
    .into()
}

#[async_trait]
impl ClienteRepository for SeaOrmClienteRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Cliente>> {
        let found = Entity::find_by_id(id.to_string())
            .filter(alive())
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(mapper::to_domain).transpose()
    }

    async fn find_con_contactos(&self, id: Uuid) -> AppResult<Option<Cliente>> {
        let Some(mut cliente) = self.find_by_id(id).await? else {
            return Ok(None);
        };
        cliente.contactos = self.contactos_de(id).await?;
        Ok(Some(cliente))
    }

    async fn search(
        &self,
        filtro: &ClienteFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<ClienteConResumen>> {
        let condition = filtro_condition(filtro);
        let order = match sort_dir {
            SortDir::Asc => Order::Asc,
            SortDir::Desc => Order::Desc,
        };

        let mut query = Entity::find()
            .filter(condition.clone())
            .column_as(obras_count_expr(), "obras_count")
            .column_as(facturas_count_expr(), "facturas_count")
            .column_as(deuda_expr(), "deuda");

        query = match sort_by {
            Some("cuit") => query.order_by(lower(Column::Cuit), order),
            Some("deuda") => query.order_by(Expr::col(Alias::new("deuda")), order),
            Some("obrasCount") => query.order_by(Expr::col(Alias::new("obras_count")), order),
            Some("facturasCount") => query.order_by(Expr::col(Alias::new("facturas_count")), order),
            Some("createdAt") => query.order_by(Column::CreatedAt, order),
            _ => query.order_by(lower(Column::Nombre), order),
        }
        .order_by_asc(Column::Id);

        let total = Entity::find()
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
            .map(ClienteConResumen::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PagedResult::new(items, total, page))
    }

    async fn lookup(&self, texto: Option<&str>, limite: u64) -> AppResult<Vec<Cliente>> {
        let filtro = ClienteFiltro {
            texto: texto.map(str::to_owned),
            ..ClienteFiltro::default()
        };
        let rows = Entity::find()
            .filter(filtro_condition(&filtro))
            .order_by_asc(lower(Column::Nombre))
            .limit(limite)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::to_domain).collect()
    }

    async fn insert(&self, entity: &Cliente) -> AppResult<()> {
        Entity::insert(mapper::to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &Cliente, esperado: RowVersion) -> AppResult<()> {
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

    async fn insert_contacto(&self, entity: &ClienteContacto) -> AppResult<()> {
        cliente_contacto::Entity::insert(mapper::contacto_to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    /// No row version here: a contact is part of the customer aggregate and the caller already
    /// checked the parent's version, so a second check would only add a way to fail.
    async fn update_contacto(&self, entity: &ClienteContacto) -> AppResult<()> {
        let result = cliente_contacto::Entity::update_many()
            .set(mapper::contacto_to_active(entity))
            .filter(cliente_contacto::Column::Id.eq(entity.id.to_string()))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if result.rows_affected == 0 {
            return Err(AppError::not_found(ENTITY_CONTACTO, entity.id));
        }
        Ok(())
    }

    async fn soft_delete_contactos_excepto(
        &self,
        cliente_id: Uuid,
        conservar: &[Uuid],
        at: DateTime<Utc>,
    ) -> AppResult<()> {
        let mut condition = Condition::all()
            .add(cliente_contacto::Column::ClienteId.eq(cliente_id.to_string()))
            .add(cliente_contacto::Column::IsDeleted.eq(false));
        if !conservar.is_empty() {
            condition = condition.add(
                cliente_contacto::Column::Id
                    .is_not_in(conservar.iter().map(Uuid::to_string).collect::<Vec<_>>()),
            );
        }

        cliente_contacto::Entity::update_many()
            .col_expr(cliente_contacto::Column::IsDeleted, Expr::value(true))
            .col_expr(
                cliente_contacto::Column::DeletedAt,
                Expr::value(time::to_storage(at)),
            )
            .col_expr(
                cliente_contacto::Column::UpdatedAt,
                Expr::value(time::to_storage(at)),
            )
            .filter(condition)
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn count_obras(&self, id: Uuid) -> AppResult<u64> {
        obra::Entity::find()
            .filter(obra::Column::ClienteId.eq(id.to_string()))
            .filter(obra::Column::IsDeleted.eq(false))
            .count(self.conn())
            .await
            .map_err(AppError::persistence)
    }

    async fn count_facturas(&self, id: Uuid) -> AppResult<u64> {
        factura::Entity::find()
            .filter(factura::Column::ClienteId.eq(id.to_string()))
            .filter(factura::Column::IsDeleted.eq(false))
            .count(self.conn())
            .await
            .map_err(AppError::persistence)
    }

    async fn count_movimientos(&self, id: Uuid) -> AppResult<u64> {
        movimiento::Entity::find()
            .filter(movimiento::Column::ClienteId.eq(id.to_string()))
            .filter(movimiento::Column::IsDeleted.eq(false))
            .count(self.conn())
            .await
            .map_err(AppError::persistence)
    }
}
