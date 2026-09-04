use anyhow::{Context, Result};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseTransaction, Statement};
use sqlx::sqlite::SqlitePool;
use crate::report::{ImportReport, WarningCode};

/// Derives feriados from the legacy config file.
///
/// The old system stored holidays in `appsettings.json` under `Application:Settlement:Holidays`
/// with two incompatible serializations. We try both.
/// Returns the count of recovered holidays.
#[allow(unused_variables)]
pub async fn derive_feriados(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    report: &mut ImportReport,
) -> Result<u64> {
    // Read the legacy config from AppMetadata if available.
    let config_json: Option<String> = db
        .query_one(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT value FROM app_metadata WHERE key = 'config.json'".to_owned(),
        ))
        .await?
        .and_then(|r| r.try_get::<String>("", "value").ok());

    let Some(json_str) = config_json else {
        return Ok(0);
    };

    let parsed: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return Ok(0),
    };

    // Navigate to Application.Settlement.Holidays.
    let holidays = parsed
        .get("Application")
        .and_then(|a| a.get("Settlement"))
        .and_then(|s| s.get("Holidays"));

    let Some(holidays) = holidays else {
        return Ok(0);
    };

    let mut count = 0u64;

    // Try format 1: [{"Date": "...", "Name": "..."}]
    if let Some(arr) = holidays.as_array() {
        for item in arr {
            let fecha = if let Some(date) = item.get("Date").and_then(|d| d.as_str()) {
                date.to_owned()
            } else if let Some(date) = item.as_str() {
                // Format 2: ["2026-01-01T00:00:00"]
                date.split('T').next().unwrap_or(date).to_owned()
            } else {
                report.warn(
                    WarningCode::FeriadoNoParseable,
                    "Feriados",
                    None,
                    serde_json::json!({ "raw": item }),
                );
                continue;
            };

            let nombre = item
                .get("Name")
                .and_then(|n| n.as_str())
                .unwrap_or("Feriado")
                .to_owned();

            // Normalize date to YYYY-MM-DD.
            let fecha_normalized = fecha.split('T').next().unwrap_or(&fecha);

            let sql = format!(
                "INSERT OR IGNORE INTO feriados (fecha, nombre, tipo, origen, created_at)                  VALUES ('{}', '{}', NULL, 'Manual', '{}')",
                fecha_normalized.replace('\'', "''"),
                nombre.replace('\'', "''"),
                chrono::Utc::now().to_rfc3339(),
            );
            db.execute(Statement::from_string(DatabaseBackend::Sqlite, sql))
                .await
                .context("inserting derived feriado")?;
            count += 1;
        }
    }

    Ok(count)
}
