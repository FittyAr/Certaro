//! One-shot importer from the legacy C# database. See `docs/15-migracion-de-datos.md`.
//!
//! Reads the source read-only and writes a fresh database. It never modifies the source.

#![forbid(unsafe_code)]
#![allow(dead_code)] // Many functions are used only when all 21 tables are implemented.

mod dates;
mod derive;
mod inspect;
mod money;
mod prepare;
mod report;
mod scale;
mod text;
mod transfer;
mod verify;

use anyhow::{Context, Result};
use clap::Parser;
use sea_orm::TransactionTrait;
use sqlx::sqlite::SqliteConnectOptions;
use std::path::PathBuf;
use std::process::ExitCode;

use report::{ImportReport, Outcome};

#[derive(Debug, Parser)]
#[command(
    name = "eo-import-legacy",
    about = "Imports an ElectroObraApp (C#) database into the new schema",
    version
)]
struct Cli {
    /// Legacy database. Opened read-only.
    #[arg(long)]
    source: PathBuf,

    /// Destination database. Must not already contain business rows.
    #[arg(long)]
    target: PathBuf,

    /// Reads, verifies and writes the report without touching the destination.
    #[arg(long)]
    dry_run: bool,

    /// IANA timezone used to read the legacy local timestamps.
    #[arg(long, default_value = "America/Argentina/Buenos_Aires")]
    timezone: String,

    /// Where to write `import_report.json`. Defaults to the destination's directory.
    #[arg(long)]
    report: Option<PathBuf>,

    /// Forces the scale interpretation when the migration history is missing.
    #[arg(long, conflicts_with = "assume_unscaled")]
    assume_scaled: bool,

    #[arg(long, conflicts_with = "assume_scaled")]
    assume_unscaled: bool,

    /// Nulls out orphan nullable foreign keys instead of aborting.
    #[arg(long)]
    allow_orphans: bool,

    /// Log at debug level.
    #[arg(long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    let level = if cli.verbose {
        tracing::level_filters::LevelFilter::DEBUG
    } else {
        tracing::level_filters::LevelFilter::INFO
    };
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(level)
        .init();

    match run(cli).await {
        Ok(report) => {
            let code = match report.outcome {
                Outcome::Success => 0,
                Outcome::SuccessWithWarnings => 1,
                Outcome::Aborted => 2,
                Outcome::Rollback => {
                    if report.has_blocking_issues() { 2 } else { 3 }
                }
            };
            ExitCode::from(code)
        }
        Err(e) => {
            tracing::error!("import failed: {e:#}");
            ExitCode::from(3u8)
        }
    }
}

async fn run(cli: Cli) -> Result<ImportReport> {
    let tz: chrono_tz::Tz = cli
        .timezone
        .parse()
        .with_context(|| format!("invalid timezone: {}", cli.timezone))?;

    // Phase 1: inspect source.
    tracing::info!("phase 1: inspecting source database");
    let legacy_options = SqliteConnectOptions::new()
        .filename(&cli.source)
        .read_only(true);
    let legacy = sqlx::sqlite::SqlitePool::connect_with(legacy_options)
        .await
        .with_context(|| format!("opening source database: {}", cli.source.display()))?;

    let (inventory, mut source_info) = inspect::inspect_source(&legacy)
        .await
        .context("phase 1: inspection failed")?;
    source_info.path = cli.source.display().to_string();

    tracing::info!("found {} tables", inventory.len());
    for (table, count) in &inventory {
        tracing::info!("  {table}: {count} rows");
    }

    // Phase 2: detect scale.
    tracing::info!("phase 2: detecting monetary scale");
    let scale = scale::detect_scale(&legacy, cli.assume_scaled, cli.assume_unscaled)
        .await
        .context("phase 2: scale detection failed")?;
    source_info.scale_state = scale;
    tracing::info!("scale state: {:?}", scale);

    let target_info = crate::report::TargetInfo {
        path: cli.target.display().to_string(),
    };
    let mut rpt = ImportReport::new(source_info, target_info, cli.dry_run);

    if scale == crate::report::ScaleState::UnscaledIntegers {
        rpt.warn(
            crate::report::WarningCode::EscalaSinDecimales,
            "*",
            None,
            serde_json::json!({ "detail": "monetary values were stored as integers without decimals; precision is lost" }),
        );
    }

    let report = &mut rpt;
    // `report` is `&mut ImportReport` from here on. We return `rpt` at the end.

    // Phase 3: prepare destination.
    tracing::info!("phase 3: preparing destination database");
    let db = prepare::prepare_target(&cli.target)
        .await
        .context("phase 3: destination preparation failed")?;

    // Phase 4: transfer.
    tracing::info!("phase 4: transferring data");
    let txn = db.begin().await.context("beginning transaction")?;

    transfer::transfer_all(&txn, &legacy, scale, tz, cli.allow_orphans, report)
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
    } else if cli.dry_run {
        tracing::info!("dry run: rolling back");
        txn.rollback().await.context("rollback")?;
    } else {
        tracing::info!("committing transaction");
        txn.commit().await.context("commit")?;
    }

    // Always write the report, even on failure.
    let report_path = cli
        .report
        .unwrap_or_else(|| cli.target.parent().unwrap_or(&cli.target).join("import_report.json"));
    let report_json = serde_json::to_string_pretty(&report)?;
    std::fs::write(&report_path, &report_json)
        .with_context(|| format!("writing report to {}", report_path.display()))?;
    tracing::info!("report written to {}", report_path.display());

    verify_result?;

    Ok(rpt)
}
