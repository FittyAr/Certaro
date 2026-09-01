//! A shared, replaceable handle to the database connection.
//!
//! Restoring a backup has to overwrite the database file, and on Windows that fails outright while
//! a connection holds it open — the legacy system copied over the live file and either failed or
//! left the `-wal` sidecar describing a database that no longer existed. So the connection lives
//! behind this handle: every caller borrows it, and restore is the one operation that swaps it.

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use tokio::sync::{RwLock, RwLockReadGuard};

#[derive(Clone)]
pub struct DbHandle {
    inner: Arc<RwLock<DatabaseConnection>>,
}

impl DbHandle {
    #[must_use]
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            inner: Arc::new(RwLock::new(db)),
        }
    }

    /// Borrows the connection. Held only long enough to start a transaction, never for the length
    /// of one: the transaction owns its own connection from the pool.
    pub async fn read(&self) -> RwLockReadGuard<'_, DatabaseConnection> {
        self.inner.read().await
    }

    /// Puts `db` in place and returns the previous connection, so the caller closes it explicitly
    /// rather than leaving that to a drop whose timing is not observable.
    pub async fn replace(&self, db: DatabaseConnection) -> DatabaseConnection {
        let mut guard = self.inner.write().await;
        std::mem::replace(&mut *guard, db)
    }

    /// Closes the current connection and leaves the handle disconnected.
    ///
    /// While disconnected every query fails with a connection error, which is correct and visible:
    /// the alternative is a query that silently reads the file being replaced.
    pub async fn disconnect(&self) -> Result<(), sea_orm::DbErr> {
        let anterior = self.replace(DatabaseConnection::Disconnected).await;
        match anterior {
            DatabaseConnection::Disconnected => Ok(()),
            conexion => conexion.close().await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn el_handle_devuelve_la_conexion_anterior_al_reemplazar() {
        let handle = DbHandle::new(DatabaseConnection::Disconnected);
        let anterior = handle.replace(DatabaseConnection::Disconnected).await;
        assert!(matches!(anterior, DatabaseConnection::Disconnected));
    }

    #[tokio::test]
    async fn desconectar_dos_veces_no_falla() {
        let handle = DbHandle::new(DatabaseConnection::Disconnected);
        assert!(handle.disconnect().await.is_ok());
        assert!(handle.disconnect().await.is_ok());
    }

    #[tokio::test]
    async fn los_clones_comparten_la_misma_conexion() {
        let handle = DbHandle::new(DatabaseConnection::Disconnected);
        let clon = handle.clone();
        let real = crate::persistence::open_in_memory().await.unwrap();
        clon.replace(real).await;
        // The swap made through the clone is visible through the original.
        assert!(!matches!(
            *handle.read().await,
            DatabaseConnection::Disconnected
        ));
    }
}
