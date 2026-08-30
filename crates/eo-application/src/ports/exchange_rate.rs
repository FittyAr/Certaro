//! Port of the dollar quote service. See `docs/13-servicios-externos-y-archivos.md` §2.1.

use async_trait::async_trait;

use crate::dtos::cotizaciones::Cotizacion;
use crate::result::AppResult;

#[async_trait]
pub trait ExchangeRateProvider: Send + Sync {
    /// Every house the service publishes, unfiltered: deciding which ones to show is a
    /// configuration matter and belongs to the use case, not to the adapter.
    async fn fetch(&self) -> AppResult<Vec<Cotizacion>>;
}
