//! Ports: the traits the application layer depends on. Infrastructure implements them.
//!
//! See `docs/02-arquitectura.md` §5.

pub mod clock;
pub mod holidays;
pub mod id_generator;
pub mod repositories;
pub mod settings;

pub use clock::ClockPort;
pub use holidays::HolidayProvider;
pub use id_generator::IdGeneratorPort;
pub use repositories::{
    AdelantoCandidato, AsistenciaRepository, CategoriaConUso, CategoriaFiltro, CategoriaRepository,
    CertificadoConRelaciones, CertificadoFiltro, CertificadoRepository, ClienteConResumen,
    ClienteFiltro, ClienteRepository, EmpleadoFiltro, EmpleadoRepository, FacturaConResumen,
    FacturaFiltro, FacturaRepository, FeriadoRepository, LiquidacionConRelaciones,
    LiquidacionFiltro, LiquidacionRepository, MovimientoConRelaciones, MovimientoFiltro,
    MovimientoRepository, MovimientoResumen, ObraConResumen, ObraFiltro, ObraRepository,
    OrdenTrabajoConRelaciones, OrdenTrabajoRepository, ReferenciaTabla, SortDir,
    TipoMovimientoConUso, TipoMovimientoFiltro, TipoMovimientoRepository, TrabajoConRelaciones,
    TrabajoFiltro, TrabajoRepository, Transaction, UnitOfWork,
};
pub use settings::SettingsStore;
