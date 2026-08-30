//! Ports: the traits the application layer depends on. Infrastructure implements them.
//!
//! See `docs/02-arquitectura.md` §5.

pub mod clock;
pub mod exchange_rate;
pub mod holidays;
pub mod id_generator;
pub mod repositories;
pub mod settings;

pub use clock::ClockPort;
pub use exchange_rate::ExchangeRateProvider;
pub use holidays::HolidayProvider;
pub use id_generator::IdGeneratorPort;
pub use repositories::{
    AdelantoCandidato, AsistenciaRepository, CategoriaConUso, CategoriaFiltro, CategoriaRepository,
    CertificadoConRelaciones, CertificadoFiltro, CertificadoRepository, ClienteConResumen,
    ClienteFiltro, ClienteRepository, DashboardRepository, EmpleadoFiltro, EmpleadoRepository,
    EstadoBase, FacturaConResumen, FacturaFiltro, FacturaPendiente, FacturaRepository,
    FeriadoRepository, LiquidacionConRelaciones, LiquidacionFiltro, LiquidacionRepository,
    MetadataRepository, MovimientoConRelaciones, MovimientoFiltro, MovimientoRepository,
    MovimientoResumen, ObraConResumen, ObraFiltro, ObraRepository, OrdenTrabajoConRelaciones,
    OrdenTrabajoRepository, ReferenciaTabla, RentabilidadFila, SortDir, TipoMovimientoConUso,
    TipoMovimientoFiltro, TipoMovimientoRepository, TotalMensual, TotalPorNombre,
    TrabajoConRelaciones, TrabajoFiltro, TrabajoRepository, Transaction, UnitOfWork,
};
pub use settings::SettingsStore;
