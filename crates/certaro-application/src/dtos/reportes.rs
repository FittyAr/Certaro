//! Contract of the reports. See `docs/11-contratos-tauri.md` §5.11 and `docs/12`.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dtos::movimientos::{MovimientoFiltroDto, MovimientoListItem, MovimientoResumenDto};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum FormatoExport {
    Pdf,
    Xlsx,
    Docx,
    Csv,
    Json,
}

impl FormatoExport {
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            FormatoExport::Pdf => "pdf",
            FormatoExport::Xlsx => "xlsx",
            FormatoExport::Docx => "docx",
            FormatoExport::Csv => "csv",
            FormatoExport::Json => "json",
        }
    }
}

/// What is being asked for, with the parameters that report needs.
///
/// A discriminated union rather than one struct with every optional field: it makes «a settlement
/// report without a settlement» impossible to express, which is the kind of request the legacy
/// report centre accepted and then failed on halfway through.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "reporte", rename_all = "PascalCase")]
pub enum ReporteRequest {
    #[serde(rename_all = "camelCase")]
    Movimientos {
        #[serde(default)]
        filtro: MovimientoFiltroDto,
    },
    #[serde(rename_all = "camelCase")]
    Liquidacion { id: Uuid },
    #[serde(rename_all = "camelCase")]
    Certificado { id: Uuid },
}

impl ReporteRequest {
    /// The formats this report can be produced in. Asking for another one is a validation error,
    /// not an empty file.
    #[must_use]
    pub fn formatos(&self) -> &'static [FormatoExport] {
        match self {
            ReporteRequest::Movimientos { .. } => &[
                FormatoExport::Pdf,
                FormatoExport::Xlsx,
                FormatoExport::Docx,
                FormatoExport::Csv,
                FormatoExport::Json,
            ],
            ReporteRequest::Liquidacion { .. } | ReporteRequest::Certificado { .. } => {
                &[FormatoExport::Pdf]
            }
        }
    }

    #[must_use]
    pub const fn nombre(&self) -> &'static str {
        match self {
            ReporteRequest::Movimientos { .. } => "Movimientos",
            ReporteRequest::Liquidacion { .. } => "Liquidacion",
            ReporteRequest::Certificado { .. } => "Certificado",
        }
    }
}

/// What the interface shows once the file is on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub ruta: String,
    pub bytes: u64,
    pub registros: u64,
}

/// One line of the "filters applied" text the report prints, already resolved to a label and a
/// value. The report only translates the label; resolving the identifier into a name is a database
/// question and belongs to the use case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiltroDescripcion {
    pub clave: String,
    pub valor: String,
}

/// Everything the movements report prints. The rows are the whole filter, unpaginated (doc 12 §1.2
/// rule 6): exporting the visible page was one of the legacy defects this replaces.
#[derive(Debug, Clone, PartialEq)]
pub struct ReporteMovimientos {
    pub items: Vec<MovimientoListItem>,
    pub resumen: MovimientoResumenDto,
    pub filtro: MovimientoFiltroDto,
    pub filtros_descripcion: Vec<FiltroDescripcion>,
}

/// A document generated in memory. Writing it is the caller's business, so no generator knows a
/// path and every layout test runs without touching the disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedReport {
    pub bytes: Vec<u8>,
    pub registros: u64,
    pub nombre_sugerido: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn los_movimientos_admiten_los_cinco_formatos() {
        let r = ReporteRequest::Movimientos {
            filtro: MovimientoFiltroDto::default(),
        };
        assert_eq!(r.formatos().len(), 5);
    }

    #[test]
    fn la_liquidacion_y_el_certificado_solo_admiten_pdf() {
        for r in [
            ReporteRequest::Liquidacion { id: Uuid::nil() },
            ReporteRequest::Certificado { id: Uuid::nil() },
        ] {
            assert_eq!(r.formatos(), &[FormatoExport::Pdf]);
        }
    }

    #[test]
    fn la_peticion_se_deserializa_por_su_discriminante() {
        let json = r#"{"reporte":"Liquidacion","id":"00000000-0000-0000-0000-000000000001"}"#;
        let parsed: ReporteRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed,
            ReporteRequest::Liquidacion {
                id: Uuid::from_u128(1)
            }
        );
    }

    #[test]
    fn los_movimientos_sin_filtro_explicito_se_deserializan_igual() {
        let parsed: ReporteRequest = serde_json::from_str(r#"{"reporte":"Movimientos"}"#).unwrap();
        assert!(matches!(parsed, ReporteRequest::Movimientos { .. }));
    }

    #[test]
    fn cada_formato_tiene_su_extension() {
        assert_eq!(FormatoExport::Xlsx.extension(), "xlsx");
        assert_eq!(FormatoExport::Pdf.extension(), "pdf");
    }
}
