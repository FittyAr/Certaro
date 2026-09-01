//! `AppError` to `ApiError`. See `docs/02-arquitectura.md` §6.2.
//!
//! This is the only place that decides what the user is allowed to see. Persistence, IO and
//! unexpected errors keep their cause in the log and hand the frontend a generic key plus the
//! trace id, so a user can report "trace 0192f3a1…" and the line is findable.

use certaro_application::AppError;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiFieldError {
    pub field: String,
    pub message_key: String,
    pub params: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub code: String,
    pub message_key: String,
    pub params: BTreeMap<String, String>,
    /// Empty except for `VALIDATION`.
    pub fields: Vec<ApiFieldError>,
    /// Correlates with the log line.
    pub trace_id: String,
}

impl ApiError {
    pub fn from_app_error(error: &AppError, trace_id: &str) -> Self {
        if error.is_internal() {
            // `{error:?}` includes the whole `anyhow` chain; the user gets none of it.
            tracing::error!(trace_id, code = error.code(), cause = ?error, "internal error");
        } else {
            tracing::warn!(trace_id, code = error.code(), "request rejected");
        }

        Self {
            code: error.code().to_owned(),
            message_key: error.message_key().to_owned(),
            params: error.params(),
            fields: error
                .fields()
                .iter()
                .map(|f| ApiFieldError {
                    field: f.field.clone(),
                    message_key: f.message_key.clone(),
                    params: f.params.clone(),
                })
                .collect(),
            trace_id: trace_id.to_owned(),
        }
    }
}

/// The result type every command returns.
pub type ApiResult<T> = Result<T, ApiError>;

/// Runs a use case and converts its error, generating the trace id shared by log and response.
pub fn handle<T>(command: &'static str, result: Result<T, AppError>) -> ApiResult<T> {
    result.map_err(|e| {
        let trace_id = uuid::Uuid::now_v7().to_string();
        tracing::debug!(trace_id, command, "command failed");
        ApiError::from_app_error(&e, &trace_id)
    })
}
