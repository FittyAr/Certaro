//! Conversions between SeaORM models and domain entities.
//!
//! Everything the database stores as text or as a scaled integer is interpreted exactly here and
//! nowhere else. A model never leaves this layer.

pub mod adjunto;
pub mod asistencia_empleado;
pub mod auth;
pub mod categoria;
pub mod certificado;
pub mod cliente;
pub mod empleado;
pub mod factura;
pub mod feriado;
pub mod liquidacion;
pub mod movimiento;
pub mod proyecto;
pub mod orden_trabajo;
pub mod tipo_movimiento;
pub mod trabajo;

use chrono::{DateTime, NaiveDate, Utc};
use certaro_application::AppError;
use certaro_domain::entities::Audit;
use certaro_domain::{time, RowVersion};
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

/// A civil date, stored as the bare `YYYY-MM-DD` the domain and the frontend both use.
/// Tolerates the legacy `YYYY-MM-DDTHH:MM:SSZ` form that the first seed wrote.
pub fn civil(raw: &str) -> Result<NaiveDate, AppError> {
    time::parse_civil(raw)
        .or_else(|_| {
            time::from_storage(raw).map(|dt| dt.date_naive())
        })
        .map_err(|e| AppError::persistence(anyhow::anyhow!("invalid date {raw:?}: {e}")))
}

pub fn civil_opt(raw: Option<&str>) -> Result<Option<NaiveDate>, AppError> {
    raw.map(civil).transpose()
}

/// Renders a civil date for storage. Lexicographic order matches chronological order, so a plain
/// `ORDER BY` on the column is correct.
#[must_use]
pub fn civil_to_storage(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

pub fn uuid(raw: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(raw)
        .map_err(|e| AppError::persistence(anyhow::anyhow!("invalid uuid {raw:?}: {e}")))
}

pub fn uuid_opt(raw: Option<&str>) -> Result<Option<Uuid>, AppError> {
    raw.map(uuid).transpose()
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
