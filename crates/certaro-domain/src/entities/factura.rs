use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::audit::Audit;
use crate::enums::EstadoFactura;
use crate::error::DomainError;
use crate::money::Money;

/// An invoice issued to a customer. See `docs/05-dominio-entidades.md` §2.10.
///
/// `total` is stored even though it is derivable. It is a deliberate denormalisation: the user
/// copies the total from the paper, the system overwrites it with `subtotal + iva` on every save,
/// and the receivables queries get to read one column instead of adding two.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Factura {
    pub id: Uuid,
    /// Not unique: the legacy data has repeated numbers across points of sale.
    pub numero: String,
    pub fecha: NaiveDate,
    pub fecha_vencimiento: Option<NaiveDate>,
    pub cliente_id: Uuid,
    pub estado: EstadoFactura,
    pub subtotal: Money,
    /// An amount, not a rate, and never computed: it is copied from the paper.
    pub iva: Money,
    pub total: Money,
    pub observaciones: Option<String>,
    /// Loaded by the repository. Empty on an invoice read from a list query.
    pub pagos: Vec<PagoFactura>,
    #[serde(flatten)]
    pub audit: Audit,
}

impl Factura {
    pub fn total_calculado(&self) -> Result<Money, DomainError> {
        self.subtotal.checked_add(self.iva)
    }

    pub fn total_pagado(&self) -> Result<Money, DomainError> {
        Money::try_sum(
            self.pagos
                .iter()
                .filter(|p| !p.audit.is_deleted)
                .map(|p| p.monto),
        )
    }

    /// Can go negative when someone overpays. It is not clamped: a credit in favour of the
    /// customer is real information and hiding it would make the account impossible to reconcile.
    pub fn saldo_pendiente(&self) -> Result<Money, DomainError> {
        self.total.checked_sub(self.total_pagado()?)
    }

    pub fn esta_saldada(&self) -> Result<bool, DomainError> {
        Ok(self.saldo_pendiente()?.raw() <= 0)
    }

    /// The due date, falling back to `fecha + dias_default` when the column is empty. Most legacy
    /// invoices have no due date, and treating them as never due would empty the arrears report.
    pub fn vencimiento_efectivo(&self, dias_default: u32) -> NaiveDate {
        self.fecha_vencimiento
            .unwrap_or_else(|| self.fecha + chrono::Duration::days(i64::from(dias_default)))
    }

    /// Overdue means: it counts as debt, it still owes something, and its due date has passed.
    pub fn esta_vencida(&self, hoy: NaiveDate, dias_default: u32) -> Result<bool, DomainError> {
        if !self.estado.cuenta_como_deuda() || self.esta_saldada()? {
            return Ok(false);
        }
        Ok(hoy > self.vencimiento_efectivo(dias_default))
    }

    /// Days past due, never negative: an invoice not yet due is zero days late, not "-5".
    pub fn dias_mora(&self, hoy: NaiveDate, dias_default: u32) -> Result<i64, DomainError> {
        if !self.esta_vencida(hoy, dias_default)? {
            return Ok(0);
        }
        Ok((hoy - self.vencimiento_efectivo(dias_default)).num_days())
    }
}

/// A payment imputed to an invoice. See `docs/05-dominio-entidades.md` §2.17.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PagoFactura {
    pub id: Uuid,
    pub factura_id: Uuid,
    pub fecha: NaiveDate,
    pub monto: Money,
    /// Free text in the database. The interface offers the [`MedioPago`](crate::enums) options,
    /// but a historical value outside that list is shown as it was written.
    pub medio_pago: String,
    #[serde(flatten)]
    pub audit: Audit,
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;

    fn ahora() -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(0, 0).unwrap()
    }

    fn dia(d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 1, d).unwrap()
    }

    fn pago(monto: &str) -> PagoFactura {
        PagoFactura {
            id: Uuid::new_v4(),
            factura_id: Uuid::from_u128(1),
            fecha: dia(1),
            monto: Money::parse(monto).unwrap(),
            medio_pago: "Efectivo".into(),
            audit: Audit::new(ahora()),
        }
    }

    fn factura(total: &str, pagos: Vec<PagoFactura>) -> Factura {
        Factura {
            id: Uuid::from_u128(1),
            numero: "0001-00000001".into(),
            fecha: dia(1),
            fecha_vencimiento: None,
            cliente_id: Uuid::from_u128(2),
            estado: EstadoFactura::Emitida,
            subtotal: Money::parse(total).unwrap(),
            iva: Money::ZERO,
            total: Money::parse(total).unwrap(),
            observaciones: None,
            pagos,
            audit: Audit::new(ahora()),
        }
    }

    #[test]
    fn el_total_es_la_suma_de_sus_partes() {
        let mut f = factura("1000", vec![]);
        f.subtotal = Money::parse("1000").unwrap();
        f.iva = Money::parse("210").unwrap();
        assert_eq!(f.total_calculado().unwrap(), Money::parse("1210").unwrap());
    }

    #[test]
    fn un_pago_borrado_no_suma() {
        let mut borrado = pago("400");
        borrado.audit.soft_delete(ahora());
        let f = factura("1000", vec![pago("100"), borrado]);
        assert_eq!(f.total_pagado().unwrap(), Money::parse("100").unwrap());
        assert_eq!(f.saldo_pendiente().unwrap(), Money::parse("900").unwrap());
    }

    #[test]
    fn el_saldo_puede_ser_negativo() {
        let f = factura("1000", vec![pago("1200")]);
        assert_eq!(f.saldo_pendiente().unwrap(), Money::parse("-200").unwrap());
        assert!(f.esta_saldada().unwrap());
    }

    #[test]
    fn sin_columna_de_vencimiento_se_usa_el_default() {
        let f = factura("1000", vec![]);
        assert_eq!(
            f.vencimiento_efectivo(30),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap()
        );
        assert!(!f.esta_vencida(dia(31), 30).unwrap());
        assert!(f
            .esta_vencida(NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(), 30)
            .unwrap());
    }

    #[test]
    fn una_factura_saldada_nunca_esta_vencida() {
        let f = factura("1000", vec![pago("1000")]);
        let tarde = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        assert!(!f.esta_vencida(tarde, 30).unwrap());
        assert_eq!(f.dias_mora(tarde, 30).unwrap(), 0);
    }

    #[test]
    fn una_anulada_no_devenga_mora() {
        let mut f = factura("1000", vec![]);
        f.estado = EstadoFactura::Anulada;
        let tarde = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        assert!(!f.esta_vencida(tarde, 30).unwrap());
    }

    #[test]
    fn la_mora_se_cuenta_desde_el_vencimiento() {
        let mut f = factura("1000", vec![]);
        f.fecha_vencimiento = Some(dia(10));
        assert_eq!(f.dias_mora(dia(15), 30).unwrap(), 5);
        assert_eq!(f.dias_mora(dia(10), 30).unwrap(), 0);
    }
}
