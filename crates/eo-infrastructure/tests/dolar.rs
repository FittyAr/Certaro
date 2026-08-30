//! The dollar quote service against a local HTTP server. No test touches the real network.
//!
//! What is being proved here is the degradation: a timeout, a 500 and a malformed body must all
//! end up as an empty list or as the cache, never as an error the user sees.

use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use eo_application::config::AppConfig;
use eo_application::ports::exchange_rate::ExchangeRateProvider;
use eo_application::ports::repositories::UnitOfWork;
use eo_application::ports::{ClockPort, SettingsStore};
use eo_application::use_cases::cotizaciones::CotizacionesService;
use eo_domain::clock::FixedClock;
use eo_domain::Money;
use eo_infrastructure::config::FileSettingsStore;
use eo_infrastructure::external::dolar::HttpExchangeRateProvider;
use eo_infrastructure::persistence::{open_in_memory, SeaOrmUnitOfWork};
use pretty_assertions::assert_eq;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The response documented in `docs/13-servicios-externos-y-archivos.md` §2.2.
const PAYLOAD: &str = r#"[
  {
    "moneda": "USD",
    "casa": "oficial",
    "nombre": "Oficial",
    "compra": 950.5,
    "venta": 990.5,
    "fechaActualizacion": "2026-08-28T10:00:00.000Z"
  },
  {
    "moneda": "USD",
    "casa": "blue",
    "nombre": "Blue",
    "compra": 1200,
    "venta": 1250,
    "fechaActualizacion": "2026-08-28T10:00:00.000Z"
  },
  {
    "moneda": "USD",
    "casa": "cripto",
    "nombre": "Cripto",
    "compra": 1210,
    "venta": 1260,
    "fechaActualizacion": "2026-08-28T10:00:00.000Z"
  }
]"#;

fn ahora() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 29, 12, 0, 0)
        .single()
        .unwrap_or_else(Utc::now)
}

/// A provider pointed at the mock server. `reintentos` is zero so a failing test does not spend
/// four seconds sleeping between attempts.
fn provider(url: &str, timeout: u32) -> HttpExchangeRateProvider {
    let mut config = AppConfig::default().external_apis;
    config.dollar_url = url.to_owned();
    config.timeout_seconds = timeout;
    config.reintentos = 0;
    HttpExchangeRateProvider::new(&config).unwrap()
}

/// A service whose cache is a real database, which is where the TTL is read from.
async fn servicio(
    provider: Arc<dyn ExchangeRateProvider>,
    reloj: DateTime<Utc>,
) -> CotizacionesService {
    let db = open_in_memory().await.unwrap();
    let uow: Arc<dyn UnitOfWork> = Arc::new(SeaOrmUnitOfWork::new(db));
    let clock: Arc<dyn ClockPort> = Arc::new(FixedClock(reloj));
    let settings: Arc<dyn SettingsStore> = Arc::new(FileSettingsStore::new(
        std::env::temp_dir().join("eo-test-dolar.json"),
        AppConfig::default(),
    ));
    CotizacionesService::new(uow, clock, provider, settings)
}

async fn servidor(respuesta: ResponseTemplate) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/dolares"))
        .respond_with(respuesta)
        .mount(&server)
        .await;
    server
}

fn url(server: &MockServer) -> String {
    format!("{}/v1/dolares", server.uri())
}

#[tokio::test]
async fn dolar_parsea_la_respuesta_esperada() {
    let server =
        servidor(ResponseTemplate::new(200).set_body_raw(PAYLOAD, "application/json")).await;
    let cotizaciones = provider(&url(&server), 5).fetch().await.unwrap();

    assert_eq!(cotizaciones.len(), 3);
    let oficial = &cotizaciones[0];
    assert_eq!(oficial.casa, "oficial");
    assert_eq!(oficial.nombre, "Oficial");
    // Read through the text, so the cent survives.
    assert_eq!(oficial.compra, Money::parse("950.5").unwrap());
    assert_eq!(oficial.venta, Money::parse("990.5").unwrap());
    assert!(!oficial.desactualizada);
}

#[tokio::test]
async fn dolar_descarta_solo_la_casa_con_importe_invalido() {
    let cuerpo = r#"[
      {"casa":"oficial","nombre":"Oficial","compra":"s/d","venta":990.5},
      {"casa":"blue","nombre":"Blue","compra":1200,"venta":1250}
    ]"#;
    let server =
        servidor(ResponseTemplate::new(200).set_body_raw(cuerpo, "application/json")).await;

    let cotizaciones = provider(&url(&server), 5).fetch().await.unwrap();

    // The bad element is dropped; the good one survives.
    assert_eq!(cotizaciones.len(), 1);
    assert_eq!(cotizaciones[0].casa, "blue");
}

