//! One-shot importer from the legacy C# database. See `docs/15-migracion-de-datos.md`.
//!
//! Reads the source read-only and writes a fresh database. It never modifies the source.

#![forbid(unsafe_code)]

use clap::Parser;
use certaro_import_legacy::{run_import, ImportOptions, Outcome};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(
    name = "certaro-import-legacy",
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

    let report_path = cli.report.clone().unwrap_or_else(|| {
        cli.target
            .parent()
            .unwrap_or(&cli.target)
            .join("import_report.json")
    });

    let opts = ImportOptions {
        source: cli.source,
        target: cli.target,
        dry_run: cli.dry_run,
        timezone: cli.timezone,
        report: Some(report_path.clone()),
        assume_scaled: cli.assume_scaled,
        assume_unscaled: cli.assume_unscaled,
        allow_orphans: cli.allow_orphans,
    };

    match run_import(opts).await {
        Ok(report) => {
            let code = match report.outcome {
                Outcome::Success => 0,
                Outcome::SuccessWithWarnings => 1,
                Outcome::AlreadyMigrated => 0,
                Outcome::Aborted => 2,
                Outcome::Rollback => {
                    if report.has_blocking_issues() {
                        2
                    } else {
                        3
                    }
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
