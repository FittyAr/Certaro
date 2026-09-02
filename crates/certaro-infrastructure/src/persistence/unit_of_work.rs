//! Unit of work over a SeaORM transaction. See `docs/02-arquitectura.md` §9.
//!
//! Every use case runs inside one, including the read-only ones, so a listing that issues several
//! queries observes a single snapshot.

use std::sync::Arc;

use async_trait::async_trait;
use certaro_application::ports::repositories::{
    AdjuntoRepository, AsistenciaRepository, AuthExternoRepository, CategoriaRepository,
    CertificadoRepository, ClienteRepository, DashboardRepository, EmpleadoRepository,
    FacturaRepository, FeriadoRepository, LiquidacionRepository, MetadataRepository,
    MovimientoRepository, OrdenTrabajoRepository, PermisoRepository, ProyectoRepository,
    RolRepository, SesionRepository, TipoMovimientoRepository, TrabajoRepository, Transaction,
    UnitOfWork, UsuarioRepository,
};
use certaro_application::{AppError, AppResult};
use sea_orm::{DatabaseTransaction, TransactionTrait};

use crate::persistence::handle::DbHandle;
use crate::persistence::repositories::adjunto::SeaOrmAdjuntoRepository;
use crate::persistence::repositories::asistencia::SeaOrmAsistenciaRepository;
use crate::persistence::repositories::auth::{
    SeaOrmAuthExternoRepository, SeaOrmPermisoRepository, SeaOrmRolRepository,
    SeaOrmSesionRepository, SeaOrmUsuarioRepository,
};
use crate::persistence::repositories::categoria::SeaOrmCategoriaRepository;
use crate::persistence::repositories::certificado::SeaOrmCertificadoRepository;
use crate::persistence::repositories::cliente::SeaOrmClienteRepository;
use crate::persistence::repositories::dashboard::SeaOrmDashboardRepository;
use crate::persistence::repositories::empleado::SeaOrmEmpleadoRepository;
use crate::persistence::repositories::factura::SeaOrmFacturaRepository;
use crate::persistence::repositories::feriado::SeaOrmFeriadoRepository;
use crate::persistence::repositories::liquidacion::SeaOrmLiquidacionRepository;
use crate::persistence::repositories::metadata::SeaOrmMetadataRepository;
use crate::persistence::repositories::movimiento::SeaOrmMovimientoRepository;
use crate::persistence::repositories::proyecto::SeaOrmProyectoRepository;
use crate::persistence::repositories::orden_trabajo::SeaOrmOrdenTrabajoRepository;
use crate::persistence::repositories::tipo_movimiento::SeaOrmTipoMovimientoRepository;
use crate::persistence::repositories::trabajo::SeaOrmTrabajoRepository;

pub struct SeaOrmUnitOfWork {
    db: DbHandle,
}

impl SeaOrmUnitOfWork {
    pub fn new(db: DbHandle) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UnitOfWork for SeaOrmUnitOfWork {
    async fn begin(&self) -> AppResult<Box<dyn Transaction>> {
        // The guard is released as soon as the transaction exists: it owns its own connection.
        let tx = Arc::new(
            self.db
                .read()
                .await
                .begin()
                .await
                .map_err(AppError::persistence)?,
        );
        Ok(Box::new(SeaOrmTransaction::new(tx)))
    }
}

/// Owns the transaction and one repository per aggregate, all sharing it through an `Arc`.
///
/// Dropping this without committing rolls back, which is what makes an early `?` in a use case
/// safe: there is no path that leaves a half-written transaction open.
pub struct SeaOrmTransaction {
    tx: Arc<DatabaseTransaction>,
    tipos_movimiento: SeaOrmTipoMovimientoRepository,
    categorias: SeaOrmCategoriaRepository,
    movimientos: SeaOrmMovimientoRepository,
    clientes: SeaOrmClienteRepository,
    proyectos: SeaOrmProyectoRepository,
    trabajos: SeaOrmTrabajoRepository,
    facturas: SeaOrmFacturaRepository,
    ordenes_trabajo: SeaOrmOrdenTrabajoRepository,
    certificados: SeaOrmCertificadoRepository,
    empleados: SeaOrmEmpleadoRepository,
    asistencias: SeaOrmAsistenciaRepository,
    liquidaciones: SeaOrmLiquidacionRepository,
    adjuntos: SeaOrmAdjuntoRepository,
    feriados: SeaOrmFeriadoRepository,
    dashboard: SeaOrmDashboardRepository,
    metadata: SeaOrmMetadataRepository,
    usuarios: SeaOrmUsuarioRepository,
    roles: SeaOrmRolRepository,
    permisos: SeaOrmPermisoRepository,
    sesiones: SeaOrmSesionRepository,
    auth_externo: SeaOrmAuthExternoRepository,
}

impl SeaOrmTransaction {
    fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self {
            tipos_movimiento: SeaOrmTipoMovimientoRepository::new(Arc::clone(&tx)),
            categorias: SeaOrmCategoriaRepository::new(Arc::clone(&tx)),
            movimientos: SeaOrmMovimientoRepository::new(Arc::clone(&tx)),
            clientes: SeaOrmClienteRepository::new(Arc::clone(&tx)),
            proyectos: SeaOrmProyectoRepository::new(Arc::clone(&tx)),
            trabajos: SeaOrmTrabajoRepository::new(Arc::clone(&tx)),
            facturas: SeaOrmFacturaRepository::new(Arc::clone(&tx)),
            ordenes_trabajo: SeaOrmOrdenTrabajoRepository::new(Arc::clone(&tx)),
            certificados: SeaOrmCertificadoRepository::new(Arc::clone(&tx)),
            empleados: SeaOrmEmpleadoRepository::new(Arc::clone(&tx)),
            asistencias: SeaOrmAsistenciaRepository::new(Arc::clone(&tx)),
            liquidaciones: SeaOrmLiquidacionRepository::new(Arc::clone(&tx)),
            adjuntos: SeaOrmAdjuntoRepository::new(Arc::clone(&tx)),
            feriados: SeaOrmFeriadoRepository::new(Arc::clone(&tx)),
            dashboard: SeaOrmDashboardRepository::new(Arc::clone(&tx)),
            metadata: SeaOrmMetadataRepository::new(Arc::clone(&tx)),
            usuarios: SeaOrmUsuarioRepository::new(Arc::clone(&tx)),
            roles: SeaOrmRolRepository::new(Arc::clone(&tx)),
            permisos: SeaOrmPermisoRepository::new(Arc::clone(&tx)),
            sesiones: SeaOrmSesionRepository::new(Arc::clone(&tx)),
            auth_externo: SeaOrmAuthExternoRepository::new(Arc::clone(&tx)),
            tx,
        }
    }

