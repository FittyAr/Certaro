//! End-to-end exercise of `movimientos`: the server-side filter, the summary over the whole
//! filter, the foreign-key checks and the freeze on a settled advance.

use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use certaro_application::config::AppConfig;
use certaro_application::dtos::categorias::CategoriaInput;
use certaro_application::dtos::common::ListQuery;
use certaro_application::dtos::movimientos::{MovimientoFiltroDto, MovimientoInput};
use certaro_application::ports::repositories::{SortDir, UnitOfWork};
use certaro_application::ports::{ClockPort, IdGeneratorPort, SettingsStore};
use certaro_application::use_cases::categorias::CategoriasService;
use certaro_application::use_cases::movimientos::MovimientosService;
use certaro_domain::clock::FixedClock;
use certaro_domain::constants::tipos_movimiento;
use certaro_domain::ids::UuidV7Generator;
use certaro_domain::{Decimal4, Moneda, Money};
use certaro_infrastructure::config::FileSettingsStore;
use certaro_infrastructure::persistence::DbHandle;
use certaro_infrastructure::persistence::{open_in_memory, SeaOrmUnitOfWork};
pub use sea_orm::{ConnectionTrait, DatabaseConnection};
use uuid::Uuid;

pub struct Fixture {
    pub movimientos: MovimientosService,
    pub categorias: CategoriasService,
    pub db: DatabaseConnection,
}

pub async fn fixture() -> Fixture {
    let db = open_in_memory().await.unwrap();
    let uow: Arc<dyn UnitOfWork> = Arc::new(SeaOrmUnitOfWork::new(DbHandle::new(db.clone())));
    let clock: Arc<dyn ClockPort> = Arc::new(FixedClock(
        DateTime::parse_from_rfc3339("2026-08-29T12:00:00Z")
            .unwrap()
            .into(),
    ));
    let ids: Arc<dyn IdGeneratorPort> = Arc::new(UuidV7Generator);
    let settings: Arc<dyn SettingsStore> = Arc::new(FileSettingsStore::new(
        std::env::temp_dir().join("eo-test-config.json"),
        AppConfig::default(),
    ));

    Fixture {
        movimientos: MovimientosService::new(
            Arc::clone(&uow),
            Arc::clone(&clock),
            Arc::clone(&ids),
            settings,
        ),
        categorias: CategoriasService::new(uow, clock, ids),
        db,
    }
}

impl Fixture {
    pub async fn categoria(&self, nombre: &str) -> Uuid {
        self.categorias
            .create(CategoriaInput {
                nombre: nombre.to_owned(),
                descripcion: None,
                color_hex: Some("#FFAA00".into()),
                icono: None,
                categoria_padre_id: None,
            })
            .await
            .unwrap()
            .id
    }
}

pub fn fecha(day: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, day, 10, 0, 0).unwrap()
}

pub fn input(concepto: &str, monto: &str, cantidad: &str, categoria: Uuid) -> MovimientoInput {
    MovimientoInput {
        fecha: fecha(15),
        concepto: concepto.to_owned(),
        monto: Money::parse(monto).unwrap(),
        cantidad: Decimal4::parse(cantidad).unwrap(),
        tipo_movimiento_id: tipos_movimiento::GASTO,
        moneda: Moneda::Ars,
        cotizacion_aplicada: None,
        tipo_concepto_pago_id: None,
        categoria_id: Some(categoria),
        cliente_id: None,
        trabajo_id: None,
        empleado_id: None,
        factura_id: None,
    }
}

pub fn query(filtro: MovimientoFiltroDto) -> ListQuery<MovimientoFiltroDto> {
    ListQuery {
        filtro,
        page: 1,
        page_size: 30,
        sort_by: None,
        sort_dir: SortDir::Asc,
    }
}
