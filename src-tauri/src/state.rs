//! Shared application state. Built once in `bootstrap`, read by every command.
//!
//! Use cases are constructed here and stored behind `Arc`, so a command never resolves a
//! dependency graph on the hot path. See `docs/02-arquitectura.md` §8.

use std::sync::{Arc, OnceLock};

use certaro_application::config::AppConfig;
use certaro_application::ports::exchange_rate::ExchangeRateProvider;
use certaro_application::ports::holidays::HolidayProvider;
use certaro_application::ports::repositories::UnitOfWork;
use certaro_application::ports::settings::SettingsStore;
use certaro_application::ports::{AttachmentStore, BackupPort, OpenerPort};
use certaro_application::use_cases::adjuntos::AdjuntosService;
use certaro_application::use_cases::asistencias::AsistenciasService;
use certaro_application::use_cases::categorias::CategoriasService;
use certaro_application::use_cases::certificados::CertificadosService;
use certaro_application::use_cases::clientes::ClientesService;
use certaro_application::use_cases::comercial::ComercialService;
use certaro_application::use_cases::configuracion::ConfiguracionService;
use certaro_application::use_cases::cotizaciones::CotizacionesService;
use certaro_application::use_cases::dashboard::DashboardService;
use certaro_application::use_cases::empleados::EmpleadosService;
use certaro_application::use_cases::facturas::FacturasService;
use certaro_application::use_cases::feriados::FeriadosService;
use certaro_application::use_cases::liquidaciones::LiquidacionesService;
use certaro_application::use_cases::movimientos::MovimientosService;
use certaro_application::use_cases::proyectos::ProyectosService;
use certaro_application::use_cases::ordenes_trabajo::OrdenesTrabajoService;
use certaro_application::use_cases::reportes::ReportesService;
use certaro_application::use_cases::sistema::SistemaService;
use certaro_application::use_cases::tipos_movimiento::TiposMovimientoService;
use certaro_application::use_cases::trabajos::TrabajosService;
use certaro_application::AppError;
use certaro_domain::clock::{Clock, SystemClock};
use certaro_domain::ids::{IdGenerator, UuidV7Generator};
use certaro_infrastructure::paths::AppPaths;
use certaro_infrastructure::reporting::adapter::{FsFileWriter, ReportGeneratorAdapter};

/// The use cases, available only once the background bootstrap has opened the database.
pub struct Services {
    pub tipos_movimiento: TiposMovimientoService,
    pub categorias: CategoriasService,
    pub movimientos: MovimientosService,
    pub clientes: ClientesService,
    pub proyectos: ProyectosService,
    pub trabajos: TrabajosService,
    pub facturas: FacturasService,
    pub ordenes_trabajo: OrdenesTrabajoService,
    pub certificados: CertificadosService,
    pub empleados: EmpleadosService,
    pub asistencias: AsistenciasService,
    pub liquidaciones: LiquidacionesService,
    pub feriados: FeriadosService,
    pub dashboard: DashboardService,
    pub comercial: ComercialService,
    pub cotizaciones: CotizacionesService,
    pub reportes: ReportesService,
    pub adjuntos: AdjuntosService,
    pub sistema: SistemaService,
    pub configuracion: ConfiguracionService,
}

