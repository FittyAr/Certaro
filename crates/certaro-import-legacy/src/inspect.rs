//! Phase 1: inspection of the source database. See `docs/15-migracion-de-datos.md` §2.

use anyhow::{Context, Result};
use sqlx::sqlite::SqlitePool;

use crate::report::{ScaleState, SourceInfo};

/// The 21 business tables the importer expects, in the order they appear in the legacy schema.
const EXPECTED_TABLES: &[&str] = &[
    "TiposMovimiento",
    "TiposConceptoPago",
    "Categorias",
    "Clientes",
    "ClienteContactos",
    "Obras",
    "Trabajos",
    "OrdenesTrabajo",
    "OrdenTrabajoItems",
    "Facturas",
    "PagosFactura",
    "Empleados",
    "AsistenciasEmpleado",
    "Liquidaciones",
    "Movimientos",
    "Adjuntos",
    "AppMetadata",
];

/// Tables that exist in the legacy schema but are not imported.
const EXCLUDED_TABLES: &[&str] = &["__EFMigrationsHistory", "SchemaVersions"];

/// Returns true if the database looks like an already-migrated Certaro/Rust database
/// (snake_case tables) rather than a legacy Avalonia/C# database (PascalCase).
pub fn is_already_migrated(all_tables: &[String]) -> bool {
    // Rust/Certaro uses seaql_migrations and snake_case tables; legacy uses PascalCase
    all_tables.iter().any(|t| t == "seaql_migrations")
        || all_tables.iter().any(|t| t == "tipos_movimiento")
}

/// Row counts per table, in the same order as `EXPECTED_TABLES`.
pub type Inventory = Vec<(String, u64)>;

/// Phase 1: verify that the expected tables exist, run `PRAGMA integrity_check`, and count rows.
pub async fn inspect_source(pool: &SqlitePool) -> Result<(Inventory, SourceInfo)> {
    // Integrity check.
    let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(pool)
        .await
        .context("integrity_check")?;
    if integrity.to_lowercase() != "ok" {
        anyhow::bail!("source database integrity check failed: {integrity}");
    }

    // List all tables.
    let all_tables: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .fetch_all(pool)
            .await
            .context("listing tables")?;

    // Verify expected tables exist.
    for table in EXPECTED_TABLES {
        if !all_tables.iter().any(|t| t == table) {
            anyhow::bail!("expected table {table} not found in source database");
        }
    }

    // Count rows per table.
    let mut inventory = Vec::new();
    for table in EXPECTED_TABLES {
        let count: u64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM [{table}]"))
            .fetch_one(pool)
            .await
            .with_context(|| format!("counting rows in {table}"))?;
        inventory.push((table.to_string(), count));
    }

    // Read schema version from __EFMigrationsHistory.
    let schema_version = if all_tables.iter().any(|t| t == "__EFMigrationsHistory") {
        let versions: Vec<String> = sqlx::query_scalar(
            "SELECT MigrationId FROM __EFMigrationsHistory ORDER BY MigrationId",
        )
        .fetch_all(pool)
        .await
        .context("reading migration history")?;
        versions.last().cloned()
    } else {
        None
    };

    let source_info = SourceInfo {
        path: String::new(), // Filled by the caller.
        schema_version,
        scale_state: ScaleState::Unknown, // Filled by phase 2.
        integrity_check: integrity,
    };

    Ok((inventory, source_info))
}

/// Returns the list of tables in the source that are neither expected nor excluded.
/// These would be unexpected and should cause an abort.
pub fn unexpected_tables(all_tables: &[String]) -> Vec<&str> {
    all_tables
        .iter()
        .filter(|t| {
            !EXPECTED_TABLES.contains(&t.as_str()) && !EXCLUDED_TABLES.contains(&t.as_str())
        })
        .map(|s| s.as_str())
        .collect()
}
