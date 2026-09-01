//! Structured logging. See `docs/02-arquitectura.md` §7 and `docs/18-devops.md`.
//!
//! Two sinks: a compact console line in development, and a daily-rotated JSON file always. The
//! database sink the legacy system left as a stub is deliberately not implemented.

use anyhow::Context;
use certaro_application::config::{LogLevel, LoggingConfig};
use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Holds the background writer alive. Dropping it flushes and stops the file sink, so `main` must
/// keep it for the whole run.
#[must_use = "dropping the guard stops the file logger"]
pub struct TelemetryGuard(#[allow(dead_code)] WorkerGuard);

/// Installs the global subscriber. Call once, before anything else logs.
pub fn init(config: &LoggingConfig, log_dir: &Path) -> anyhow::Result<TelemetryGuard> {
    std::fs::create_dir_all(log_dir).context("creating the log directory")?;

    let appender = tracing_appender::rolling::daily(log_dir, "certaro.log");
    let (writer, guard) = tracing_appender::non_blocking(appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(false)
        .with_writer(writer);

    let console_layer = config.console_enabled.then(|| {
        tracing_subscriber::fmt::layer()
            .compact()
            .with_target(false)
            .with_writer(std::io::stderr)
    });

    tracing_subscriber::registry()
        .with(env_filter(config))
        .with(file_layer)
        .with(console_layer)
        .init();

    tracing::info!(
        level = ?config.level,
        retention_days = config.retention_days,
        "telemetry initialised"
    );

    Ok(TelemetryGuard(guard))
}

/// `EO_LOG` wins over the configured filter, which wins over the configured level. Having the
/// environment variable on top is what makes it possible to debug a packaged build.
fn env_filter(config: &LoggingConfig) -> EnvFilter {
    if let Ok(filter) = EnvFilter::try_from_env("EO_LOG") {
        return filter;
    }
    if !config.filter.is_empty() {
        if let Ok(filter) = EnvFilter::try_new(&config.filter) {
            return filter;
        }
        tracing::warn!(filter = %config.filter, "invalid log filter, falling back to level");
    }
    EnvFilter::new(level_directive(config.level))
}

const fn level_directive(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Trace => "trace",
        LogLevel::Debug => "debug",
        LogLevel::Info => "info",
        LogLevel::Warn => "warn",
        LogLevel::Error => "error",
    }
}

/// Deletes log files older than the retention window. Failures are logged, never fatal: not being
/// able to tidy up is not a reason to refuse to start.
pub fn prune_old_logs(log_dir: &Path, retention_days: u32) {
    let Ok(entries) = std::fs::read_dir(log_dir) else {
        return;
    };
    let cutoff = std::time::SystemTime::now()
        - std::time::Duration::from_secs(u64::from(retention_days) * 24 * 60 * 60);

    for entry in entries.flatten() {
        let is_old = entry
            .metadata()
            .and_then(|m| m.modified())
            .is_ok_and(|modified| modified < cutoff);
        if is_old {
            if let Err(e) = std::fs::remove_file(entry.path()) {
                tracing::warn!(error = %e, path = ?entry.path(), "could not remove an old log file");
            }
        }
    }
}
