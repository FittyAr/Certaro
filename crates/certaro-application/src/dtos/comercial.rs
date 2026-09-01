//! Contract of the commercial analysis: account statement, ageing and profitability.
//! See `docs/11-contratos-tauri.md` §5.2 and `docs/06-casos-de-uso-y-formulas.md` §4.5, §4.6 y §7.

use chrono::NaiveDate;
use certaro_domain::{EstadoFactura, Money};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CuentaCorrienteQuery {
    pub cliente_id: Uuid,
    /// Adds the settled invoices, which the screen hides by default: the statement is about what
    /// is owed, and a paid invoice is history.
    #[serde(default)]
    pub incluir_pagadas: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CuentaCorrienteFactura {
    pub id: Uuid,
    pub numero: String,
    pub fecha: NaiveDate,
    pub fecha_vencimiento: Option<NaiveDate>,
    pub estado: EstadoFactura,
    pub total: Money,
    pub pagado: Money,
    pub saldo: Money,
    /// Days past the due date, or past the issue date when none was loaded. Zero for a settled
    /// invoice: a paid row is not in arrears (doc 06 §4.5).
    pub dias_mora: i64,
}

/// A customer's statement. An unknown customer yields an empty statement rather than an error:
/// that is what the legacy service did and what the screen expects (doc 06 §4.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CuentaCorriente {
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
    pub total_facturado: Money,
    pub total_pagado: Money,
    pub saldo: Money,
    pub facturas: Vec<CuentaCorrienteFactura>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AntiguedadDeudaQuery {
    /// Defaults to today in the configured timezone.
    pub fecha_corte: Option<NaiveDate>,
    pub cliente_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AntiguedadDeudaCliente {
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
    pub total: Money,
    pub bucket0a30: Money,
    pub bucket31a60: Money,
    pub bucket61a90: Money,
    pub bucket_mas90: Money,
}

/// The ageing report. The buckets always add up to the total, which is the invariant its test
/// checks (doc 06 §4.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AntiguedadDeuda {
    pub fecha_corte: NaiveDate,
    pub total: Money,
    pub bucket0a30: Money,
    pub bucket31a60: Money,
    pub bucket61a90: Money,
    pub bucket_mas90: Money,
    /// Upper bound of each closed bucket, in days. Sent so the screen labels the columns from the
    /// configuration instead of hardcoding `30 / 60 / 90`.
    pub limites: Vec<u32>,
    pub detalle: Vec<AntiguedadDeudaCliente>,
}

/// Accumulator shared by the report and its per-customer breakdown.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BucketsDeuda {
    pub total: Money,
    pub b0a30: Money,
    pub b31a60: Money,
    pub b61a90: Money,
    pub mas90: Money,
}

impl BucketsDeuda {
    /// Adds a balance to the bucket its age falls into. The bounds are inclusive on the upper
    /// side: 30 days is still `0-30` and 31 is already `31-60`.
    pub fn add(
        &mut self,
        dias: i64,
        saldo: Money,
        limites: &[u32],
    ) -> Result<(), certaro_domain::DomainError> {
        let bucket = match limites {
            [a, b, c, ..] => {
                if dias <= i64::from(*a) {
                    &mut self.b0a30
                } else if dias <= i64::from(*b) {
                    &mut self.b31a60
                } else if dias <= i64::from(*c) {
                    &mut self.b61a90
                } else {
                    &mut self.mas90
                }
            }
            // A misconfigured list must not lose money: everything lands in the open bucket.
            _ => &mut self.mas90,
        };
        *bucket = bucket.checked_add(saldo)?;
        self.total = self.total.checked_add(saldo)?;
        Ok(())
    }
}
