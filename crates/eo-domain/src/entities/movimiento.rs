use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::constants;
use crate::decimal4::Decimal4;
use crate::entities::audit::Audit;
use crate::enums::Moneda;
use crate::error::DomainError;
use crate::money::Money;

/// A cash movement. See `docs/05-dominio-entidades.md` §2.13.
///
/// The sign is **not** here: it lives in `tipos_movimiento.es_ingreso`, so any aggregation has to
/// join. Keeping a sign on the movement as well would give two sources of truth that can disagree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Movimiento {
    pub id: Uuid,
    /// An instant, not a civil date: the time of day is part of the record.
    pub fecha: DateTime<Utc>,
    pub concepto: String,
    /// Unit amount. The total is `monto * cantidad` and is never stored (INV-01).
    pub monto: Money,
    /// Defaults to `1.0` (RC-03), so an ordinary movement needs no quantity at all.
    pub cantidad: Decimal4,
    pub tipo_movimiento_id: Uuid,
    pub moneda: Moneda,
    pub cotizacion_aplicada: Option<Money>,
    pub tipo_concepto_pago_id: Option<Uuid>,
    pub categoria_id: Option<Uuid>,
    pub cliente_id: Option<Uuid>,
    /// How a movement is charged to a site: through the job, never directly.
    pub trabajo_id: Option<Uuid>,
    pub empleado_id: Option<Uuid>,
    pub factura_id: Option<Uuid>,
    #[serde(flatten)]
    pub audit: Audit,
}

impl Movimiento {
    /// INV-01. Computed on every read; storing it would let the stored value drift from its parts.
    pub fn total(&self) -> Result<Money, DomainError> {
        self.monto.checked_mul(self.cantidad)
    }

    /// An advance is identified by the seeded type, not by its name: a user renaming the row must
    /// not break the payroll that discounts it.
    pub fn es_adelanto(&self) -> bool {
        self.tipo_movimiento_id == constants::tipos_movimiento::ADELANTO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn movimiento(monto: &str, cantidad: &str) -> Movimiento {
        Movimiento {
            id: Uuid::from_u128(1),
            fecha: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
            concepto: "Cable 2.5".into(),
            monto: Money::parse(monto).unwrap(),
            cantidad: Decimal4::parse(cantidad).unwrap(),
            tipo_movimiento_id: constants::tipos_movimiento::GASTO,
            moneda: Moneda::Ars,
            cotizacion_aplicada: None,
            tipo_concepto_pago_id: None,
            categoria_id: None,
            cliente_id: None,
            trabajo_id: None,
            empleado_id: None,
            factura_id: None,
            audit: Audit::new(DateTime::<Utc>::from_timestamp(0, 0).unwrap()),
        }
    }

    /// The verifiable examples of `docs/06-casos-de-uso-y-formulas.md` §3.1, taken as written.
    #[test]
    fn el_total_es_el_monto_por_la_cantidad() {
        let casos = [
            ("40000.0000", "1.0000", "40000.0000"),
            ("1500.5000", "2.0000", "3001.0000"),
            ("1200.0000", "5.0000", "6000.0000"),
            ("333.3333", "3.0000", "999.9999"),
        ];
        for (monto, cantidad, esperado) in casos {
            assert_eq!(
                movimiento(monto, cantidad)
                    .total()
                    .unwrap()
                    .to_decimal_string(),
                esperado,
                "{monto} x {cantidad}"
            );
        }
    }

    #[test]
    fn el_ultimo_decimal_redondea_alejandose_del_cero() {
        // 0.0001 x 0.5 is 0.00005 exactly, which rounds up rather than to even.
        assert_eq!(
            movimiento("0.0001", "0.5000")
                .total()
                .unwrap()
                .to_decimal_string(),
            "0.0001"
        );
    }

    #[test]
    fn un_adelanto_se_reconoce_por_el_tipo_sembrado() {
        let mut m = movimiento("1000.0000", "1.0000");
        assert!(!m.es_adelanto());
        m.tipo_movimiento_id = constants::tipos_movimiento::ADELANTO;
        assert!(m.es_adelanto());
    }
}
