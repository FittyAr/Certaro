//! Phase 2: scale detection. See `docs/15-migracion-de-datos.md` §3.2.

use anyhow::{Context, Result};
use sqlx::sqlite::SqlitePool;

use crate::report::ScaleState;

/// Detects whether the legacy database already has monetary values scaled ×10_000.
///
/// The detection reads `__EFMigrationsHistory` and checks for the `RescaleMonetaryValues`
/// migration. If the table is missing or empty, the caller must provide explicit flags.
pub async fn detect_scale(
    pool: &SqlitePool,
    assume_scaled: bool,
    assume_unscaled: bool,
) -> Result<ScaleState> {
    // Check if the migration history table exists.
    let has_history: bool = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='__EFMigrationsHistory'",
    )
    .fetch_one(pool)
    .await
    .context("checking migration history table")?;

    if has_history {
        let applied: Vec<String> = sqlx::query_scalar(
            "SELECT MigrationId FROM __EFMigrationsHistory ORDER BY MigrationId",
        )
        .fetch_all(pool)
        .await
        .context("reading migration history")?;

        if applied.iter().any(|m| m.contains("RescaleMonetaryValues")) {
            return Ok(ScaleState::AlreadyScaled);
        }
        if !applied.is_empty() {
            return Ok(ScaleState::UnscaledIntegers);
        }
    }

    // Migration history is missing or empty. Require explicit flags.
    if assume_scaled {
        return Ok(ScaleState::AlreadyScaled);
    }
    if assume_unscaled {
        return Ok(ScaleState::UnscaledIntegers);
    }

    anyhow::bail!(
        "cannot determine scale state: __EFMigrationsHistory is missing or empty. \
         Use --assume-scaled or --assume-unscaled to force."
    )
}
