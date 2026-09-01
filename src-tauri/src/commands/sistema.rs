//! Commands of configuration, backups and the JSON dump. See `docs/11-contratos-tauri.md` §5.13.
//!
//! `backup_restore` and `backup_import_json` are destructive. The frontend confirms them twice and
//! the backend takes a backup of the current state before either runs, so there is always a way
//! back from a mistaken click.

use certaro_application::config::AppConfig;
use certaro_application::dtos::dashboard::EstadoSistema;
use certaro_application::ports::{BackupItem, ImportResumen, VerificacionBackup};
use certaro_application::use_cases::configuracion::Cambios;
use tauri::State;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn config_get_all(state: State<'_, AppState>) -> ApiResult<AppConfig> {
    handle("config_get_all", Ok(state.config()))
}

#[tauri::command]
pub async fn config_set(state: State<'_, AppState>, cambios: Cambios) -> ApiResult<AppConfig> {
    let outcome = match state.services() {
        Ok(services) => services.configuracion.set(cambios).await,
        Err(e) => Err(e),
    };
    handle("config_set", outcome)
}

#[tauri::command]
pub async fn config_reset(state: State<'_, AppState>, claves: Vec<String>) -> ApiResult<AppConfig> {
    let outcome = match state.services() {
        Ok(services) => services.configuracion.reset(claves).await,
        Err(e) => Err(e),
    };
    handle("config_reset", outcome)
}

#[tauri::command]
pub async fn sistema_info(state: State<'_, AppState>) -> ApiResult<EstadoSistema> {
    let outcome = match state.services() {
        Ok(services) => services.sistema.info(env!("CARGO_PKG_VERSION")).await,
        Err(e) => Err(e),
    };
    handle("sistema_info", outcome)
}

#[tauri::command]
pub async fn backup_list(state: State<'_, AppState>) -> ApiResult<Vec<BackupItem>> {
    let outcome = match state.services() {
        Ok(services) => services.sistema.backups().await,
        Err(e) => Err(e),
    };
    handle("backup_list", outcome)
}

#[tauri::command]
pub async fn backup_create(state: State<'_, AppState>) -> ApiResult<BackupItem> {
    let outcome = match state.services() {
        Ok(services) => services.sistema.backup_create().await,
        Err(e) => Err(e),
    };
    handle("backup_create", outcome)
}

#[tauri::command]
pub async fn backup_verify(
    state: State<'_, AppState>,
    nombre: String,
) -> ApiResult<VerificacionBackup> {
    let outcome = match state.services() {
        Ok(services) => services.sistema.backup_verify(&nombre).await,
        Err(e) => Err(e),
    };
    handle("backup_verify", outcome)
}

#[tauri::command]
pub async fn backup_restore(state: State<'_, AppState>, nombre: String) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.sistema.backup_restore(&nombre).await,
        Err(e) => Err(e),
    };
    handle("backup_restore", outcome)
}

#[tauri::command]
pub async fn backup_export_json(
    state: State<'_, AppState>,
    destino: String,
) -> ApiResult<ImportResumen> {
    let outcome = match state.services() {
        Ok(services) => services.sistema.export_json(&destino).await,
        Err(e) => Err(e),
    };
    handle("backup_export_json", outcome)
}

