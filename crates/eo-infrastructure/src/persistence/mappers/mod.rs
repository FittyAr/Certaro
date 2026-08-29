//! Conversions between SeaORM models and domain entities.
//!
//! Everything the database stores as text or as a scaled integer is interpreted exactly here and
//! nowhere else. A model never leaves this layer.

pub mod tipo_movimiento;

use chrono::{DateTime, Utc};
use eo_application::AppError;
use eo_domain::entities::Audit;
use eo_domain::{time, RowVersion};
use uuid::Uuid;

/// A stored timestamp that cannot be parsed is data corruption, not a user error, so it surfaces
/// as a persistence failure with the offending value in the log.
pub fn instant(raw: &str) -> Result<DateTime<Utc>, AppError> {
    time::from_storage(raw)
        .map_err(|e| AppError::persistence(anyhow::anyhow!("invalid timestamp {raw:?}: {e}")))
}

pub fn instant_opt(raw: Option<&str>) -> Result<Option<DateTime<Utc>>, AppError> {
    raw.map(instant).transpose()
}

pub fn uuid(raw: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(raw)
        .map_err(|e| AppError::persistence(anyhow::anyhow!("invalid uuid {raw:?}: {e}")))
}

pub fn row_version(raw: &[u8]) -> Result<RowVersion, AppError> {
    RowVersion::from_slice(raw)
        .map_err(|e| AppError::persistence(anyhow::anyhow!("invalid row version: {e}")))
}

/// Reads the audit block shared by every business table.
pub fn audit(
    created_at: &str,
    updated_at: Option<&str>,
    version: &[u8],
    is_deleted: bool,
    deleted_at: Option<&str>,
) -> Result<Audit, AppError> {
    Ok(Audit {
        created_at: instant(created_at)?,
        updated_at: instant_opt(updated_at)?,
        row_version: row_version(version)?,
        is_deleted,
        deleted_at: instant_opt(deleted_at)?,
    })
}
