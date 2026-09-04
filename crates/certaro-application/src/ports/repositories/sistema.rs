use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;
use certaro_domain::entities::*;
use certaro_domain::{Decimal4, EstadoFactura, EstadoProyecto, EstadoTrabajo, Moneda, Money, RowVersion};
use crate::paging::{PageRequest, PagedResult};
use crate::result::AppResult;
use super::common::*;

#[async_trait]
pub trait AdjuntoRepository: Send + Sync {
    /// The live attachments of one record, newest first.
    async fn de_entidad(
        &self,
        entidad_tipo: EntidadAdjunto,
        entidad_id: Uuid,
    ) -> AppResult<Vec<Adjunto>>;

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Adjunto>>;

    /// How many attachments each of those records has, for the badge on a listing. Records with
    /// none are simply absent from the result.
    async fn count_de(
        &self,
        entidad_tipo: EntidadAdjunto,
        entidad_ids: &[Uuid],
    ) -> AppResult<Vec<(Uuid, u64)>>;

    async fn insert(&self, entity: &Adjunto) -> AppResult<()>;

    /// Soft delete. The file itself goes to the trash, which is the store's job.
    async fn soft_delete(&self, id: Uuid, at: DateTime<Utc>) -> AppResult<()>;
}

#[async_trait]
pub trait FeriadoRepository: Send + Sync {
    /// The holidays of a closed range of civil dates. The settlement reads the table, never the
    /// network.
    async fn del_rango(&self, desde: NaiveDate, hasta: NaiveDate) -> AppResult<Vec<Feriado>>;
    async fn del_anio(&self, anio: i32) -> AppResult<Vec<Feriado>>;
    async fn count_anio(&self, anio: i32) -> AppResult<u64>;

    /// Inserts what is missing and leaves every existing row alone, so a hand-added holiday is
    /// never overwritten by a sync. Returns how many rows were added.
    async fn insertar_faltantes(&self, feriados: &[Feriado]) -> AppResult<u64>;

    /// Upsert of a hand-added holiday.
    async fn upsert_manual(&self, entity: &Feriado) -> AppResult<()>;

    /// A real delete: a holiday left behind would keep paying its multiplier.
    async fn delete(&self, fecha: NaiveDate) -> AppResult<()>;
}

/// A total with the name it is grouped by, for the top-customers and top-categories rankings.

/// The internal key/value store. Not a business record: no audit block, no soft delete.
#[async_trait]
pub trait MetadataRepository: Send + Sync {
    /// The value and when it was written, which is what a cache needs to know if it expired.
    async fn get(&self, key: &str) -> AppResult<Option<(String, DateTime<Utc>)>>;
    async fn set(&self, key: &str, value: &str, at: DateTime<Utc>) -> AppResult<()>;
}
