//! Application-level commands: liveness, build metadata and the configuration snapshot.

use crate::error::{handle, ApiResult};
use crate::state::AppState;
use certaro_application::config::AppConfig;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub environment: String,
    pub data_dir: String,
}

/// Round-trips a string. Exists so the frontend can prove the IPC bridge works before any screen
/// depends on it, which is what phase 0 verifies.
#[tauri::command]
pub fn ping(message: String) -> String {
    format!("pong: {message}")
}

#[tauri::command]
pub fn app_info(state: State<'_, AppState>) -> ApiResult<AppInfo> {
    let config = state.config();
    handle(
        "app_info",
        Ok(AppInfo {
            name: config.application.name.clone(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            environment: format!("{:?}", config.application.environment).to_lowercase(),
            data_dir: state.paths.root().display().to_string(),
        }),
    )
}

#[tauri::command]
pub fn app_config(state: State<'_, AppState>) -> ApiResult<AppConfig> {
    handle("app_config", Ok(state.config()))
}

#[tauri::command]
pub fn app_is_ready(state: State<'_, AppState>) -> bool {
    state.services().is_ok()
}
