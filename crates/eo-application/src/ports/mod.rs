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
    CategoriaConUso, CategoriaFiltro, CategoriaRepository, MovimientoConRelaciones,
    MovimientoFiltro, MovimientoRepository, MovimientoResumen, ReferenciaTabla, SortDir,
    TipoMovimientoConUso, TipoMovimientoFiltro, TipoMovimientoRepository, Transaction, UnitOfWork,
};
pub use settings::SettingsStore;
