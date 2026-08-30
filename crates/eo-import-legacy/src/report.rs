//! The import report. See `docs/15-migracion-de-datos.md` §7.5.

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub tool_version: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub source: SourceInfo,
    pub target: TargetInfo,
    pub dry_run: bool,
    pub outcome: Outcome,
    pub tables: Vec<TableReport>,
    pub derived: DerivedReport,
    pub warnings: Vec<Warning>,
    pub blocking_issues: Vec<String>,
    pub attachments: AttachmentReport,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    pub path: String,
    pub schema_version: Option<String>,
    pub scale_state: ScaleState,
    pub integrity_check: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetInfo {
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum Outcome {
    Success,
    SuccessWithWarnings,
    Aborted,
    Rollback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum ScaleState {
    AlreadyScaled,
    UnscaledIntegers,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableReport {
    pub source: String,
    pub target: String,
    pub source_rows: u64,
    pub target_rows: u64,
    pub skipped: u64,
    pub monetary_sums: Vec<MonetarySum>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonetarySum {
    pub column: String,
    pub source: i64,
    pub target: i64,
    pub match_: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivedReport {
    pub certificados: u64,
    pub certificado_items: u64,
    pub liquidacion_adelantos: u64,
    pub contactos_creados: u64,
    pub feriados_recuperados: u64,
    pub facturas_reclasificadas: FacturasReclasificadas,
    pub vencimientos_estimados: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct FacturasReclasificadas {
    pub pagada: u64,
    pub pagada_parcial: u64,
    pub vencida: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Warning {
    pub code: WarningCode,
    pub table: String,
    pub row_id: Option<String>,
    pub detail: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WarningCode {
    PagoEscalaHeuristica,
    CotizacionEscalaHeuristica,
    CotizacionCeroDescartada,
    EscalaSinDecimales,
    AsistenciaColision,
    PorcentajeExcede100,
    AdelantoSumaDifiere,
    VencimientoEstimado,
    AdjuntoArchivoFalta,
    AdjuntoHuerfano,
    AdjuntoExcedeLimite,
    ColorHexInvalido,
    FeriadoNoParseable,
    ConceptoPagoIdDistinto,
    FkHuerfanaAnulada,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentReport {
    pub files_copied: u64,
    pub files_missing: u64,
    pub orphan_files: u64,
}

impl ImportReport {
    #[must_use]
    pub fn new(source: SourceInfo, target: TargetInfo, dry_run: bool) -> Self {
        Self {
            tool_version: env!("CARGO_PKG_VERSION").to_owned(),
            started_at: Utc::now(),
            finished_at: None,
            source,
            target,
            dry_run,
            outcome: Outcome::Success,
            tables: Vec::new(),
            derived: DerivedReport::default(),
            warnings: Vec::new(),
            blocking_issues: Vec::new(),
            attachments: AttachmentReport::default(),
        }
    }

    pub fn warn(&mut self, code: WarningCode, table: &str, row_id: Option<Uuid>, detail: serde_json::Value) {
        self.warnings.push(Warning {
            code,
            table: table.to_owned(),
            row_id: row_id.map(|id| id.to_string()),
            detail,
        });
    }

    pub fn block(&mut self, issue: String) {
        self.blocking_issues.push(issue);
    }

    #[must_use]
    pub fn has_blocking_issues(&self) -> bool {
        !self.blocking_issues.is_empty()
    }

    pub fn finish(&mut self) {
        self.finished_at = Some(Utc::now());
        if self.has_blocking_issues() {
            self.outcome = Outcome::Rollback;
        } else if !self.warnings.is_empty() {
            self.outcome = Outcome::SuccessWithWarnings;
        } else {
            self.outcome = Outcome::Success;
        }
    }
}
