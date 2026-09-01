//! Report generation. See `docs/12-reportes-y-exportaciones.md`.
//!
//! Every generator returns bytes and knows nothing about paths: writing to disk belongs to the
//! caller. That is what lets the whole of this module be tested in memory, which matters because
//! layout is exactly the kind of code that stops being verified once its tests need a temporary
//! directory.

pub mod adapter;
pub mod csv;
pub mod docx;
pub mod filename;
pub mod format;
pub mod json;
pub mod movimientos;
pub mod pdf;
#[cfg(test)]
pub mod tests_support;
pub mod xlsx;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use certaro_application::config::{AppConfig, BusinessConfig, LocaleConfig, ReportConfig};
use certaro_application::ports::Translator;
use certaro_application::result::AppResult;

pub use filename::FormatoExport;

/// What every report needs and no report should look up for itself.
pub struct ReportContext {
    pub empresa: DatosEmpresa,
    pub locale: LocaleConfig,
    pub report: ReportConfig,
    pub i18n: Arc<dyn Translator>,
    pub generado_en: DateTime<Utc>,
}

impl ReportContext {
    #[must_use]
    pub fn new(config: &AppConfig, i18n: Arc<dyn Translator>, generado_en: DateTime<Utc>) -> Self {
        Self {
            empresa: DatosEmpresa::from_config(config),
            locale: config.locale.clone(),
            report: config.report.clone(),
            i18n,
            generado_en,
        }
    }

    /// Shorthand, because a generator resolves dozens of keys and the noise adds up.
    #[must_use]
    pub fn t(&self, key: &str) -> String {
        self.i18n.text(key)
    }

    #[must_use]
    pub fn tp(&self, key: &str, params: &[(&str, &str)]) -> String {
        self.i18n.format(key, params)
    }
}

/// The company as the paperwork names it. All of it configuration: the legacy system had
/// `"PABLO BAEZ"` and `"GENERCON"` compiled into the certificate, so a second user of the software
/// could not print one without a rebuild.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatosEmpresa {
    pub nombre: String,
    pub lema: String,
    pub contratista: String,
    pub cuit: String,
    pub direccion: String,
    pub telefono: String,
    pub email: String,
    pub logo_path: Option<std::path::PathBuf>,
}

impl DatosEmpresa {
    #[must_use]
    pub fn from_config(config: &AppConfig) -> Self {
        let business: &BusinessConfig = &config.business;
        Self {
            nombre: config.nombre_para_reportes().to_owned(),
            lema: business.lema.clone(),
            contratista: business.contratista.clone(),
            cuit: business.cuit.clone(),
            direccion: business.direccion.clone(),
            telefono: business.telefono.clone(),
            email: business.email.clone(),
            logo_path: business.logo_path.clone(),
        }
    }
}

/// A generated document, ready to be written wherever the caller decides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedReport {
    pub bytes: Vec<u8>,
    /// Rows the report covers, which the interface shows once it finishes.
    pub registros: u64,
    /// Proposed file name, for the save dialog.
    pub nombre_sugerido: String,
}

/// A report generator. `Params` carries whatever that particular report needs.
pub trait ReportGenerator {
    type Params;

    fn generate(&self, params: &Self::Params, ctx: &ReportContext) -> AppResult<GeneratedReport>;
}

/// Wraps a generator failure, keeping the step that failed in the internal cause. The user sees
/// the `Error.Io` key; the log sees which part of which document broke.
pub(crate) fn io_error(operacion: &str, cause: impl std::fmt::Display) -> certaro_application::AppError {
    certaro_application::AppError::io(anyhow::anyhow!("{operacion}: {cause}"))
}

/// The footer line every report shares: when it was generated and by whom.
#[must_use]
pub fn footer_text(ctx: &ReportContext, pagina: usize, total: usize) -> String {
    let pie = if ctx.report.pie_de_pagina.trim().is_empty() {
        ctx.empresa.nombre.clone()
    } else {
        ctx.report.pie_de_pagina.clone()
    };
    let paginacion = ctx.tp(
        "Report.Footer.Page",
        &[
            ("actual", &pagina.to_string()),
            ("total", &total.to_string()),
        ],
    );
    format!("{pie} · {paginacion}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::JsonTranslator;

    fn contexto(config: &AppConfig) -> ReportContext {
        ReportContext::new(
            config,
            Arc::new(JsonTranslator::new(&config.locale.language)),
            Utc::now(),
        )
    }

    #[test]
    fn el_nombre_de_la_empresa_cae_en_el_de_la_aplicacion_cuando_esta_vacio() {
        let config = AppConfig::default();
        let ctx = contexto(&config);
        assert_eq!(ctx.empresa.nombre, config.application.name);
    }

    #[test]
    fn el_pie_usa_el_texto_configurado_cuando_hay_uno() {
        let mut config = AppConfig::default();
        config.report.pie_de_pagina = "Uso interno".to_owned();
        let pie = footer_text(&contexto(&config), 2, 5);
        assert!(pie.starts_with("Uso interno"), "{pie}");
        assert!(pie.contains('2') && pie.contains('5'), "{pie}");
    }

    #[test]
    fn sin_pie_configurado_se_usa_el_nombre_de_la_empresa() {
        let mut config = AppConfig::default();
        config.business.nombre_comercial = "GENERCON".to_owned();
        let pie = footer_text(&contexto(&config), 1, 1);
        assert!(pie.starts_with("GENERCON"), "{pie}");
    }
}
