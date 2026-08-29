//! Ports: the traits the application layer depends on. Infrastructure implements them.
//!
//! See `docs/02-arquitectura.md` §5.

pub mod clock;
pub mod id_generator;
pub mod repositories;
pub mod settings;

pub use clock::ClockPort;
pub use id_generator::IdGeneratorPort;
pub use repositories::{
    CategoriaConUso, CategoriaFiltro, CategoriaRepository, CertificadoConRelaciones,
    CertificadoFiltro, CertificadoRepository, ClienteConResumen, ClienteFiltro, ClienteRepository,
    FacturaConResumen, FacturaFiltro, FacturaRepository, MovimientoConRelaciones, MovimientoFiltro,
    MovimientoRepository, MovimientoResumen, ObraConResumen, ObraFiltro, ObraRepository,
    OrdenTrabajoConRelaciones, OrdenTrabajoRepository, ReferenciaTabla, SortDir,
    TipoMovimientoConUso, TipoMovimientoFiltro, TipoMovimientoRepository, TrabajoConRelaciones,
    TrabajoFiltro, TrabajoRepository, Transaction, UnitOfWork,
};
pub use settings::SettingsStore;
