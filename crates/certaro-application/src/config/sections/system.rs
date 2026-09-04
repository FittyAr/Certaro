use serde::{Deserialize, Serialize};
use super::super::types::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ExternalApisConfig {
    pub dollar_url: String,
    pub holiday_url: String,
    pub timeout_seconds: u32,
    pub reintentos: u8,
    pub dollar_auto_update: bool,
    pub dollar_cache_minutes: u32,
}

impl Default for ExternalApisConfig {
    fn default() -> Self {
        Self {
            dollar_url: "https://dolarapi.com/v1/dolares".to_owned(),
            holiday_url: "https://api.argentinadatos.com/v1/feriados/".to_owned(),
            timeout_seconds: 30,
            reintentos: 2,
            dollar_auto_update: true,
            dollar_cache_minutes: 60,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AttachmentsConfig {
    pub max_size_mb: u32,
    pub max_total_mb: u32,
    pub trash_retention_days: u32,
    pub extensiones_permitidas: Vec<String>,
}

impl Default for AttachmentsConfig {
    fn default() -> Self {
        Self {
            max_size_mb: 25,
            max_total_mb: 200,
            trash_retention_days: 30,
            extensiones_permitidas: [
                "pdf", "jpg", "jpeg", "png", "gif", "webp", "doc", "docx", "xls", "xlsx", "csv",
                "txt", "zip",
            ]
            .iter()
            .map(|s| (*s).to_owned())
            .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BackupConfig {
    pub enabled: bool,
    pub directory: String,
    pub retention_days: u32,
    pub minimo_a_conservar: u8,
    /// Automatic backup on start when the newest one is older than this.
    pub max_age_days: u32,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            directory: "Backups".to_owned(),
            retention_days: 30,
            minimo_a_conservar: 3,
            max_age_days: 7,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CommunicationConfig {
    pub email_cliente: EmailClient,
    pub gmail_url: String,
    pub outlook_url: String,
    pub yahoo_url: String,
    pub codigo_pais: String,
    /// i18n key of the template, not the text: the message is built in the frontend.
    pub whats_app_template: String,
    pub whats_app_liquidacion_template: String,
    pub email_liquidacion_asunto: String,
}

impl Default for CommunicationConfig {
    fn default() -> Self {
        Self {
            email_cliente: EmailClient::SystemDefault,
            gmail_url: "https://mail.google.com/mail/u/0/?view=cm&fs=1&to={email}".to_owned(),
            outlook_url: "https://outlook.live.com/mail/0/deeplink/compose?to={email}".to_owned(),
            yahoo_url: "https://mail.yahoo.com/d/compose-message?to={email}".to_owned(),
            codigo_pais: "54".to_owned(),
            whats_app_template: "Communication.WhatsAppDefault".to_owned(),
            whats_app_liquidacion_template: "Communication.WhatsAppLiquidacionDefault".to_owned(),
            email_liquidacion_asunto: "Communication.EmailLiquidacionAsuntoDefault".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub retention_days: u32,
    pub console_enabled: bool,
    /// `EnvFilter` syntax; empty means "derive it from `level`".
    pub filter: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            retention_days: 30,
            console_enabled: false,
            filter: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ValidationConfig {
    /// `YYYY-MM-DD`.
    pub fecha_minima: String,
    pub fecha_futura_max_dias: u32,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            fecha_minima: "2000-01-01".to_owned(),
            fecha_futura_max_dias: 365,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ReportConfig {
    pub font: String,
    pub mostrar_logo: bool,
    pub mostrar_firmas: bool,
    /// Empty falls back to the trading name.
    pub pie_de_pagina: String,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            font: "Inter".to_owned(),
            mostrar_logo: true,
            mostrar_firmas: true,
            pie_de_pagina: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DatabaseConfig {
    pub provider: DatabaseProvider,
    /// Connection URL or path. If None for SQLite, the default data_dir/certaro.db is used.
    pub url: Option<String>,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            provider: DatabaseProvider::Sqlite,
            url: None,
            max_connections: 8,
            min_connections: 1,
            acquire_timeout_seconds: 10,
        }
    }
}
