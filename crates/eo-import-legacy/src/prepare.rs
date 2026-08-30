//! Phase 3: destination preparation. See `docs/15-migracion-de-datos.md` §2.

use anyhow::{Context, Result};
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use std::path::Path;

/// Opens (or creates) the destination database, runs migrations, and verifies it is empty.
pub async fn prepare_target(path: &Path) -> Result<DatabaseConnection> {
    let db = eo_infrastructure::persistence::connection::open(path)
        .await
        .context("opening target database")?;

    // Verify the destination is empty (no business rows).
    let result = db
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT COUNT(*) as cnt FROM movimientos".to_owned(),
        ))
        .await
        .context("checking if target is empty")?;

    let count: i64 = result
        .map(|row| row.try_get("", "cnt").unwrap_or(0))
        .unwrap_or(0);

    if count > 0 {
        anyhow::bail!(
            "target database already contains {count} movimientos. \
             The importer requires an empty destination."
        );
    }

    Ok(db)
}
