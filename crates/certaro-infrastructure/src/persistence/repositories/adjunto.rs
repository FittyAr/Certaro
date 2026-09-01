use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use certaro_application::ports::repositories::AdjuntoRepository;
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::{Adjunto, EntidadAdjunto};
use certaro_domain::time;
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use crate::persistence::mappers::adjunto as mapper;
use crate::persistence::models::adjunto::{Column, Entity};

const ENTITY: &str = "Adjunto";

pub struct SeaOrmAdjuntoRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmAdjuntoRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[async_trait]
impl AdjuntoRepository for SeaOrmAdjuntoRepository {
    async fn de_entidad(
        &self,
        entidad_tipo: EntidadAdjunto,
        entidad_id: Uuid,
    ) -> AppResult<Vec<Adjunto>> {
        let rows = Entity::find()
            .filter(Column::IsDeleted.eq(false))
            .filter(Column::EntidadTipo.eq(entidad_tipo.as_str()))
            .filter(Column::EntidadId.eq(entidad_id.to_string()))
            // Newest first: the file just attached is the one being looked for.
            .order_by_desc(Column::CreatedAt)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::to_domain).collect()
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Adjunto>> {
        let row = Entity::find()
            .filter(Column::IsDeleted.eq(false))
            .filter(Column::Id.eq(id.to_string()))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        row.map(mapper::to_domain).transpose()
    }

    async fn count_de(
        &self,
        entidad_tipo: EntidadAdjunto,
        entidad_ids: &[Uuid],
    ) -> AppResult<Vec<(Uuid, u64)>> {
        if entidad_ids.is_empty() {
            return Ok(Vec::new());
        }
        // Counted in the application rather than with a `GROUP BY`: the identifier list is one page
        // of rows at most, and this keeps the mapping of the polymorphic key in one place.
        let rows = Entity::find()
            .filter(Column::IsDeleted.eq(false))
            .filter(Column::EntidadTipo.eq(entidad_tipo.as_str()))
            .filter(
                Column::EntidadId.is_in(
                    entidad_ids
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>(),
                ),
            )
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        let mut conteos: Vec<(Uuid, u64)> = Vec::new();
        for row in rows {
            let id = crate::persistence::mappers::uuid(&row.entidad_id)?;
            match conteos.iter_mut().find(|(existente, _)| *existente == id) {
                Some((_, count)) => *count += 1,
                None => conteos.push((id, 1)),
            }
        }
        Ok(conteos)
    }

    async fn insert(&self, entity: &Adjunto) -> AppResult<()> {
        Entity::insert(mapper::to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn soft_delete(&self, id: Uuid, at: DateTime<Utc>) -> AppResult<()> {
        // No expected row version: an attachment has no editable field, so there is no concurrent
        // edit to lose. Deleting one that is already deleted is not an error either.
        let result = Entity::update_many()
            .col_expr(Column::IsDeleted, Expr::value(true))
            .col_expr(Column::DeletedAt, Expr::value(time::to_storage(at)))
            .col_expr(Column::UpdatedAt, Expr::value(time::to_storage(at)))
            .filter(Column::Id.eq(id.to_string()))
            .filter(Column::IsDeleted.eq(false))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if result.rows_affected == 0 {
            return Err(AppError::not_found(ENTITY, id));
        }
        Ok(())
    }
}
