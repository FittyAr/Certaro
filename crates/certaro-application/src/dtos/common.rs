//! Shapes shared by every module's contract. See `docs/11-contratos-tauri.md` §4.1.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use certaro_domain::entities::Audit;
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

/// The state of an entity together with where it may legally go next.
///
/// The frontend renders the buttons from `transiciones_permitidas` instead of listing the enum:
/// a dropdown built from every variant is how the legacy system let an annulled invoice go back
/// to draft.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoInfo {
    /// Variant name, e.g. `"Emitida"`.
    pub actual: String,
    /// Full i18n key, e.g. `"State.Factura.Emitida"`.
    pub clave: String,
    pub transiciones_permitidas: Vec<TransicionPermitida>,
    pub es_terminal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransicionPermitida {
    pub destino: String,
    pub clave: String,
    /// i18n key of the button label, e.g. `"Actions.Factura.Anular"`.
    pub accion: String,
    pub requiere_confirmacion: bool,
}

impl EstadoInfo {
    /// Builds the block for any state machine. `confirmar` decides which edges the interface has
    /// to ask about before following.
    pub fn build<S: certaro_domain::StateMachine>(estado: S, confirmar: impl Fn(S, S) -> bool) -> Self {
        let entity = S::ENTITY;
        Self {
            actual: estado.as_key().to_owned(),
            clave: format!("State.{entity}.{}", estado.as_key()),
            transiciones_permitidas: estado
                .allowed_targets()
                .iter()
                .map(|&destino| TransicionPermitida {
                    destino: destino.as_key().to_owned(),
                    clave: format!("State.{entity}.{}", destino.as_key()),
                    accion: format!("Actions.{entity}.{}", destino.as_key()),
                    requiere_confirmacion: confirmar(estado, destino),
                })
                .collect(),
            es_terminal: estado.is_terminal(),
        }
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
