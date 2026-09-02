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

// ---------------------------------------------------------------- enums

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseProvider {
    #[default]
    Sqlite,
    Mysql,
    Postgres,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Environment {
    Development,
    #[default]
    Production,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemePreference {
    Light,
    Dark,
    #[default]
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EmailClient {
    #[default]
    SystemDefault,
    Gmail,
    Outlook,
    Yahoo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DashboardPeriod {
    #[default]
    Mensual,
    Anual,
    Total,
}

// ---------------------------------------------------------------- sections

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ApplicationConfig {
    pub name: String,
    pub environment: Environment,
    pub seed_enabled: bool,
    pub last_page_size: u32,
    pub theme: ThemePreference,
    pub last_route: String,
    pub sidebar_expanded: bool,
    /// Only set for tests and portable installs; otherwise resolved from the operating system.
    pub data_dir: Option<PathBuf>,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            name: "Certaro".to_owned(),
            environment: Environment::Production,
            seed_enabled: false,
            last_page_size: 30,
            theme: ThemePreference::System,
            last_route: "dashboard".to_owned(),
            sidebar_expanded: true,
            data_dir: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LocaleConfig {
    pub language: String,
    pub formato_fecha: String,
    pub formato_fecha_hora: String,
    pub primer_dia_semana: u8,
    pub simbolo_moneda: String,
    pub separador_miles: String,
    pub separador_decimal: String,
    /// Decimals **shown**; storage is always four (doc 04).
    pub decimales_moneda: u8,
    pub decimales_porcentaje: u8,
    pub moneda_por_defecto: String,
    /// IANA name. Only affects presentation and the reading of civil dates (doc 04 §3.4).
    pub zona_horaria: String,
}

impl Default for LocaleConfig {
    fn default() -> Self {
        Self {
            language: "es".to_owned(),
            formato_fecha: "dd/MM/yyyy".to_owned(),
            formato_fecha_hora: "dd/MM/yyyy HH:mm".to_owned(),
            primer_dia_semana: 1,
            simbolo_moneda: "$".to_owned(),
            separador_miles: ".".to_owned(),
            separador_decimal: ",".to_owned(),
            decimales_moneda: 2,
            decimales_porcentaje: 2,
            moneda_por_defecto: "ars".to_owned(),
            zona_horaria: "America/Argentina/Buenos_Aires".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DiasPorFrecuencia {
    pub diario: Decimal4,
    /// Monday to Saturday.
    pub semanal: Decimal4,
    pub quincenal: Decimal4,
    pub mensual: Decimal4,
}

impl Default for DiasPorFrecuencia {
    fn default() -> Self {
        Self {
            diario: Decimal4::ONE,
            semanal: Decimal4::from_raw(60_000),
            quincenal: Decimal4::from_raw(150_000),
            mensual: Decimal4::from_raw(300_000),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BusinessConfig {
    /// The "GENERCON" of the certificate. Empty falls back to `Application.Name` in reports.
    pub nombre_comercial: String,
    /// The "ENERGIA CONTROLADA" / "Cuentas Claras".
    pub lema: String,
    /// The "PABLO BAEZ".
    pub contratista: String,
    pub cuit: String,
    pub direccion: String,
    pub telefono: String,
    pub email: String,
    pub logo_path: Option<PathBuf>,
    /// Only a suggestion; VAT is typed by hand (doc 06 §4.1).
    pub iva_sugerido: Decimal4,
    pub factura_dias_vencimiento_default: u32,
    /// Upper bound of each ageing bucket, in days, inclusive. Anything past the last one falls
    /// into the open-ended bucket.
    pub buckets_antiguedad: Vec<u32>,
    /// How much a payment may exceed the outstanding balance before being refused (INV-09). Zero
    /// by default; a small tolerance exists for the cent that rounding leaves behind.
    pub tolerancia_sobrepago_factura: Money,
    pub categoria_profundidad_maxima: u8,
    pub dias_por_frecuencia: DiasPorFrecuencia,
}

impl Default for BusinessConfig {
    fn default() -> Self {
        Self {
            nombre_comercial: String::new(),
            lema: String::new(),
            contratista: String::new(),
            cuit: String::new(),
            direccion: String::new(),
            telefono: String::new(),
            email: String::new(),
            logo_path: None,
            iva_sugerido: Decimal4::from_raw(210_000),
            factura_dias_vencimiento_default: 30,
            buckets_antiguedad: vec![30, 60, 90],
            tolerancia_sobrepago_factura: Money::ZERO,
            categoria_profundidad_maxima: 3,
            dias_por_frecuencia: DiasPorFrecuencia::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SettlementConfig {
    pub multiplicador_sabado: Decimal4,
    pub multiplicador_domingo: Decimal4,
    pub multiplicador_feriado: Decimal4,
    pub incluir_sabado: bool,
    pub incluir_domingo: bool,
    pub incluir_feriado: bool,
    pub periodo_por_defecto_dias: u32,
    pub sincronizar_feriados_al_iniciar: bool,
    /// The current year plus the next ones.
    pub anios_feriados_a_sincronizar: u8,
    /// Maximum number of days the attendance grid may query in one request.
    /// 92 covers the last 3 months and keeps the matrix bounded.
    pub asistencia_max_rango_dias: u32,
}

impl Default for SettlementConfig {
    fn default() -> Self {
        Self {
            multiplicador_sabado: Decimal4::from_raw(15_000),
            multiplicador_domingo: Decimal4::from_raw(20_000),
            multiplicador_feriado: Decimal4::from_raw(20_000),
            incluir_sabado: false,
            incluir_domingo: false,
            incluir_feriado: false,
            periodo_por_defecto_dias: 15,
            sincronizar_feriados_al_iniciar: true,
            anios_feriados_a_sincronizar: 2,
            asistencia_max_rango_dias: 92,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DashboardConfig {
    pub last_period: DashboardPeriod,
    pub privacy_mode: bool,
    pub casas_dolar: Vec<String>,
    pub cotizacion_por_defecto: String,
    pub top_clientes_cantidad: u8,
    pub ultimos_movimientos_cantidad: u8,
    pub proyectos_ranking_cantidad: u8,
    pub top_categorias_cantidad: u8,
    /// A drop steeper than this, in percent, raises the falling-income alert (doc 06 §9.11).
    pub alerta_caida_ingresos_pct: Decimal4,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            last_period: DashboardPeriod::Mensual,
            privacy_mode: false,
            casas_dolar: vec!["oficial".to_owned(), "blue".to_owned()],
            cotizacion_por_defecto: "blue".to_owned(),
            top_clientes_cantidad: 5,
            ultimos_movimientos_cantidad: 10,
            proyectos_ranking_cantidad: 5,
            top_categorias_cantidad: 5,
            alerta_caida_ingresos_pct: Decimal4::from_raw(200_000),
        }
    }
}

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
        use crate::error::FieldError;
        use crate::paging::PageRequest;

        let mut errors = Vec::new();
        if !PageRequest::ALLOWED_SIZES.contains(&self.application.last_page_size) {
            errors.push(FieldError::new(
                "application.lastPageSize",
                "Validation.Config.ValorNoPermitido",
            ));
        }
        if !matches!(self.locale.language.as_str(), "es" | "en") {
            errors.push(FieldError::new(
                "locale.language",
                "Validation.Config.ValorNoPermitido",
            ));
        }
        if self.locale.zona_horaria.parse::<chrono_tz::Tz>().is_err() {
            errors.push(FieldError::new(
                "locale.zonaHoraria",
                "Validation.Config.ValorNoPermitido",
            ));
        }
        if self.locale.decimales_moneda > 4 {
            errors.push(
                FieldError::new(
                    "locale.decimalesMoneda",
                    "Validation.Config.ValorNoPermitido",
                )
                .with_param("max", 4),
            );
        }
        if self.external_apis.timeout_seconds == 0 {
            errors.push(FieldError::new(
                "externalApis.timeoutSeconds",
                "Validation.Config.ValorNoPermitido",
            ));
        }
        if self.attachments.max_size_mb == 0
            || self.attachments.max_size_mb > self.attachments.max_total_mb
        {
            errors.push(FieldError::new(
                "attachments.maxSizeMb",
                "Validation.Config.ValorNoPermitido",
            ));
        }
        if self.database.max_connections == 0 {
            errors.push(FieldError::new(
                "database.maxConnections",
                "Validation.Config.ValorNoPermitido",
            ));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(crate::error::AppError::Validation(errors))
        }
    }
}

/// A key/value view used only by the settings screen to render "changed from default" markers.
pub type ConfigOverrides = BTreeMap<String, String>;
