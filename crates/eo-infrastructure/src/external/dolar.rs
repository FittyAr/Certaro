//! Dollar quotes over HTTP. See `docs/13-servicios-externos-y-archivos.md` §2.
//!
//! The amounts are read from the JSON as text and parsed into `Money`: going through `f64` would
//! turn `990.5` into something that is not quite `990.5`, and that value ends up copied into a
//! movement (doc 04 §1.3).

use std::time::Duration;

use async_trait::async_trait;
use eo_application::config::ExternalApisConfig;
use eo_application::dtos::cotizaciones::Cotizacion;
use eo_application::ports::exchange_rate::ExchangeRateProvider;
use eo_application::{AppError, AppResult};
use eo_domain::{time, Money};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use tracing::warn;

const SERVICE: &str = "dolar";

pub struct HttpExchangeRateProvider {
    client: reqwest::Client,
    url: String,
    reintentos: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CotizacionResponse {
    #[serde(default)]
    casa: String,
    #[serde(default)]
    nombre: String,
    /// Left as raw JSON so the number keeps the digits it was written with.
    compra: Option<JsonValue>,
    venta: Option<JsonValue>,
    fecha_actualizacion: Option<String>,
}

impl HttpExchangeRateProvider {
    /// # Errors
    /// When the HTTP client cannot be built, which only happens if the TLS backend is unusable.
    pub fn new(config: &ExternalApisConfig) -> AppResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(u64::from(config.timeout_seconds)))
            .build()
            .map_err(|e| AppError::unexpected(anyhow::anyhow!("http client: {e}")))?;

        Ok(Self {
            client,
            url: config.dollar_url.trim().to_owned(),
            reintentos: config.reintentos,
        })
    }

    async fn intentar(&self) -> Result<Vec<CotizacionResponse>, reqwest::Error> {
        self.client
            .get(&self.url)
            .header("Accept", "application/json")
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<CotizacionResponse>>()
            .await
    }
}

/// A JSON number or string as `Money`. Anything else is absent rather than zero: a quote of zero
/// would be applied to a movement as if it were real.
fn monto(value: Option<&JsonValue>) -> Option<Money> {
    match value {
        Some(JsonValue::Number(n)) => Money::parse(&n.to_string()).ok(),
        Some(JsonValue::String(s)) => Money::parse(s.trim()).ok(),
        _ => None,
    }
}

#[async_trait]
impl ExchangeRateProvider for HttpExchangeRateProvider {
    async fn fetch(&self) -> AppResult<Vec<Cotizacion>> {
        let intentos = u32::from(self.reintentos).saturating_add(1);
        let mut ultimo: Option<String> = None;

        for intento in 1..=intentos {
            match self.intentar().await {
                Ok(payload) => {
                    let cotizaciones = payload
                        .into_iter()
                        .filter_map(|item| {
                            let casa = item.casa.trim().to_lowercase();
                            // A house with an unreadable amount is dropped on its own: one bad
                            // element must not cost the rest of the list.
                            let (Some(compra), Some(venta)) =
                                (monto(item.compra.as_ref()), monto(item.venta.as_ref()))
                            else {
                                warn!(
                                    service = SERVICE,
                                    casa = %casa,
                                    "cotización descartada: importe no numérico"
                                );
                                return None;
                            };
                            let fecha_actualizacion = item
                                .fecha_actualizacion
                                .as_deref()
                                .and_then(|raw| time::from_storage(raw).ok())
                                .unwrap_or_else(chrono::Utc::now);

                            Some(Cotizacion {
                                casa,
                                nombre: item.nombre,
                                compra,
                                venta,
                                fecha_actualizacion,
                                desactualizada: false,
                            })
                        })
                        .collect();
                    return Ok(cotizaciones);
                }
                Err(e) => {
                    warn!(
                        service = SERVICE,
                        url = %self.url,
                        intento,
                        de = intentos,
                        error = %e,
                        "la consulta de cotizaciones falló"
                    );
                    ultimo = Some(e.to_string());
                }
            }
        }

        warn!(
            service = SERVICE,
            error = ultimo.unwrap_or_default(),
            "cotizaciones no disponibles tras cada reintento"
        );
        Err(AppError::ExternalUnavailable { service: SERVICE })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_importe_se_lee_por_su_texto_y_no_por_su_coma_flotante() {
        let numero = serde_json::json!(990.5);
        assert_eq!(monto(Some(&numero)), Some(Money::parse("990.5").unwrap()));

        let texto = serde_json::json!("1234.5678");
        assert_eq!(
            monto(Some(&texto)),
            Some(Money::parse("1234.5678").unwrap())
        );
    }

    #[test]
    fn un_importe_no_numerico_es_ausencia_y_no_cero() {
        assert_eq!(monto(Some(&serde_json::json!("s/d"))), None);
        assert_eq!(monto(Some(&serde_json::json!(null))), None);
        assert_eq!(monto(None), None);
    }
}
