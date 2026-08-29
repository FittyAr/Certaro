//! Unit of work over a SeaORM transaction. See `docs/02-arquitectura.md` §9.
//!
//! Every use case runs inside one, including the read-only ones, so a listing that issues several
//! queries observes a single snapshot.

use std::sync::Arc;

use async_trait::async_trait;
use eo_application::ports::repositories::{
    CategoriaRepository, MovimientoRepository, TipoMovimientoRepository, Transaction, UnitOfWork,
};
use eo_application::{AppError, AppResult};
use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};

use crate::persistence::repositories::categoria::SeaOrmCategoriaRepository;
use crate::persistence::repositories::movimiento::SeaOrmMovimientoRepository;
use crate::persistence::repositories::tipo_movimiento::SeaOrmTipoMovimientoRepository;

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
}

impl SeaOrmTransaction {
    fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self {
            tipos_movimiento: SeaOrmTipoMovimientoRepository::new(Arc::clone(&tx)),
            categorias: SeaOrmCategoriaRepository::new(Arc::clone(&tx)),
            movimientos: SeaOrmMovimientoRepository::new(Arc::clone(&tx)),
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
        } = self;
        drop(tipos_movimiento);
        drop(categorias);
        drop(movimientos);
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
