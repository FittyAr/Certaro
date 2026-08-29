//! Shared application state. Built once in `bootstrap`, read by every command.
//!
//! Use cases are constructed here and stored behind `Arc`, so a command never resolves a
//! dependency graph on the hot path. See `docs/02-arquitectura.md` §8.

use std::sync::{Arc, OnceLock};

use eo_application::config::AppConfig;
use eo_application::ports::repositories::UnitOfWork;
use eo_application::ports::settings::SettingsStore;
use eo_application::use_cases::categorias::CategoriasService;
use eo_application::use_cases::clientes::ClientesService;
use eo_application::use_cases::facturas::FacturasService;
use eo_application::use_cases::movimientos::MovimientosService;
use eo_application::use_cases::obras::ObrasService;
use eo_application::use_cases::tipos_movimiento::TiposMovimientoService;
use eo_application::use_cases::trabajos::TrabajosService;
use eo_application::AppError;
use eo_domain::clock::{Clock, SystemClock};
use eo_domain::ids::{IdGenerator, UuidV7Generator};
use eo_infrastructure::paths::AppPaths;

/// The use cases, available only once the background bootstrap has opened the database.
pub struct Services {
    pub tipos_movimiento: TiposMovimientoService,
    pub categorias: CategoriasService,
    pub movimientos: MovimientosService,
    pub clientes: ClientesService,
    pub obras: ObrasService,
    pub trabajos: TrabajosService,
    pub facturas: FacturasService,
}

impl Services {
    fn build(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
        settings: Arc<dyn SettingsStore>,
    ) -> Self {
        Self {
            tipos_movimiento: TiposMovimientoService::new(
                Arc::clone(&uow),
                Arc::clone(&clock),
                Arc::clone(&ids),
            ),
            categorias: CategoriasService::new(
                Arc::clone(&uow),
                Arc::clone(&clock),
                Arc::clone(&ids),
            ),
            clientes: ClientesService::new(
                Arc::clone(&uow),
                Arc::clone(&clock),
                Arc::clone(&ids),
            ),
            obras: ObrasService::new(Arc::clone(&uow), Arc::clone(&clock), Arc::clone(&ids)),
            trabajos: TrabajosService::new(Arc::clone(&uow), Arc::clone(&clock), Arc::clone(&ids)),
            facturas: FacturasService::new(
                Arc::clone(&uow),
                Arc::clone(&clock),
                Arc::clone(&ids),
                Arc::clone(&settings),
            ),
            movimientos: MovimientosService::new(uow, clock, ids, settings),
        }
    }
}

pub struct AppState {
    pub paths: AppPaths,
    pub settings: Arc<dyn SettingsStore>,
    pub clock: Arc<dyn Clock>,
    pub ids: Arc<dyn IdGenerator>,
    /// Written once by `bootstrap`. A command that arrives before the window received
    /// `app://ready` gets a clean error instead of a panic on an empty database handle.
    services: OnceLock<Services>,
}

impl AppState {
    pub fn new(paths: AppPaths, settings: Arc<dyn SettingsStore>) -> Self {
        Self {
            paths,
            settings,
            clock: Arc::new(SystemClock),
            ids: Arc::new(UuidV7Generator),
            services: OnceLock::new(),
        }
    }

    pub fn config(&self) -> AppConfig {
        self.settings.snapshot()
    }

    /// Publishes the use cases. Called exactly once; a second call is ignored, which can only
    /// happen if bootstrap were ever run twice.
    pub fn install_services(&self, uow: Arc<dyn UnitOfWork>) {
        let services = Services::build(
            uow,
            Arc::clone(&self.clock),
            Arc::clone(&self.ids),
            Arc::clone(&self.settings),
        );
        let _ = self.services.set(services);
    }

    pub fn services(&self) -> Result<&Services, AppError> {
        self.services
            .get()
            .ok_or_else(|| AppError::conflict("APP_NOT_READY", "Error.AppNotReady"))
    }
}
