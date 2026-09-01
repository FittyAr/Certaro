//! Report generation and file writing. See `docs/12-reportes-y-exportaciones.md` §6.
//!
//! Two ports, not one: generating bytes is pure and heavily tested, writing them is IO and barely
//! testable. Keeping them apart is what lets every layout test run in memory.

use std::path::Path;

use crate::dtos::certificados::CertificadoDetalle;
use crate::dtos::liquidaciones::LiquidacionDetalle;
use crate::dtos::reportes::{FormatoExport, GeneratedReport, ReporteMovimientos};
use crate::result::AppResult;

/// Produces the documents. The implementation owns the layout, the fonts and the translator.
pub trait ReportPort: Send + Sync {
    fn movimientos(
        &self,
        data: &ReporteMovimientos,
        formato: FormatoExport,
    ) -> AppResult<GeneratedReport>;

    fn liquidacion(&self, data: &LiquidacionDetalle) -> AppResult<GeneratedReport>;

    fn certificado(&self, data: &CertificadoDetalle) -> AppResult<GeneratedReport>;

    /// The filename to prefill the save dialog with, before anything is generated.
    ///
    /// `detalle` is the part that names the subject — the employee, the site — and is sanitised by
    /// the implementation: it reaches here as the user typed it.
    fn nombre_sugerido(
        &self,
        reporte: &str,
        formato: FormatoExport,
        detalle: Option<&str>,
    ) -> AppResult<String>;
}

/// Writes an exported file. Validates the destination before writing a single byte: the frontend
/// hands over a path from the system dialog, and a path from outside is never trusted (doc 11 §1).
pub trait FileWriterPort: Send + Sync {
    /// Returns the number of bytes written.
    fn write(&self, destino: &Path, bytes: &[u8], formato: FormatoExport) -> AppResult<u64>;
}
