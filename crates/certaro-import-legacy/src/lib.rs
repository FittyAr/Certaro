//! Library interface for the legacy SQLite importer. See `docs/15-migracion-de-datos.md`.

#![forbid(unsafe_code)]
#![allow(dead_code)]

pub mod dates;
pub mod derive;
pub mod inspect;
pub mod money;
pub mod prepare;
pub mod report;
pub mod scale;
pub mod text;
pub mod transfer;
pub mod verify;

pub use report::{
    AttachmentReport, DerivedReport, ImportReport, Outcome, ScaleState, SourceInfo, TableReport,
    TargetInfo, Warning, WarningCode,
};

use anyhow::{Context, Result};
use sea_orm::TransactionTrait;
use sqlx::sqlite::SqliteConnectOptions;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportOptions {
    pub source: PathBuf,
    pub target: PathBuf,
    pub dry_run: bool,
    pub timezone: String,
    pub report: Option<PathBuf>,
    pub assume_scaled: bool,
    pub assume_unscaled: bool,
    pub allow_orphans: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            source: PathBuf::new(),
            target: PathBuf::new(),
            dry_run: false,
            timezone: "America/Argentina/Buenos_Aires".to_string(),
            report: None,
            assume_scaled: false,
            assume_unscaled: false,
            allow_orphans: true,
        }
    }
}

pub async fn run_import(opts: ImportOptions) -> Result<ImportReport> {
    let tz: chrono_tz::Tz = opts
        .timezone
        .parse()
        .with_context(|| format!("invalid timezone: {}", opts.timezone))?;

    // Phase 1: inspect source.
    tracing::info!("phase 1: inspecting source database");
    let legacy_options = SqliteConnectOptions::new()
        .filename(&opts.source)
        .read_only(true);
    let legacy = sqlx::sqlite::SqlitePool::connect_with(legacy_options)
        .await
        .with_context(|| format!("opening source database: {}", opts.source.display()))?;

    // Early detection: if the source is already a Rust/Certaro DB (snake_case), don't run the 7-phase
    // legacy import. Just copy the file to the target and rename the source to .bkp.
    // This handles the case where the user points the importer at a DB that was already migrated
    // (e.g., a previous Rust DB copy left at the legacy path).
    {
        let all_tables: Vec<String> =
            sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
                .fetch_all(&legacy)
                .await
                .context("listing tables for migration check")?;
        if inspect::is_already_migrated(&all_tables) {
            tracing::info!("source is already a Certaro database (already migrated), copying directly");
            // Close the pool before file operations (Windows cannot rename an open file)
            drop(legacy);
            if !opts.dry_run {
                tokio::fs::copy(&opts.source, &opts.target)
                    .await
                    .with_context(|| {
                        format!(
                            "copying already-migrated database from {} to {}",
                            opts.source.display(),
                            opts.target.display()
                        )
                    })?;
                // Rename original to .bkp (e.g., electroobra.db -> electroobra.db.bkp)
                let bkp = std::path::PathBuf::from(format!("{}.bkp", opts.source.display()));
                // Only rename if source and target are different files
                if opts.source != opts.target {
                    // Best-effort: if rename fails (e.g., file in use), log but don't fail the import
                    if let Err(e) = tokio::fs::rename(&opts.source, &bkp).await {
                        tracing::warn!("could not rename original to .bkp: {e}");
                    } else {
                        tracing::info!("renamed original to {}", bkp.display());
                    }
                }
            }
            let source_info = report::SourceInfo {
                path: opts.source.display().to_string(),
                schema_version: None,
                scale_state: report::ScaleState::Unknown,
                integrity_check: "ok".to_string(),
            };
            let target_info = report::TargetInfo {
                path: opts.target.display().to_string(),
            };
            let mut rpt = ImportReport::new(source_info, target_info, opts.dry_run);
            rpt.finish();
            rpt.outcome = Outcome::AlreadyMigrated;
            return Ok(rpt);
        }
    }

    let (inventory, mut source_info) = inspect::inspect_source(&legacy)
        .await
        .context("phase 1: inspection failed")?;
    source_info.path = opts.source.display().to_string();

    tracing::info!("found {} tables", inventory.len());
    for (table, count) in &inventory {
        tracing::info!("  {table}: {count} rows");
    }

    // Phase 2: detect scale.
    tracing::info!("phase 2: detecting monetary scale");
    let scale = scale::detect_scale(&legacy, opts.assume_scaled, opts.assume_unscaled)
        .await
        .context("phase 2: scale detection failed")?;
    source_info.scale_state = scale;
    tracing::info!("scale state: {:?}", scale);

    let target_info = report::TargetInfo {
        path: opts.target.display().to_string(),
    };
    let mut rpt = ImportReport::new(source_info, target_info, opts.dry_run);

    if scale == report::ScaleState::UnscaledIntegers {
        rpt.warn(
            report::WarningCode::EscalaSinDecimales,
            "*",
            None,
            serde_json::json!({ "detail": "monetary values were stored as integers without decimals; precision is lost" }),
        );
    }

    let report = &mut rpt;

    // Phase 3: prepare destination.
    tracing::info!("phase 3: preparing destination database");
    let db = prepare::prepare_target(&opts.target)
        .await
        .context("phase 3: destination preparation failed")?;

    // Phase 4: transfer.
    tracing::info!("phase 4: transferring data");
    let txn = db.begin().await.context("beginning transaction")?;

    transfer::transfer_all(&txn, &legacy, scale, tz, opts.allow_orphans, report)
        .await
        .context("phase 4: transfer failed")?;

    // Phase 5: derivation (certificates, advances, contacts, holidays, invoice states).
    tracing::info!("phase 5: deriving data");
    derive::derive_all(&txn, &legacy, report)
        .await
        .context("phase 5: derivation failed")?;

    // Phase 6: verification.
    tracing::info!("phase 6: verifying");
    let verify_result = verify::verify(&txn, report).await;

    // Phase 7: commit or rollback.
    report.finish();

    if report.has_blocking_issues() || verify_result.is_err() {
        tracing::error!("rolling back due to blocking issues");
        txn.rollback().await.context("rollback")?;
        report.outcome = Outcome::Rollback;
    } else if opts.dry_run {
        tracing::info!("dry run: rolling back");
        txn.rollback().await.context("rollback")?;
    } else {
        tracing::info!("committing transaction");
        txn.commit().await.context("commit")?;
    }

    // Write report if path was requested.
    if let Some(report_path) = &opts.report {
        let report_json = serde_json::to_string_pretty(&report)?;
        let _ = std::fs::write(report_path, &report_json);
    }

    verify_result?;

    Ok(rpt)
}
