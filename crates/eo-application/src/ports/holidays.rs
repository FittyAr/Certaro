//! Holiday calendar of a third party. See `docs/13-servicios-externos-y-archivos.md` §3.

use async_trait::async_trait;
use eo_domain::entities::Feriado;

use crate::result::AppResult;

#[async_trait]
pub trait HolidayProvider: Send + Sync {
    /// The holidays of one year. An unreachable service is not fatal: the caller degrades to an
    /// empty calendar and warns, because a settlement is still better than no settlement.
    async fn fetch(&self, anio: i32) -> AppResult<Vec<Feriado>>;
}