#[tokio::test]
async fn dolar_error_500_devuelve_lista_vacia() {
    let server = servidor(ResponseTemplate::new(500)).await;
    let servicio = servicio(Arc::new(provider(&url(&server), 5)), ahora()).await;

    // Degradation is silent: the caller gets a list, not an error.
    assert!(servicio.list().await.unwrap().is_empty());
}

#[tokio::test]
async fn dolar_json_malformado_devuelve_lista_vacia() {
    let server =
        servidor(ResponseTemplate::new(200).set_body_raw("{no es json}", "application/json")).await;
    let servicio = servicio(Arc::new(provider(&url(&server), 5)), ahora()).await;

    assert!(servicio.list().await.unwrap().is_empty());
}

#[tokio::test]
async fn dolar_timeout_devuelve_lista_vacia() {
    let server = servidor(
        ResponseTemplate::new(200)
            .set_body_raw(PAYLOAD, "application/json")
            .set_delay(Duration::from_secs(2)),
    )
    .await;
    let servicio = servicio(Arc::new(provider(&url(&server), 1)), ahora()).await;

    assert!(servicio.list().await.unwrap().is_empty());
}

#[tokio::test]
async fn solo_se_muestran_las_casas_configuradas() {
    let server =
        servidor(ResponseTemplate::new(200).set_body_raw(PAYLOAD, "application/json")).await;
    let servicio = servicio(Arc::new(provider(&url(&server), 5)), ahora()).await;

    let cotizaciones = servicio.list().await.unwrap();

    // `cripto` came in the payload but is not in `Dashboard.CasasDolar`.
    assert_eq!(cotizaciones.len(), 2);
    assert_eq!(cotizaciones[0].casa, "oficial");
    assert_eq!(cotizaciones[1].casa, "blue");
}

#[tokio::test]
async fn dolar_usa_la_cache_dentro_de_la_ventana() {
    let server =
        servidor(ResponseTemplate::new(200).set_body_raw(PAYLOAD, "application/json")).await;
    let servicio = servicio(Arc::new(provider(&url(&server), 5)), ahora()).await;

    assert_eq!(servicio.list().await.unwrap().len(), 2);
    assert_eq!(servicio.list().await.unwrap().len(), 2);

    // Two calls, one request: the second was served from the cache.
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn el_refresco_explicito_ignora_la_cache() {
    let server =
        servidor(ResponseTemplate::new(200).set_body_raw(PAYLOAD, "application/json")).await;
    let servicio = servicio(Arc::new(provider(&url(&server), 5)), ahora()).await;

    servicio.list().await.unwrap();
    servicio.refresh().await.unwrap();

    assert_eq!(server.received_requests().await.unwrap().len(), 2);
}

#[tokio::test]
async fn con_el_servicio_caido_se_sirve_la_cache_marcada_como_vieja() {
    let server =
        servidor(ResponseTemplate::new(200).set_body_raw(PAYLOAD, "application/json")).await;
    let db = open_in_memory().await.unwrap();
    let uow: Arc<dyn UnitOfWork> = Arc::new(SeaOrmUnitOfWork::new(db));
    let settings: Arc<dyn SettingsStore> = Arc::new(FileSettingsStore::new(
        std::env::temp_dir().join("eo-test-dolar-cache.json"),
        AppConfig::default(),
    ));

    // First a good response fills the cache.
    let bueno = CotizacionesService::new(
        Arc::clone(&uow),
        Arc::new(FixedClock(ahora())),
        Arc::new(provider(&url(&server), 5)),
        Arc::clone(&settings),
    );
    assert_eq!(bueno.list().await.unwrap().len(), 2);

    // Then a broken service, on a clock past the TTL so the cache is stale.
    let caido = servidor(ResponseTemplate::new(503)).await;
    let tarde = ahora() + chrono::Duration::hours(5);
    let degradado = CotizacionesService::new(
        uow,
        Arc::new(FixedClock(tarde)),
        Arc::new(provider(&url(&caido), 5)),
        settings,
    );

    let cotizaciones = degradado.list().await.unwrap();
    assert_eq!(cotizaciones.len(), 2);
    assert_eq!(cotizaciones[0].venta, Money::parse("990.5").unwrap());
    // Flagged, so the screen can say "as of…" instead of pretending it is current.
    assert!(cotizaciones.iter().all(|c| c.desactualizada));
}
