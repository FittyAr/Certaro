//! Shapes shared by every module's contract. See `docs/11-contratos-tauri.md` §4.1.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use eo_domain::entities::Audit;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::paging::PageRequest;
use crate::ports::repositories::SortDir;

/// A list request as it arrives from the frontend: a module-specific filter plus the paging and
/// sorting that every list shares.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery<F> {
    #[serde(default)]
    pub filtro: F,
    pub page: u32,
    pub page_size: u32,
    pub sort_by: Option<String>,
    #[serde(default)]
    pub sort_dir: SortDir,
}

impl<F> ListQuery<F> {
    pub fn page_request(&self) -> PageRequest {
        PageRequest::new(self.page, self.page_size)
    }
}

/// One option of a selector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LookupItem {
    pub id: Uuid,
    pub label: String,
    /// Whatever the selector needs to render the option: a colour, a rate, a state.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub meta: BTreeMap<String, String>,
}

impl LookupItem {
    pub fn new(id: Uuid, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            meta: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_meta(mut self, key: &str, value: impl Into<String>) -> Self {
        self.meta.insert(key.to_owned(), value.into());
        self
    }
}

/// The audit block as the frontend sees it: the row version travels as hexadecimal text so it
/// survives JSON without losing the top bits of a `u64`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditDto {
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub row_version: String,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<&Audit> for AuditDto {
    fn from(audit: &Audit) -> Self {
        Self {
            created_at: audit.created_at,
            updated_at: audit.updated_at,
            row_version: audit.row_version.to_hex(),
            is_deleted: audit.is_deleted,
            deleted_at: audit.deleted_at,
        }
    }
}
