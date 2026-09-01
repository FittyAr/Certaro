use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use certaro_application::ports::repositories::{ObraConResumen, ObraFiltro, ObraRepository, SortDir};
use certaro_application::{AppError, AppResult, PageRequest, PagedResult};
use certaro_domain::entities::{Obra, Trabajo};
use certaro_domain::{time, EstadoObra, EstadoTrabajo, Money, RowVersion};
use sea_orm::sea_query::{Alias, Expr, Func, Query, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, FromQueryResult, JoinType, Order,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::persistence::mappers::obra as mapper;
use crate::persistence::mappers::trabajo as trabajo_mapper;
use crate::persistence::models::obra::{self as model, Column, Entity};
use crate::persistence::models::{cliente, movimiento, tipo_movimiento, trabajo};

const ENTITY: &str = "Obra";

pub struct SeaOrmObraRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmObraRepository {
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
    numero: i32,
    nombre: String,
    direccion: Option<String>,
    localidad: Option<String>,
    cliente_id: String,
    estado: i32,
    created_at: String,
    updated_at: Option<String>,
    row_version: Vec<u8>,
    is_deleted: bool,
    deleted_at: Option<String>,
    cliente_nombre: String,
    trabajos_count: i64,
    rentabilidad: i64,
}

impl TryFrom<RowConResumen> for ObraConResumen {
    type Error = AppError;

    fn try_from(row: RowConResumen) -> Result<Self, Self::Error> {
        let model = model::Model {
            id: row.id,
            numero: row.numero,
            nombre: row.nombre,
            direccion: row.direccion,
            localidad: row.localidad,
            cliente_id: row.cliente_id,
            estado: row.estado,
            created_at: row.created_at,
            updated_at: row.updated_at,
            row_version: row.row_version,
            is_deleted: row.is_deleted,
            deleted_at: row.deleted_at,
        };
        Ok(Self {
            obra: mapper::to_domain(model)?,
            cliente_nombre: row.cliente_nombre,
            trabajos_count: row.trabajos_count.max(0) as u64,
            // The subquery already sums the product of two scaled values, so it comes back at
            // scale 10^8 and is narrowed here.
            rentabilidad: Money::from_product_sum(i128::from(row.rentabilidad))
                .map_err(AppError::from)?,
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

fn filtro_condition(filtro: &ObraFiltro) -> Condition {
    let mut c = alive();
    if let Some(texto) = filtro.texto.as_deref() {
        let limpio = texto.trim();
        let patron = format!("%{}%", limpio.to_lowercase());
        let mut any = Condition::any()
            .add(lower(Column::Nombre).like(patron.clone()))
            .add(lower(Column::Direccion).like(patron.clone()))
            .add(lower(Column::Localidad).like(patron));
        // The number is an integer, so it is matched exactly when the text is one instead of
        // being cast to text on every row.
        if let Ok(numero) = limpio.parse::<i32>() {
            any = any.add(Column::Numero.eq(numero));
        }
        c = c.add(any);
    }
    if let Some(id) = filtro.cliente_id {
        c = c.add(Column::ClienteId.eq(id.to_string()));
    }
    if let Some(estado) = filtro.estado {
        c = c.add(Column::Estado.eq(estado.as_i32()));
    }
    if filtro.solo_activas {
        // "Active" here means "still going on", which includes a paused site: the shorthand
        // exists to hide what is finished or cancelled, not to hide what is on hold.
        c = c
            .add(Column::Estado.is_in([EstadoObra::Activa.as_i32(), EstadoObra::Pausada.as_i32()]));
    }
    c
}

fn trabajos_count_expr() -> SimpleExpr {
    SimpleExpr::SubQuery(
        None,
        Box::new(
            Query::select()
                .expr(Expr::col(trabajo::Column::Id).count())
                .from(trabajo::Entity)
                .and_where(
                    Expr::col((trabajo::Entity, trabajo::Column::ObraId))
                        .equals((Entity, Column::Id)),
                )
                .and_where(Expr::col((trabajo::Entity, trabajo::Column::IsDeleted)).eq(false))
                .take()
                .into_sub_query_statement(),
        ),
    )
}

/// Sum of the movements imputed through one of the site's jobs, restricted to one sign.
fn suma_movimientos_expr(es_ingreso: bool) -> SimpleExpr {
    Func::coalesce([
        SimpleExpr::SubQuery(
            None,
            Box::new(
                Query::select()
                    .expr(Func::sum(
                        Expr::col((movimiento::Entity, movimiento::Column::Monto)).mul(Expr::col(
                            (movimiento::Entity, movimiento::Column::Cantidad),
                        )),
                    ))
                    .from(movimiento::Entity)
                    .inner_join(
                        trabajo::Entity,
                        Expr::col((trabajo::Entity, trabajo::Column::Id))
                            .equals((movimiento::Entity, movimiento::Column::TrabajoId)),
                    )
                    .inner_join(
                        tipo_movimiento::Entity,
                        Expr::col((tipo_movimiento::Entity, tipo_movimiento::Column::Id))
                            .equals((movimiento::Entity, movimiento::Column::TipoMovimientoId)),
                    )
                    .and_where(
                        Expr::col((trabajo::Entity, trabajo::Column::ObraId))
                            .equals((Entity, Column::Id)),
                    )
                    .and_where(
                        Expr::col((tipo_movimiento::Entity, tipo_movimiento::Column::EsIngreso))
                            .eq(es_ingreso),
                    )
                    .and_where(
                        Expr::col((movimiento::Entity, movimiento::Column::IsDeleted)).eq(false),
                    )
                    .and_where(Expr::col((trabajo::Entity, trabajo::Column::IsDeleted)).eq(false))
                    .take()
                    .into_sub_query_statement(),
            ),
        ),
        Expr::value(0i64),
    ])
    .into()
}

/// Income minus expenses of every movement imputed through one of the site's jobs.
///
/// The sign lives in the movement type, which is a joined table, so it is resolved as the
/// difference of two restricted sums rather than a `CASE` over a column this query does not own.
fn rentabilidad_expr() -> SimpleExpr {
    suma_movimientos_expr(true).sub(suma_movimientos_expr(false))
}

fn base_query() -> sea_orm::Select<Entity> {
    Entity::find()
        .join(JoinType::InnerJoin, cliente_join())
        .column_as(
            Expr::col((cliente::Entity, cliente::Column::Nombre)),
            "cliente_nombre",
        )
        .column_as(trabajos_count_expr(), "trabajos_count")
        .column_as(rentabilidad_expr(), "rentabilidad")
}

#[async_trait]
impl ObraRepository for SeaOrmObraRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Obra>> {
        let found = Entity::find_by_id(id.to_string())
            .filter(alive())
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(mapper::to_domain).transpose()
    }

    async fn find_detalle(&self, id: Uuid) -> AppResult<Option<ObraConResumen>> {
        let found = base_query()
            .filter(alive())
            .filter(Column::Id.eq(id.to_string()))
            .into_model::<RowConResumen>()
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(ObraConResumen::try_from).transpose()
    }

    async fn search(
        &self,
        filtro: &ObraFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<ObraConResumen>> {
        let condition = filtro_condition(filtro);
        let order = match sort_dir {
            SortDir::Asc => Order::Asc,
            SortDir::Desc => Order::Desc,
        };

        let mut query = base_query().filter(condition.clone());

        query = match sort_by {
            Some("nombre") => query.order_by(lower(Column::Nombre), order),
            Some("estado") => query.order_by(Column::Estado, order),
            Some("clienteNombre") => query.order_by(
                SimpleExpr::from(Func::lower(Expr::col((
                    cliente::Entity,
                    cliente::Column::Nombre,
                )))),
                order,
            ),
            Some("trabajosCount") => query.order_by(Expr::col(Alias::new("trabajos_count")), order),
            Some("rentabilidad") => query.order_by(Expr::col(Alias::new("rentabilidad")), order),
            Some("createdAt") => query.order_by(Column::CreatedAt, order),
            // The number is the identifier the user reads out loud, and the newest site is the
            // one being worked on, so the default is the highest first.
            _ => query.order_by(
                Column::Numero,
                match sort_by {
                    None => Order::Desc,
                    Some(_) => order,
                },
            ),
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
            .map(ObraConResumen::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PagedResult::new(items, total, page))
    }

    async fn lookup(
        &self,
        cliente_id: Option<Uuid>,
        texto: Option<&str>,
        limite: u64,
    ) -> AppResult<Vec<Obra>> {
        let filtro = ObraFiltro {
            texto: texto.map(str::to_owned),
            cliente_id,
            ..ObraFiltro::default()
        };
        let rows = Entity::find()
            .filter(filtro_condition(&filtro))
            .order_by_desc(Column::Numero)
            .limit(limite)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::to_domain).collect()
    }

    /// Deleted sites count on purpose: the number stays reserved (INV-06) and the unique index is
    /// not filtered by `is_deleted`, so ignoring them here would produce a constraint violation
    /// instead of a validation message.
    async fn numero_ocupado(&self, numero: i32, excluir: Option<Uuid>) -> AppResult<bool> {
        let mut condition = Condition::all().add(Column::Numero.eq(numero));
        if let Some(id) = excluir {
            condition = condition.add(Column::Id.ne(id.to_string()));
        }
        let count = Entity::find()
            .filter(condition)
            .count(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(count > 0)
    }

    async fn siguiente_numero(&self) -> AppResult<i32> {
        #[derive(Debug, FromQueryResult)]
        struct MaxRow {
            maximo: Option<i32>,
        }

        let row = Entity::find()
            .select_only()
            .expr_as(Expr::col(Column::Numero).max(), "maximo")
            .into_model::<MaxRow>()
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;

        Ok(row.and_then(|r| r.maximo).unwrap_or(0) + 1)
    }

    async fn insert(&self, entity: &Obra) -> AppResult<()> {
        Entity::insert(mapper::to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &Obra, esperado: RowVersion) -> AppResult<()> {
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

    async fn count_trabajos(&self, id: Uuid) -> AppResult<u64> {
        trabajo::Entity::find()
            .filter(trabajo::Column::ObraId.eq(id.to_string()))
            .filter(trabajo::Column::IsDeleted.eq(false))
            .count(self.conn())
            .await
            .map_err(AppError::persistence)
    }

    async fn trabajos_abiertos(&self, id: Uuid) -> AppResult<Vec<Trabajo>> {
        let abiertos: Vec<i32> = EstadoTrabajo::ALL
            .iter()
            .filter(|e| e.esta_abierto())
            .map(|e| e.as_i32())
            .collect();

        let rows = trabajo::Entity::find()
            .filter(trabajo::Column::ObraId.eq(id.to_string()))
            .filter(trabajo::Column::IsDeleted.eq(false))
            .filter(trabajo::Column::Estado.is_in(abiertos))
            .order_by_asc(trabajo::Column::FechaInicio)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        rows.into_iter().map(trabajo_mapper::to_domain).collect()
    }
}
