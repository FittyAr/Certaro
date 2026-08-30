use std::sync::Arc;

use async_trait::async_trait;
use chrono::NaiveDate;
use eo_application::ports::repositories::FeriadoRepository;
use eo_application::{AppError, AppResult};
use eo_domain::entities::Feriado;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ColumnTrait, DatabaseTransaction, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};

use crate::persistence::mappers::{civil_to_storage, feriado as mapper};
use crate::persistence::models::feriado::{Column, Entity};

pub struct SeaOrmFeriadoRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmFeriadoRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

/// The bounds of a year as civil dates. Lexicographic order on `YYYY-MM-DD` is chronological, so
/// the same range comparison works for the year and for an arbitrary period.
fn limites(anio: i32) -> (String, String) {
    (format!("{anio:04}-01-01"), format!("{anio:04}-12-31"))
}

#[async_trait]
impl FeriadoRepository for SeaOrmFeriadoRepository {
    async fn del_rango(&self, desde: NaiveDate, hasta: NaiveDate) -> AppResult<Vec<Feriado>> {
        let rows = Entity::find()
            .filter(Column::Fecha.gte(civil_to_storage(desde)))
            .filter(Column::Fecha.lte(civil_to_storage(hasta)))
            .order_by_asc(Column::Fecha)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::to_domain).collect()
    }

    async fn del_anio(&self, anio: i32) -> AppResult<Vec<Feriado>> {
        let (desde, hasta) = limites(anio);
        let rows = Entity::find()
            .filter(Column::Fecha.gte(desde))
            .filter(Column::Fecha.lte(hasta))
            .order_by_asc(Column::Fecha)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::to_domain).collect()
    }

    async fn count_anio(&self, anio: i32) -> AppResult<u64> {
        let (desde, hasta) = limites(anio);
        Entity::find()
            .filter(Column::Fecha.gte(desde))
            .filter(Column::Fecha.lte(hasta))
            .count(self.conn())
            .await
            .map_err(AppError::persistence)
    }

    async fn insertar_faltantes(&self, feriados: &[Feriado]) -> AppResult<u64> {
        if feriados.is_empty() {
            return Ok(0);
        }

        let antes = Entity::find()
            .count(self.conn())
            .await
            .map_err(AppError::persistence)?;

        // `DO NOTHING` and not an update: a sync must never overwrite a hand-added holiday, and the
        // rows the API returns are identical anyway.
        Entity::insert_many(feriados.iter().map(mapper::to_active))
            .on_conflict(OnConflict::column(Column::Fecha).do_nothing().to_owned())
            .do_nothing()
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;

        let despues = Entity::find()
            .count(self.conn())
            .await
            .map_err(AppError::persistence)?;

        Ok(despues.saturating_sub(antes))
    }

    async fn upsert_manual(&self, entity: &Feriado) -> AppResult<()> {
        Entity::insert(mapper::to_active(entity))
            .on_conflict(
                OnConflict::column(Column::Fecha)
                    .update_columns([
                        Column::Nombre,
                        Column::Tipo,
                        Column::Origen,
                        Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn delete(&self, fecha: NaiveDate) -> AppResult<()> {
        Entity::delete_many()
            .filter(Column::Fecha.eq(civil_to_storage(fecha)))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }
}
