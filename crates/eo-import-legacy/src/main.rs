//! One-shot importer from the legacy C# database. See `docs/15-migracion-de-datos.md`.
//!
//! Reads the source read-only and writes a fresh database. It never modifies the source.
//! Implemented in phase 11; the CLI surface is defined here so the contract is visible earlier.

#![forbid(unsafe_code)]

use clap::Parser;
use std::path::PathBuf;

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
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt().with_target(false).init();
    anyhow::bail!(
        "the importer is implemented in phase 11; source={}, target={}, dry_run={}",
        cli.source.display(),
        cli.target.display(),
        cli.dry_run
    )
}
