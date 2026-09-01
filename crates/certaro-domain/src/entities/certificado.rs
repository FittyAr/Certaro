use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::decimal4::Decimal4;
use crate::entities::audit::Audit;
use crate::error::DomainError;
use crate::money::Money;

/// A progress certificate: what was certified, when, and for how much.
/// See `docs/05-dominio-entidades.md` §2.5.
///
/// Every amount here is **frozen** at the moment of issue. That is the whole point of the entity:
/// the legacy system overwrote the percentages on the item and kept no history, so the only copy
/// of a past certification was the PDF someone had printed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Certificado {
    pub id: Uuid,
    pub orden_trabajo_id: Uuid,
    /// Sequential within the order, from 1, and never reused (INV-15).
    pub numero: i32,
    pub fecha: NaiveDate,
    pub observaciones: Option<String>,
    pub total_certificado: Money,
    /// The UOCRA adjustment as the amount it came to, not the percentage it came from.
    pub ajuste_uocra: Money,
    pub otros_descuentos: Money,
    pub total_neto: Money,
    /// Loaded by the repository. Empty on a certificate read from a list query.
    pub items: Vec<CertificadoItem>,
    #[serde(flatten)]
    pub audit: Audit,
}

impl Certificado {
    /// Recomputes the net from the three frozen parts.
    ///
    /// Used by the tests and by the import verification: a stored `total_neto` that disagrees with
    /// this is corrupt data, not a rounding difference.
    pub fn total_neto_calculado(&self) -> Result<Money, DomainError> {
        self.total_certificado
            .checked_sub(self.ajuste_uocra)?
            .checked_sub(self.otros_descuentos)
    }

    /// Sum of what the lines certified. Must equal `total_certificado`.
    pub fn total_de_items(&self) -> Result<Money, DomainError> {
        Money::try_sum(
            self.items
                .iter()
                .filter(|i| !i.audit.is_deleted)
                .map(|i| i.subtotal_actual),
        )
    }
}

/// One certified line, with the item's values as they stood at the time.
/// See `docs/05-dominio-entidades.md` §2.6.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificadoItem {
    pub id: Uuid,
    pub certificado_id: Uuid,
    /// Kept so the item's history can be walked, but the numbers below are copies, not lookups:
    /// editing the quote afterwards must not rewrite what was certified.
    pub orden_trabajo_item_id: Uuid,
    pub cantidad: Decimal4,
    pub precio_unitario: Money,
    pub porcentaje_anterior: Decimal4,
    pub porcentaje_actual: Decimal4,
    pub subtotal_actual: Money,
    pub subtotal_acumulado: Money,
    #[serde(flatten)]
    pub audit: Audit,
}

impl CertificadoItem {
    pub fn porcentaje_acumulado(&self) -> Result<Decimal4, DomainError> {
        self.porcentaje_anterior.checked_add(self.porcentaje_actual)
    }
}