impl Services {
    #[allow(clippy::too_many_arguments)]
    fn build(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
        settings: Arc<dyn SettingsStore>,
        holidays: Arc<dyn HolidayProvider>,
        dolar: Arc<dyn ExchangeRateProvider>,
        attachments: Arc<dyn AttachmentStore>,
        opener: Arc<dyn OpenerPort>,
        backup: Arc<dyn BackupPort>,
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
            clientes: ClientesService::new(Arc::clone(&uow), Arc::clone(&clock), Arc::clone(&ids)),
            proyectos: ProyectosService::new(Arc::clone(&uow), Arc::clone(&clock), Arc::clone(&ids)),
            trabajos: TrabajosService::new(Arc::clone(&uow), Arc::clone(&clock), Arc::clone(&ids)),
            facturas: FacturasService::new(
                Arc::clone(&uow),
                Arc::clone(&clock),
                Arc::clone(&ids),
                Arc::clone(&settings),
            ),
            ordenes_trabajo: OrdenesTrabajoService::new(
                Arc::clone(&uow),
                Arc::clone(&clock),
                Arc::clone(&ids),
            ),
            certificados: CertificadosService::new(
                Arc::clone(&uow),
                Arc::clone(&clock),
                Arc::clone(&ids),
                Arc::clone(&settings),
            ),
            empleados: EmpleadosService::new(
                Arc::clone(&uow),
                Arc::clone(&clock),
                Arc::clone(&ids),
            ),
            asistencias: AsistenciasService::new(
                Arc::clone(&uow),
                Arc::clone(&clock),
                Arc::clone(&ids),
                Arc::clone(&settings),
            ),
            liquidaciones: LiquidacionesService::new(
                Arc::clone(&uow),
                Arc::clone(&clock),
                Arc::clone(&ids),
            ),
            feriados: FeriadosService::new(
                Arc::clone(&uow),
                Arc::clone(&clock),
                holidays,
                Arc::clone(&settings),
            ),
            dashboard: DashboardService::new(
                Arc::clone(&uow),
                Arc::clone(&clock),
                Arc::clone(&settings),
            ),
            comercial: ComercialService::new(
                Arc::clone(&uow),
                Arc::clone(&clock),
                Arc::clone(&settings),
            ),
            cotizaciones: CotizacionesService::new(
                Arc::clone(&uow),
                Arc::clone(&clock),
                dolar,
                Arc::clone(&settings),
            ),
            reportes: ReportesService::new(
                Arc::clone(&uow),
                Arc::new(ReportGeneratorAdapter::new(
                    Arc::clone(&settings),
                    Arc::clone(&clock),
                )),
                Arc::new(FsFileWriter),
            ),
            adjuntos: AdjuntosService::new(
                Arc::clone(&uow),
                Arc::clone(&attachments),
                opener,
                Arc::clone(&clock),
                Arc::clone(&ids),
                Arc::clone(&settings),
            ),
            sistema: SistemaService::new(
                Arc::clone(&uow),
                backup,
                attachments,
                Arc::clone(&settings),
                Arc::clone(&clock),
            ),
            configuracion: ConfiguracionService::new(Arc::clone(&settings)),
            movimientos: MovimientosService::new(uow, clock, ids, settings),
        }
    }
}

pub struct AppState {
    pub paths: AppPaths,
    pub settings: Arc<dyn SettingsStore>,
    pub clock: Arc<dyn Clock>,
    pub ids: Arc<dyn IdGenerator>,
    /// The database connection handle, allowing connection replacement on restore or legacy import.
    db: OnceLock<certaro_infrastructure::persistence::DbHandle>,
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
            db: OnceLock::new(),
            services: OnceLock::new(),
        }
    }

    pub fn config(&self) -> AppConfig {
        self.settings.snapshot()
    }

    pub fn is_sqlite_mode(&self) -> bool {
        self.config().database.provider == certaro_application::config::DatabaseProvider::Sqlite
    }

    pub fn db(&self) -> Option<&certaro_infrastructure::persistence::DbHandle> {
        self.db.get()
    }

    /// Publishes the use cases. Called exactly once; a second call is ignored, which can only
    /// happen if bootstrap were ever run twice.
    #[allow(clippy::too_many_arguments)]
    pub fn install_services(
        &self,
        db: certaro_infrastructure::persistence::DbHandle,
        uow: Arc<dyn UnitOfWork>,
        holidays: Arc<dyn HolidayProvider>,
        dolar: Arc<dyn ExchangeRateProvider>,
        attachments: Arc<dyn AttachmentStore>,
        opener: Arc<dyn OpenerPort>,
        backup: Arc<dyn BackupPort>,
    ) {
        let _ = self.db.set(db);
        let services = Services::build(
            uow,
            Arc::clone(&self.clock),
            Arc::clone(&self.ids),
            Arc::clone(&self.settings),
            holidays,
            dolar,
            attachments,
            opener,
            backup,
        );
        let _ = self.services.set(services);
    }

    pub fn services(&self) -> Result<&Services, AppError> {
        self.services
            .get()
            .ok_or_else(|| AppError::conflict("APP_NOT_READY", "Error.AppNotReady"))
    }
}
