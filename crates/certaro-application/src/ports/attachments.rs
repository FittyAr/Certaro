//! Attachment storage. See `docs/13-servicios-externos-y-archivos.md` §1.
//!
//! The port covers the **file**, not the row: the row is a repository like any other. Splitting
//! them is what lets the use case write both inside one transaction and still be tested without a
//! disk.

use async_trait::async_trait;
use std::path::{Path, PathBuf};

use certaro_domain::entities::EntidadAdjunto;
use uuid::Uuid;

use crate::result::AppResult;

/// A file accepted for storage: validated, sanitised and ready to be written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchivoAceptado {
    /// Original name after sanitising. What the user reads.
    pub nombre: String,
    pub mime: &'static str,
    pub tamano: u64,
}

/// Where a stored file ended up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchivoGuardado {
    pub archivo: ArchivoAceptado,
    /// Relative to the attachments root, with `/` separators.
    pub ruta_relativa: String,
}

#[async_trait]
pub trait AttachmentStore: Send + Sync {
    /// Validates the source without copying anything: size, extension, and that the leading bytes
    /// match the extension. Separate from `store` so the use case can refuse before it writes.
    async fn accept(&self, origen: &Path, cupo_restante: u64) -> AppResult<ArchivoAceptado>;

    /// Copies the file into place under `{tipo}/{entidad}/{id}_{nombre}`.
    async fn store(
        &self,
        origen: &Path,
        entidad_tipo: EntidadAdjunto,
        entidad_id: Uuid,
        id: Uuid,
        archivo: ArchivoAceptado,
    ) -> AppResult<ArchivoGuardado>;

    /// Moves the file to the trash rather than deleting it, so a mistaken delete is reversible for
    /// as long as the retention window lasts.
    async fn trash(&self, ruta_relativa: &str) -> AppResult<()>;

    /// Absolute path of a stored attachment, checking that the file is actually there.
    fn resolve(&self, ruta_relativa: &str) -> AppResult<PathBuf>;

    /// Bytes currently used by one entity, to enforce the per-entity quota.
    async fn usado_por(&self, entidad_tipo: EntidadAdjunto, entidad_id: Uuid) -> AppResult<u64>;

    /// Deletes trashed files older than the retention window. Returns how many went.
    async fn purge_trash(&self, dias: u32) -> AppResult<u32>;
}

/// Opening a file or showing it in the file manager. Separate from the store because it is the one
/// operation that hands control to the operating system.
pub trait OpenerPort: Send + Sync {
    fn open(&self, ruta: &Path) -> AppResult<()>;
    /// Shows the file selected in the system file manager (doc 13 §1.5).
    fn reveal(&self, ruta: &Path) -> AppResult<()>;
    /// Opens a URL in the browser or the registered handler: used by the mail and WhatsApp links.
    fn open_url(&self, url: &str) -> AppResult<()>;
}
