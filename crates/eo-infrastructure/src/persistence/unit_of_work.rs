//! Unit of work over a SeaORM transaction. See `docs/02-arquitectura.md` §9.
//!
//! Every use case runs inside one, including the read-only ones, so a listing that issues several
//! queries observes a single snapshot.

use std::sync::Arc;

use async_trait::async_trait;
use eo_application::ports::repositories::{
    CategoriaRepository, CertificadoRepository, ClienteRepository, FacturaRepository,
    MovimientoRepository, ObraRepository, OrdenTrabajoRepository, TipoMovimientoRepository,
    TrabajoRepository, Transaction, UnitOfWork,
};
use eo_application::{AppError, AppResult};
use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};

use crate::persistence::repositories::categoria::SeaOrmCategoriaRepository;
use crate::persistence::repositories::certificado::SeaOrmCertificadoRepository;
use crate::persistence::repositories::cliente::SeaOrmClienteRepository;
use crate::persistence::repositories::factura::SeaOrmFacturaRepository;
use crate::persistence::repositories::movimiento::SeaOrmMovimientoRepository;
use crate::persistence::repositories::obra::SeaOrmObraRepository;
use crate::persistence::repositories::orden_trabajo::SeaOrmOrdenTrabajoRepository;
use crate::persistence::repositories::tipo_movimiento::SeaOrmTipoMovimientoRepository;
use crate::persistence::repositories::trabajo::SeaOrmTrabajoRepository;

pub struct SeaOrmUnitOfWork {
    db: DatabaseConnection,
}

impl SeaOrmUnitOfWork {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UnitOfWork for SeaOrmUnitOfWork {
    async fn begin(&self) -> AppResult<Box<dyn Transaction>> {
        let tx = Arc::new(self.db.begin().await.map_err(AppError::persistence)?);
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
    obras: SeaOrmObraRepository,
    trabajos: SeaOrmTrabajoRepository,
    facturas: SeaOrmFacturaRepository,
    ordenes_trabajo: SeaOrmOrdenTrabajoRepository,
    certificados: SeaOrmCertificadoRepository,
}

impl SeaOrmTransaction {
    fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self {
            tipos_movimiento: SeaOrmTipoMovimientoRepository::new(Arc::clone(&tx)),
            categorias: SeaOrmCategoriaRepository::new(Arc::clone(&tx)),
            movimientos: SeaOrmMovimientoRepository::new(Arc::clone(&tx)),
            clientes: SeaOrmClienteRepository::new(Arc::clone(&tx)),
            obras: SeaOrmObraRepository::new(Arc::clone(&tx)),
            trabajos: SeaOrmTrabajoRepository::new(Arc::clone(&tx)),
            facturas: SeaOrmFacturaRepository::new(Arc::clone(&tx)),
            ordenes_trabajo: SeaOrmOrdenTrabajoRepository::new(Arc::clone(&tx)),
            certificados: SeaOrmCertificadoRepository::new(Arc::clone(&tx)),
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
            obras,
            trabajos,
            facturas,
            ordenes_trabajo,
            certificados,
        } = self;
        drop(tipos_movimiento);
        drop(categorias);
        drop(movimientos);
        drop(clientes);
        drop(obras);
        drop(trabajos);
        drop(facturas);
        drop(ordenes_trabajo);
        drop(certificados);
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

    fn obras(&self) -> &dyn ObraRepository {
        &self.obras
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