    /// Recovers sole ownership of the transaction so it can be finished. The repositories are
    /// dropped first; if any `Arc` were still alive the transaction could not be consumed, and
    /// that would be a bug in this file rather than something the caller can cause.
    fn into_inner(self) -> AppResult<DatabaseTransaction> {
        let Self {
            tx,
            tipos_movimiento,
            categorias,
            movimientos,
            clientes,
            proyectos,
            trabajos,
            facturas,
            ordenes_trabajo,
            certificados,
            empleados,
            asistencias,
            liquidaciones,
            adjuntos,
            feriados,
            dashboard,
            metadata,
            usuarios,
            roles,
            permisos,
            sesiones,
            auth_externo,
        } = self;
        drop(tipos_movimiento);
        drop(categorias);
        drop(movimientos);
        drop(clientes);
        drop(proyectos);
        drop(trabajos);
        drop(facturas);
        drop(ordenes_trabajo);
        drop(certificados);
        drop(empleados);
        drop(asistencias);
        drop(liquidaciones);
        drop(adjuntos);
        drop(feriados);
        drop(dashboard);
        drop(metadata);
        drop(usuarios);
        drop(roles);
        drop(permisos);
        drop(sesiones);
        drop(auth_externo);
        Arc::try_unwrap(tx).map_err(|_| {
            AppError::unexpected(anyhow::anyhow!("transaction still borrowed when finishing"))
        })
    }
}

#[async_trait]
impl Transaction for SeaOrmTransaction {
    fn tipos_movimiento(&self) -> &dyn TipoMovimientoRepository {
        &self.tipos_movimiento
    }

    fn categorias(&self) -> &dyn CategoriaRepository {
        &self.categorias
    }

    fn movimientos(&self) -> &dyn MovimientoRepository {
        &self.movimientos
    }

    fn clientes(&self) -> &dyn ClienteRepository {
        &self.clientes
    }

    fn proyectos(&self) -> &dyn ProyectoRepository {
        &self.proyectos
    }

    fn trabajos(&self) -> &dyn TrabajoRepository {
        &self.trabajos
    }

    fn facturas(&self) -> &dyn FacturaRepository {
        &self.facturas
    }

    fn ordenes_trabajo(&self) -> &dyn OrdenTrabajoRepository {
        &self.ordenes_trabajo
    }

    fn certificados(&self) -> &dyn CertificadoRepository {
        &self.certificados
    }

    fn empleados(&self) -> &dyn EmpleadoRepository {
        &self.empleados
    }

    fn asistencias(&self) -> &dyn AsistenciaRepository {
        &self.asistencias
    }

    fn liquidaciones(&self) -> &dyn LiquidacionRepository {
        &self.liquidaciones
    }

    fn adjuntos(&self) -> &dyn AdjuntoRepository {
        &self.adjuntos
    }

    fn feriados(&self) -> &dyn FeriadoRepository {
        &self.feriados
    }

    fn dashboard(&self) -> &dyn DashboardRepository {
        &self.dashboard
    }

    fn metadata(&self) -> &dyn MetadataRepository {
        &self.metadata
    }

    fn usuarios(&self) -> &dyn UsuarioRepository {
        &self.usuarios
    }

    fn roles(&self) -> &dyn RolRepository {
        &self.roles
    }

    fn permisos(&self) -> &dyn PermisoRepository {
        &self.permisos
    }

    fn sesiones(&self) -> &dyn SesionRepository {
        &self.sesiones
    }

    fn auth_externo(&self) -> &dyn AuthExternoRepository {
        &self.auth_externo
    }

    async fn commit(self: Box<Self>) -> AppResult<()> {
        self.into_inner()?
            .commit()
            .await
            .map_err(AppError::persistence)
    }

    async fn rollback(self: Box<Self>) -> AppResult<()> {
        self.into_inner()?
            .rollback()
            .await
            .map_err(AppError::persistence)
    }
}
