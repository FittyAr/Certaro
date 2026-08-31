//! The adapters that connect the generators to the application layer.

use std::path::Path;
use std::sync::Arc;

use eo_application::config::AppConfig;
use eo_application::dtos::certificados::CertificadoDetalle;
use eo_application::dtos::liquidaciones::LiquidacionDetalle;
use eo_application::dtos::reportes::{FormatoExport, GeneratedReport, ReporteMovimientos};
use eo_application::error::FieldError;
use eo_application::ports::{ClockPort, FileWriterPort, ReportPort, SettingsStore};
use eo_application::result::AppResult;
use eo_application::AppError;

use super::pdf;
use super::{csv, docx, filename, json, xlsx, ReportContext};
use crate::i18n::JsonTranslator;

pub struct ReportGeneratorAdapter {
    settings: Arc<dyn SettingsStore>,
    clock: Arc<dyn ClockPort>,
}

impl ReportGeneratorAdapter {
    #[must_use]
    pub fn new(settings: Arc<dyn SettingsStore>, clock: Arc<dyn ClockPort>) -> Self {
        Self { settings, clock }
    }

    /// A context built from the configuration as it stands **now**: a company name changed in
    /// settings has to show on the next report without restarting.
    fn context(&self) -> ReportContext {
        let config: AppConfig = self.settings.snapshot();
        let translator = Arc::new(JsonTranslator::new(&config.locale.language));
        ReportContext::new(&config, translator, self.clock.now_utc())
    }
}

impl ReportPort for ReportGeneratorAdapter {
    fn movimientos(
        &self,
        data: &ReporteMovimientos,
        formato: FormatoExport,
    ) -> AppResult<GeneratedReport> {
        let ctx = self.context();
        match formato {
            FormatoExport::Pdf => pdf::movimientos::generate(data, &ctx),
            FormatoExport::Xlsx => xlsx::movimientos(data, &ctx),
            FormatoExport::Docx => docx::movimientos(data, &ctx),
            FormatoExport::Csv => csv::movimientos(data, &ctx),
            FormatoExport::Json => json::movimientos(data, &ctx),
        }
    }

    fn liquidacion(&self, data: &LiquidacionDetalle) -> AppResult<GeneratedReport> {
        pdf::liquidacion::generate(data, &self.context())
    }

    fn certificado(&self, data: &CertificadoDetalle) -> AppResult<GeneratedReport> {
        pdf::certificado::generate(data, &self.context())
    }

    fn nombre_sugerido(
        &self,
        reporte: &str,
        formato: FormatoExport,
        detalle: Option<&str>,
    ) -> AppResult<String> {
        let now = self.clock.now_utc();
        let interno = formato_interno(formato);
        let nombre = match reporte {
            "movimientos" => filename::movimientos(now, interno),
            // The subject's name plus today's date: what the dialog opens with, which the user can
            // still change. The exact date of the settlement or the certificate is not known here.
            otro => format!(
                "{}_{}.{}",
                filename::sanitize(otro, "Reporte"),
                match detalle {
                    Some(d) => format!(
                        "{}_{}",
                        filename::sanitize(d, "detalle"),
                        filename::day(now.date_naive())
                    ),
                    None => filename::stamp(now),
                },
                interno.extension()
            ),
        };
        Ok(nombre)
    }
}

/// The application enum and the reporting one are deliberately separate types: the first is part of
/// the Tauri contract, the second belongs to the layout code.
const fn formato_interno(formato: FormatoExport) -> filename::FormatoExport {
    match formato {
        FormatoExport::Pdf => filename::FormatoExport::Pdf,
        FormatoExport::Xlsx => filename::FormatoExport::Xlsx,
        FormatoExport::Docx => filename::FormatoExport::Docx,
        FormatoExport::Csv => filename::FormatoExport::Csv,
        FormatoExport::Json => filename::FormatoExport::Json,
    }
}

/// Writes an export to the path the user picked in the system dialog.
#[derive(Debug, Default)]
pub struct FsFileWriter;

impl FileWriterPort for FsFileWriter {
    fn write(&self, destino: &Path, bytes: &[u8], formato: FormatoExport) -> AppResult<u64> {
        validate(destino, formato)?;
        std::fs::write(destino, bytes).map_err(|e| {
            AppError::io(anyhow::anyhow!("export.write {}: {e}", destino.display()))
        })?;
        Ok(bytes.len() as u64)
    }
}

/// The path comes from outside, so it is checked before a byte is written: the directory has to
/// exist and the extension has to match the format asked for (doc 11 §5.11).
fn validate(destino: &Path, formato: FormatoExport) -> AppResult<()> {
    let Some(directorio) = destino.parent() else {
        return Err(AppError::Validation(vec![FieldError::new(
            "destino",
            "Validation.Export.DestinoInvalido",
        )]));
    };
    if !directorio.as_os_str().is_empty() && !directorio.is_dir() {
        return Err(AppError::Validation(vec![FieldError::new(
            "destino",
            "Validation.Export.DirectorioNoExiste",
        )]));
    }
    let extension = destino
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase);
    if extension.as_deref() != Some(formato.extension()) {
        return Err(AppError::Validation(vec![FieldError::new(
            "destino",
            "Validation.Export.ExtensionNoCoincide",
        )]));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn una_extension_que_no_coincide_con_el_formato_se_rechaza() {
        let dir = tempfile::tempdir().unwrap();
        let destino = dir.path().join("movimientos.csv");
        let error = FsFileWriter
            .write(&destino, b"x", FormatoExport::Pdf)
            .unwrap_err();
        assert!(matches!(error, AppError::Validation(_)));
    }

    #[test]
    fn un_directorio_inexistente_se_rechaza_antes_de_escribir() {
        let destino = Path::new("D:/no/existe/este/camino/movimientos.pdf");
        let error = FsFileWriter
            .write(destino, b"x", FormatoExport::Pdf)
            .unwrap_err();
        assert!(matches!(error, AppError::Validation(_)));
    }

    #[test]
    fn una_ruta_valida_escribe_y_devuelve_los_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let destino = dir.path().join("movimientos.csv");
        let escritos = FsFileWriter
            .write(&destino, b"hola", FormatoExport::Csv)
            .unwrap();
        assert_eq!(escritos, 4);
        assert_eq!(std::fs::read(&destino).unwrap(), b"hola");
    }

    #[test]
    fn la_extension_se_compara_sin_distinguir_mayusculas() {
        let dir = tempfile::tempdir().unwrap();
        let destino = dir.path().join("Movimientos.CSV");
        assert!(FsFileWriter
            .write(&destino, b"x", FormatoExport::Csv)
            .is_ok());
    }
}
