//! Use cases of the dollar quotes. See `docs/13-servicios-externos-y-archivos.md` §2.
//!
//! Three rules govern this module, and all three exist so that a service being down never becomes
//! the user's problem: a failure degrades to the cache, an empty cache degrades to an empty list,
//! and no path returns an error to the screen.

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use tracing::{info, warn};

use crate::dtos::cotizaciones::Cotizacion;
use crate::ports::repositories::UnitOfWork;
use crate::ports::{ClockPort, ExchangeRateProvider, SettingsStore};
use crate::result::AppResult;
use crate::use_cases::shared::{finish_read, finish_write};

/// Key of the cached payload in `app_metadata`.
const CACHE_KEY: &str = "CotizacionesCache";

pub struct CotizacionesService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    provider: Arc<dyn ExchangeRateProvider>,
    settings: Arc<dyn SettingsStore>,
}

impl CotizacionesService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        provider: Arc<dyn ExchangeRateProvider>,
        settings: Arc<dyn SettingsStore>,
    ) -> Self {
        Self {
            uow,
            clock,
            provider,
            settings,
        }
    }

    /// The visible houses, from the cache while it is fresh and from the network otherwise.
    ///
    /// A stale cache is still served, flagged as such, when the request fails: an old number the
    /// user can see the date of beats an empty status bar.
    pub async fn list(&self) -> AppResult<Vec<Cotizacion>> {
        self.obtener(false).await
    }

    /// Forces a request, ignoring the cache. This is what the refresh button calls.
    pub async fn refresh(&self) -> AppResult<Vec<Cotizacion>> {
        self.obtener(true).await
    }

    async fn obtener(&self, forzar: bool) -> AppResult<Vec<Cotizacion>> {
        let config = self.settings.snapshot();
        let ahora = self.clock.now_utc();
        let ttl = Duration::minutes(i64::from(config.external_apis.dollar_cache_minutes));

        let cache = self.leer_cache().await?;
        if !forzar {
            if let Some((cotizaciones, escrito)) = cache.clone() {
                if ahora.signed_duration_since(escrito) < ttl {
                    return Ok(self.visibles(cotizaciones, &config.dashboard.casas_dolar));
                }
            }
        }

        match self.provider.fetch().await {
            Ok(cotizaciones) if !cotizaciones.is_empty() => {
                self.guardar_cache(&cotizaciones, ahora).await?;
                Ok(self.visibles(cotizaciones, &config.dashboard.casas_dolar))
            }
            // An empty successful response is treated as a failure: it carries no information and
            // overwriting a good cache with it would lose the last known rate.
            resultado => {
                if let Err(e) = resultado {
                    warn!(error = %e, "no se pudieron traer las cotizaciones");
                }
                match cache {
                    Some((cotizaciones, escrito)) => {
                        info!(%escrito, "se sirven cotizaciones de la caché");
                        let desactualizadas = cotizaciones
                            .into_iter()
                            .map(|c| Cotizacion {
                                desactualizada: true,
                                ..c
                            })
                            .collect();
                        Ok(self.visibles(desactualizadas, &config.dashboard.casas_dolar))
                    }
                    None => Ok(Vec::new()),
                }
            }
        }
    }

    /// Keeps only the configured houses, in the order they were configured, so the status bar does
    /// not reshuffle itself when the API changes the order of its array.
    fn visibles(&self, cotizaciones: Vec<Cotizacion>, casas: &[String]) -> Vec<Cotizacion> {
        casas
            .iter()
            .filter_map(|casa| {
                let casa = casa.trim().to_lowercase();
                cotizaciones
                    .iter()
                    .find(|c| c.casa.to_lowercase() == casa)
                    .cloned()
            })
            .collect()
    }

    async fn leer_cache(&self) -> AppResult<Option<(Vec<Cotizacion>, DateTime<Utc>)>> {
        let tx = self.uow.begin().await?;
        let result = tx.metadata().get(CACHE_KEY).await;
        let entrada = finish_read(tx, result).await?;

        Ok(entrada.and_then(|(value, escrito)| {
            match serde_json::from_str::<Vec<Cotizacion>>(&value) {
                Ok(cotizaciones) => Some((cotizaciones, escrito)),
                // A cache written by an older version is discarded, not fatal.
                Err(e) => {
                    warn!(error = %e, "la caché de cotizaciones no se pudo leer");
                    None
                }
            }
        }))
    }

    async fn guardar_cache(&self, cotizaciones: &[Cotizacion], at: DateTime<Utc>) -> AppResult<()> {
        let value = match serde_json::to_string(cotizaciones) {
            Ok(value) => value,
            Err(e) => {
                warn!(error = %e, "la caché de cotizaciones no se pudo serializar");
                return Ok(());
            }
        };

        let tx = self.uow.begin().await?;
        let outcome = tx.metadata().set(CACHE_KEY, &value, at).await;
        finish_write(tx, outcome).await
    }
}
