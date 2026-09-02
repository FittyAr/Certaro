//! Ports: the traits the application layer depends on. Infrastructure implements them.
//!
//! See `docs/02-arquitectura.md` §5.

pub mod attachments;
pub mod auth;
pub mod backup;
pub mod clock;
pub mod exchange_rate;
pub mod holidays;
pub mod id_generator;
pub mod reports;
pub mod repositories;
pub mod settings;
pub mod translator;

pub use attachments::{ArchivoAceptado, ArchivoGuardado, AttachmentStore, OpenerPort};
pub use auth::{PasswordHasher, TokenPort, TotpPort};
pub use backup::{BackupItem, BackupPort, ImportResumen, VerificacionBackup};
pub use clock::ClockPort;
pub use exchange_rate::ExchangeRateProvider;
pub use holidays::HolidayProvider;
pub use id_generator::IdGeneratorPort;
pub use reports::{FileWriterPort, ReportPort};
pub use repositories::{
    AdelantoCandidato, AdjuntoRepository, AsistenciaRepository, AuthExternoRepository,
    CategoriaConUso, CategoriaFiltro, CategoriaRepository, CertificadoConRelaciones,
    CertificadoFiltro, CertificadoRepository, ClienteConResumen, ClienteFiltro, ClienteRepository,
    DashboardRepository, EmpleadoFiltro, EmpleadoRepository, EstadoBase, FacturaConResumen,
    FacturaFiltro, FacturaPendiente, FacturaRepository, FeriadoRepository,
    LiquidacionConRelaciones, LiquidacionFiltro, LiquidacionRepository, MetadataRepository,
    MovimientoConRelaciones, MovimientoFiltro, MovimientoRepository, MovimientoResumen,
    OrdenTrabajoConRelaciones, OrdenTrabajoRepository, PermisoRepository, ProyectoConResumen,
    ProyectoFiltro, ProyectoRepository, ReferenciaTabla, RentabilidadFila, RolRepository,
    SesionRepository, SortDir, TipoMovimientoConUso, TipoMovimientoFiltro,
    TipoMovimientoRepository, TotalMensual, TotalPorNombre, TrabajoConRelaciones, TrabajoFiltro,
    TrabajoRepository, Transaction, UnitOfWork, UsuarioRepository,
    KanbanChecklistRepository, KanbanColumnaRepository, KanbanEtiquetaRepository,
    KanbanTableroRepository, KanbanTarjetaRepository,
    CalendarioEventoRepository, CalendarioGrupoRecursoRepository, CalendarioRecursoRepository,
};
pub use settings::SettingsStore;
pub use translator::{MapTranslator, Translator};
