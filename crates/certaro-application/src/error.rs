//! Application errors. See `docs/02-arquitectura.md` §6.
//!
//! An error carries an i18n **key**, never translated text. The legacy system put Spanish strings
//! in the service layer, which made the backend responsible for presentation and impossible to
//! translate; that is gone.

use certaro_domain::DomainError;
use std::collections::BTreeMap;

/// One failed field of a write DTO.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldError {
    /// Path of the offending field, in `camelCase`, matching what the frontend renders:
    /// `"concepto"`, `"items[2].porcentajeActual"`.
    pub field: String,
    /// i18n key, e.g. `"Validation.Movimiento.ConceptoRequired"`.
    pub message_key: String,
    /// Named parameters for the message, e.g. `{"max": "500"}`.
    pub params: BTreeMap<String, String>,
}

impl FieldError {
    pub fn new(field: impl Into<String>, message_key: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message_key: message_key.into(),
            params: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_param(mut self, name: impl Into<String>, value: impl ToString) -> Self {
        self.params.insert(name.into(), value.to_string());
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("validation failed")]
    Validation(Vec<FieldError>),

    #[error("not found: {entity} {id}")]
    NotFound { entity: &'static str, id: String },

    #[error("conflict: {code}")]
    Conflict {
        code: &'static str,
        message_key: &'static str,
        params: BTreeMap<String, String>,
    },

    #[error("concurrency conflict on {entity}")]
    Concurrency { entity: &'static str },

    #[error("dependency in use: {code}")]
    DependencyInUse {
        code: &'static str,
        message_key: &'static str,
        params: BTreeMap<String, String>,
    },

    #[error("domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("persistence error")]
    Persistence(#[source] anyhow::Error),

    #[error("external service unavailable: {service}")]
    ExternalUnavailable { service: &'static str },

    #[error("io error")]
    Io(#[source] anyhow::Error),

    #[error("unexpected error")]
    Unexpected(#[source] anyhow::Error),
}

impl AppError {
    /// Stable machine-readable code. The frontend switches on this, never on the message.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            AppError::Validation(_) => "VALIDATION",
            AppError::NotFound { .. } => "NOT_FOUND",
            AppError::Conflict { .. } => "CONFLICT",
            AppError::Concurrency { .. } => "CONCURRENCY",
            AppError::DependencyInUse { .. } => "DEPENDENCY_IN_USE",
            AppError::Domain(_) => "DOMAIN",
            AppError::Persistence(_) => "PERSISTENCE",
            AppError::ExternalUnavailable { .. } => "EXTERNAL_UNAVAILABLE",
            AppError::Io(_) => "IO",
            AppError::Unexpected(_) => "UNEXPECTED",
        }
    }

    /// The i18n key the frontend renders.
    #[must_use]
    pub fn message_key(&self) -> &str {
        match self {
            AppError::Validation(_) => "Error.Validation",
            AppError::NotFound { .. } => "Error.NotFound",
            AppError::Conflict { message_key, .. }
            | AppError::DependencyInUse { message_key, .. } => message_key,
            AppError::Concurrency { .. } => "Error.Concurrency",
            AppError::Domain(_) => "Error.Domain",
            AppError::Persistence(_) => "Error.Persistence",
            AppError::ExternalUnavailable { .. } => "Error.ExternalUnavailable",
            AppError::Io(_) => "Error.Io",
            AppError::Unexpected(_) => "Error.Unexpected",
        }
    }

    /// Named parameters for the message key.
    #[must_use]
    pub fn params(&self) -> BTreeMap<String, String> {
        match self {
            AppError::NotFound { entity, id } => BTreeMap::from([
                ("entity".to_owned(), (*entity).to_owned()),
                ("id".to_owned(), id.clone()),
            ]),
            AppError::Concurrency { entity } => {
                BTreeMap::from([("entity".to_owned(), (*entity).to_owned())])
            }
            AppError::ExternalUnavailable { service } => {
                BTreeMap::from([("service".to_owned(), (*service).to_owned())])
            }
            AppError::Conflict { params, .. } | AppError::DependencyInUse { params, .. } => {
                params.clone()
            }
            _ => BTreeMap::new(),
        }
    }

    /// Field errors, empty except for `Validation`.
    #[must_use]
    pub fn fields(&self) -> &[FieldError] {
        match self {
            AppError::Validation(fields) => fields,
            _ => &[],
        }
    }

    /// Whether the underlying cause must stay in the log and never reach the user.
    ///
    /// A persistence failure can carry a file path or a SQL fragment; showing either to the user
    /// is both useless and a small information leak.
    #[must_use]
    pub const fn is_internal(&self) -> bool {
        matches!(
            self,
            AppError::Persistence(_) | AppError::Io(_) | AppError::Unexpected(_)
        )
    }

    pub fn not_found(entity: &'static str, id: impl ToString) -> Self {
        AppError::NotFound {
            entity,
            id: id.to_string(),
        }
    }

    pub fn conflict(code: &'static str, message_key: &'static str) -> Self {
        AppError::Conflict {
            code,
            message_key,
            params: BTreeMap::new(),
        }
    }

    pub fn dependency_in_use(code: &'static str, message_key: &'static str) -> Self {
        AppError::DependencyInUse {
            code,
            message_key,
            params: BTreeMap::new(),
        }
    }

    pub fn persistence(e: impl Into<anyhow::Error>) -> Self {
        AppError::Persistence(e.into())
    }

    pub fn io(e: impl Into<anyhow::Error>) -> Self {
        AppError::Io(e.into())
    }

    pub fn unexpected(e: impl Into<anyhow::Error>) -> Self {
        AppError::Unexpected(e.into())
    }
}
