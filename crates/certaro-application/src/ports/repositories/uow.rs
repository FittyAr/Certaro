use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::result::AppResult;
use super::auth::*;
use super::comercial::*;
use super::dashboard::*;
use super::movimientos::*;
use super::operaciones::*;
use super::personal::*;
use super::proyectos::*;
use super::sistema::*;

/// Opens transactions. A use case that writes more than one table has to go through this.
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    async fn begin(&self) -> AppResult<Box<dyn Transaction>>;
}

/// A transaction with one accessor per aggregate. Dropping it without committing rolls back.
///
/// `Sync` as well as `Send` because a use case holds `&dyn Transaction` across an `await`, and a
/// future that does that is only `Send` if the reference is.
#[async_trait]
pub trait Transaction: Send + Sync {
    fn tipos_movimiento(&self) -> &dyn TipoMovimientoRepository;
    fn categorias(&self) -> &dyn CategoriaRepository;
    fn movimientos(&self) -> &dyn MovimientoRepository;
    fn clientes(&self) -> &dyn ClienteRepository;
    fn proyectos(&self) -> &dyn ProyectoRepository;
    fn trabajos(&self) -> &dyn TrabajoRepository;
    fn facturas(&self) -> &dyn FacturaRepository;
    fn ordenes_trabajo(&self) -> &dyn OrdenTrabajoRepository;
    fn certificados(&self) -> &dyn CertificadoRepository;
    fn empleados(&self) -> &dyn EmpleadoRepository;
    fn asistencias(&self) -> &dyn AsistenciaRepository;
    fn liquidaciones(&self) -> &dyn LiquidacionRepository;
    fn adjuntos(&self) -> &dyn AdjuntoRepository;
    fn feriados(&self) -> &dyn FeriadoRepository;
    fn dashboard(&self) -> &dyn DashboardRepository;
    fn metadata(&self) -> &dyn MetadataRepository;
    fn usuarios(&self) -> &dyn UsuarioRepository;
    fn roles(&self) -> &dyn RolRepository;
    fn permisos(&self) -> &dyn PermisoRepository;
    fn sesiones(&self) -> &dyn SesionRepository;
    fn auth_externo(&self) -> &dyn AuthExternoRepository;
    fn kanban_tableros(&self) -> &dyn KanbanTableroRepository;
    fn kanban_columnas(&self) -> &dyn KanbanColumnaRepository;
    fn kanban_tarjetas(&self) -> &dyn KanbanTarjetaRepository;
    fn kanban_etiquetas(&self) -> &dyn KanbanEtiquetaRepository;
    fn kanban_checklists(&self) -> &dyn KanbanChecklistRepository;
    fn calendario_grupos_recurso(&self) -> &dyn CalendarioGrupoRecursoRepository;
    fn calendario_recursos(&self) -> &dyn CalendarioRecursoRepository;
    fn calendario_eventos(&self) -> &dyn CalendarioEventoRepository;

    async fn commit(self: Box<Self>) -> AppResult<()>;
    async fn rollback(self: Box<Self>) -> AppResult<()>;
}

