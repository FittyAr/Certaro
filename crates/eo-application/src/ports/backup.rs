//! Backup, restore and the JSON dump of the database. See `docs/13-servicios-externos-y-archivos.md`
//! §4 and §5.
//!
//! All of it is infrastructure: `VACUUM INTO`, `PRAGMA integrity_check`, closing and reopening the
//! connection. The application layer only decides when, and in what order.

use async_trait::async_trait;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::result::AppResult;

/// One file in the backup directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupItem {
    /// File name, which is also the identifier the frontend sends back.
    pub nombre: String,
    pub creado_en: DateTime<Utc>,
    pub bytes: u64,
}

/// Result of `PRAGMA integrity_check` on a backup file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificacionBackup {
    pub ok: bool,
    /// What the pragma answered, verbatim. Not translated: it is diagnostic output.
    pub detalle: String,
}

/// How many tables and rows an import moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResumen {
    pub tablas: u32,
    pub filas: u64,
}

#[async_trait]
pub trait BackupPort: Send + Sync {
    /// Newest first.
    async fn list(&self) -> AppResult<Vec<BackupItem>>;

    /// `VACUUM INTO` plus `PRAGMA integrity_check`. A copy that does not verify is not a backup.
    async fn create(&self) -> AppResult<BackupItem>;

    async fn verify(&self, nombre: &str) -> AppResult<VerificacionBackup>;

    /// Replaces the live database with the backup. Takes a fresh backup of the current state first,
    /// so restoring by mistake is itself undoable.
    async fn restore(&self, nombre: &str) -> AppResult<()>;

    /// Deletes backups past the retention window, always keeping the newest `minimo`.
    ///
    /// The minimum is what stops the cleanup from leaving nothing behind after a long absence, which
    /// is what the legacy retention did.
    async fn prune(&self, dias: u32, minimo: usize) -> AppResult<u32>;

    /// Dumps every table to JSON: columns declared once, rows as arrays (doc 13 §5.1).
    async fn export_json(&self, destino: &Path) -> AppResult<ImportResumen>;

    /// Replaces the contents of the database with the dump, in one transaction.
    async fn import_json(&self, origen: &Path) -> AppResult<ImportResumen>;
}