#[tauri::command]
pub async fn backup_import_json(
    state: State<'_, AppState>,
    origen: String,
) -> ApiResult<ImportResumen> {
    let outcome = match state.services() {
        Ok(services) => services.sistema.import_json(&origen).await,
        Err(e) => Err(e),
    };
    handle("backup_import_json", outcome)
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyImportSummaryDto {
    pub success: bool,
    pub outcome: String,
    pub total_rows: u64,
    pub warnings_count: usize,
    pub blocking_issues: Vec<String>,
    pub warnings: Vec<LegacyImportWarningDto>,
    pub tables: Vec<LegacyImportTableDto>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyImportWarningDto {
    pub code: String,
    pub table: String,
    pub row_id: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyImportTableDto {
    pub source: String,
    pub target: String,
    pub source_rows: u64,
    pub target_rows: u64,
}

#[tauri::command]
pub async fn sistema_detect_legacy_db(
    state: State<'_, AppState>,
) -> ApiResult<Option<certaro_infrastructure::paths::LegacyDbCandidate>> {
    let candidate = state.paths.find_legacy_database();
    handle("sistema_detect_legacy_db", Ok(candidate))
}

#[tauri::command]
pub async fn sistema_run_legacy_import(
    state: State<'_, AppState>,
    origen: String,
    allow_orphans: Option<bool>,
) -> ApiResult<LegacyImportSummaryDto> {
    let outcome: Result<LegacyImportSummaryDto, certaro_application::AppError> = async {
        let source_path = std::path::PathBuf::from(&origen);
        if !source_path.is_file() {
            return Err(certaro_application::AppError::Validation(vec![
                certaro_application::error::FieldError::new("origen", "Validation.Backup.RutaInvalida"),
            ]));
        }

        let target_temp = state.paths.database().with_extension("legacy_import.tmp");
        if target_temp.exists() {
            let _ = tokio::fs::remove_file(&target_temp).await;
        }

        let opts = certaro_import_legacy::ImportOptions {
            source: source_path,
            target: target_temp.clone(),
            dry_run: false,
            timezone: "America/Argentina/Buenos_Aires".to_string(),
            report: None,
            assume_scaled: false,
            assume_unscaled: false,
            allow_orphans: allow_orphans.unwrap_or(true),
        };

        let report = certaro_import_legacy::run_import(opts).await.map_err(|e| {
            certaro_application::AppError::unexpected(anyhow::anyhow!("legacy import failed: {e}"))
        })?;

        if report.has_blocking_issues()
            || matches!(
                report.outcome,
                certaro_import_legacy::Outcome::Rollback | certaro_import_legacy::Outcome::Aborted
            )
        {
            let _ = tokio::fs::remove_file(&target_temp).await;
            return Err(certaro_application::AppError::conflict(
                "IMPORT_FAILED",
                "Welcome.ImportFailed",
            ));
        }

        // Disconnect active database connection before replacing the file
        if let Some(db_handle) = state.db() {
            db_handle
                .disconnect()
                .await
                .map_err(certaro_application::AppError::persistence)?;
        }

        let target_db = state.paths.database();
        if target_db.exists() {
            let bak = target_db.with_extension("db.pre_import_bak");
            let _ = tokio::fs::rename(&target_db, &bak).await;
        }
        let _ = tokio::fs::remove_file(target_db.with_extension("db-wal")).await;
        let _ = tokio::fs::remove_file(target_db.with_extension("db-shm")).await;

        tokio::fs::rename(&target_temp, &target_db)
            .await
            .map_err(|e| {
                certaro_application::AppError::io(anyhow::anyhow!(
                    "replacing database after import: {e}"
                ))
            })?;

        let new_db = certaro_infrastructure::persistence::open(&target_db)
            .await
            .map_err(certaro_application::AppError::persistence)?;
        if let Some(db_handle) = state.db() {
            db_handle.replace(new_db).await;
        }

        let mut total_rows: u64 = 0;
        let mut tables = Vec::new();
        for t in &report.tables {
            total_rows += t.target_rows;
            tables.push(LegacyImportTableDto {
                source: t.source.clone(),
                target: t.target.clone(),
                source_rows: t.source_rows,
                target_rows: t.target_rows,
            });
        }

        let mut warnings = Vec::new();
        for w in &report.warnings {
            warnings.push(LegacyImportWarningDto {
                code: format!("{:?}", w.code),
                table: w.table.clone(),
                row_id: w.row_id.clone(),
                detail: w.detail.to_string(),
            });
        }

        Ok(LegacyImportSummaryDto {
            success: true,
            outcome: format!("{:?}", report.outcome),
            total_rows,
            warnings_count: warnings.len(),
            blocking_issues: report.blocking_issues,
            warnings,
            tables,
        })
    }
    .await;

    handle("sistema_run_legacy_import", outcome)
}

#[tauri::command]
pub async fn dev_seed_database(
    state: State<'_, AppState>,
) -> ApiResult<certaro_infrastructure::persistence::SeedSummary> {
    let outcome: Result<certaro_infrastructure::persistence::SeedSummary, certaro_application::AppError> = async {
        let db_handle = state.db().ok_or_else(|| {
            certaro_application::AppError::unexpected(anyhow::anyhow!("database not ready"))
        })?;
        let conn = db_handle.read().await;
        certaro_infrastructure::persistence::seed_demo_data(&conn).await
    }
    .await;

    handle("dev_seed_database", outcome)
}
