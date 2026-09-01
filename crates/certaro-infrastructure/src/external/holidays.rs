//! Holiday calendar over HTTP. See `docs/13-servicios-externos-y-archivos.md` §3.

use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use certaro_application::config::ExternalApisConfig;
use certaro_application::ports::holidays::HolidayProvider;
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::{Feriado, OrigenFeriado};
use certaro_domain::time;
use serde::Deserialize;
use tracing::warn;

const SERVICE: &str = "holidays";

pub struct HttpHolidayProvider {
    client: reqwest::Client,
    base_url: String,
    reintentos: u8,
}

#[derive(Debug, Deserialize)]
struct FeriadoResponse {
    fecha: String,
    nombre: String,
    #[serde(default)]
    tipo: Option<String>,
}

impl HttpHolidayProvider {
    /// # Errors
    /// When the HTTP client cannot be built, which only happens if the TLS backend is unusable.
    pub fn new(config: &ExternalApisConfig) -> AppResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(u64::from(config.timeout_seconds)))
            .build()
            .map_err(|e| AppError::unexpected(anyhow::anyhow!("http client: {e}")))?;

        Ok(Self {
            client,
            // Normalised the same way the legacy system did, so a base configured without the
            // trailing slash does not turn `/feriados/2026` into `/feriados2026`.
            base_url: normalizar_base(&config.holiday_url),
            reintentos: config.reintentos,
        })
    }

    async fn intentar(&self, url: &str) -> Result<Vec<FeriadoResponse>, reqwest::Error> {
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<FeriadoResponse>>()
            .await
    }
}

fn normalizar_base(raw: &str) -> String {
    let base = raw.trim();
    if base.ends_with('/') {
        base.to_owned()
    } else {
        format!("{base}/")
    }
}

#[async_trait]
impl HolidayProvider for HttpHolidayProvider {
    async fn fetch(&self, anio: i32) -> AppResult<Vec<Feriado>> {
        let url = format!("{}{anio}", self.base_url);
        let intentos = u32::from(self.reintentos).saturating_add(1);
        let mut ultimo: Option<reqwest::Error> = None;

        for intento in 1..=intentos {
            match self.intentar(&url).await {
                Ok(payload) => {
                    let now = Utc::now();
                    // An unparseable date is dropped rather than failing the year: one bad element
                    // must not cost the whole calendar.
                    let feriados = payload
                        .into_iter()
                        .filter_map(|item| match time::parse_civil(&item.fecha) {
                            Ok(fecha) => Some(Feriado {
                                fecha,
                                nombre: item.nombre,
                                tipo: item.tipo,
                                origen: OrigenFeriado::Api,
                                created_at: now,
                                updated_at: None,
                            }),
                            Err(e) => {
                                warn!(
                                    service = SERVICE,
                                    fecha = %item.fecha,
                                    error = %e,
                                    "discarded holiday with an unparseable date"
                                );
                                None
                            }
                        })
                        .collect();
                    return Ok(feriados);
                }
                Err(e) => {
                    warn!(
                        service = SERVICE,
                        url = %url,
                        intento,
                        de = intentos,
                        error = %e,
                        "holiday request failed"
                    );
                    ultimo = Some(e);
                }
            }
        }

        warn!(
            service = SERVICE,
            error = ultimo.map(|e| e.to_string()).unwrap_or_default(),
            "holiday calendar unavailable after every retry"
        );
        Err(AppError::ExternalUnavailable { service: SERVICE })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_base_se_normaliza_con_barra_final() {
        assert_eq!(
            normalizar_base("https://api.example.com/v1/feriados"),
            "https://api.example.com/v1/feriados/"
        );
        assert_eq!(
            normalizar_base("https://api.example.com/v1/feriados/"),
            "https://api.example.com/v1/feriados/"
        );
    }

    #[test]
    fn la_url_del_anio_se_arma_concatenando() {
        let config = ExternalApisConfig::default();
        let provider = HttpHolidayProvider::new(&config).unwrap();
        assert_eq!(
            format!("{}{}", provider.base_url, 2026),
            "https://api.argentinadatos.com/v1/feriados/2026"
        );
    }
}
