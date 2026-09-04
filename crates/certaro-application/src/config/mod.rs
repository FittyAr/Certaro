//! Typed configuration. The catalogue of keys is `docs/14-configuracion-e-i18n.md` §2.
//!
//! Configuration is a struct, not a string dictionary. The legacy system read everything through
//! `GetValue("Application:Settlement:MultiplierSaturday", "1.5")`: the key was a literal scattered
//! across the code, the default was rewritten at every call site, and a typo silently returned the
//! default instead of failing.
//!
//! **Deviation from doc 14 §1.2**, recorded there: the types live in `eo-application` rather than
//! `eo-infrastructure`, because use cases receive the section they need and the dependency rule
//! forbids the application layer from importing infrastructure. Loading, merging the three layers
//! and persisting stay in `eo-infrastructure/src/config/`.

use certaro_domain::{Decimal4, Money};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    pub application: ApplicationConfig,
    pub locale: LocaleConfig,
    pub business: BusinessConfig,
    pub settlement: SettlementConfig,
    pub dashboard: DashboardConfig,
    pub external_apis: ExternalApisConfig,
    pub attachments: AttachmentsConfig,
    pub backup: BackupConfig,
    pub communication: CommunicationConfig,
    pub logging: LoggingConfig,
    pub validation: ValidationConfig,
    pub report: ReportConfig,
    pub database: DatabaseConfig,
}


mod sections;
mod types;
mod validation;

pub use sections::*;
pub use types::*;

impl AppConfig {
    /// Development flips three defaults that would be wrong to ship enabled.
    #[must_use]
    pub fn for_development() -> Self {
        let mut c = Self::default();
        c.application.environment = Environment::Development;
        c.application.seed_enabled = true;
        c.logging.level = LogLevel::Debug;
        c.logging.console_enabled = true;
        c
    }

    /// The trading name reports should print, with the documented fallback.
    #[must_use]
    pub fn nombre_para_reportes(&self) -> &str {
        if self.business.nombre_comercial.is_empty() {
            &self.application.name
        } else {
            &self.business.nombre_comercial
        }
    }

    /// Rejects values that are out of range instead of quietly clamping them (doc 14 §3.3).
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        validation::validate_config(self)
    }
}

/// A key/value view used only by the settings screen to render "changed from default" markers.
pub type ConfigOverrides = BTreeMap<String, String>;

