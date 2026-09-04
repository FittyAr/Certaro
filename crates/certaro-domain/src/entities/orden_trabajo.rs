use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::decimal4::Decimal4;
use crate::entities::audit::Audit;
use crate::error::DomainError;
use crate::money::Money;

/// The itemised quote of a job, and the thing certificates are issued against.
/// See `docs/05-dominio-entidades.md` §2.15.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrdenTrabajo {
    pub id: Uuid,
    pub trabajo_id: Uuid,
    pub titulo: String,
    /// Number of the last certificate issued. Written by the issuing use case, never by the form.
    pub numero_certificado: Option<String>,
    pub fecha: NaiveDate,
    pub observaciones: Option<String>,
    /// A percentage: the `8` of the paper sheet is stored as `8.0000`, not as `0.08`.
    pub ajuste_uocra_porcentaje: Decimal4,
    /// Already an amount, unlike the UOCRA adjustment.
    pub otros_descuentos: Money,
    /// Loaded by the repository. Empty on an order read from a list query.
    pub items: Vec<OrdenTrabajoItem>,
    #[serde(flatten)]
    pub audit: Audit,
}

impl OrdenTrabajo {
    fn items_vivos(&self) -> impl Iterator<Item = &OrdenTrabajoItem> {
        self.items.iter().filter(|i| !i.audit.is_deleted)
    }

    /// Full value of the quote, at 100 % of every item. Not what a certificate pays.
    pub fn total_presupuestado(&self) -> Result<Money, DomainError> {
        self.items_vivos()
            .try_fold(Money::ZERO, |acc, item| acc.checked_add(item.base()?))
    }

    /// What would be certified right now: the sum of the current progress of every item.
    /// See `docs/06-casos-de-uso-y-formulas.md` §5.4.
    pub fn total_certificado(&self) -> Result<Money, DomainError> {
        self.items_vivos().try_fold(Money::ZERO, |acc, item| {
            acc.checked_add(item.subtotales()?.0)
        })
    }

    /// The UOCRA adjustment as an amount. It is an addition/escalation on the certified amount.
    pub fn ajuste_uocra(&self) -> Result<Money, DomainError> {
        self.total_certificado()?
            .percent(self.ajuste_uocra_porcentaje)
    }

    pub fn total_neto(&self) -> Result<Money, DomainError> {
        self.total_certificado()?
            .checked_add(self.ajuste_uocra()?)?
            .checked_sub(self.otros_descuentos)
    }

    /// The UOCRA adjustment calculated on the full budgeted quote amount.
    pub fn ajuste_uocra_presupuestado(&self) -> Result<Money, DomainError> {
        self.total_presupuestado()?
            .percent(self.ajuste_uocra_porcentaje)
    }

    /// Full net value of the budgeted quote: total_presupuestado + ajuste_uocra_presupuestado - otros_descuentos.
    pub fn total_presupuestado_neto(&self) -> Result<Money, DomainError> {
        self.total_presupuestado()?
            .checked_add(self.ajuste_uocra_presupuestado()?)?
            .checked_sub(self.otros_descuentos)
    }

    /// Whether anything at all would be certified. An order with no progress cannot be issued.
    pub fn tiene_avance(&self) -> bool {
        self.items_vivos()
            .any(|i| i.porcentaje_actual.is_positive())
    }
}

/// One line of the quote. See `docs/05-dominio-entidades.md` §2.16.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrdenTrabajoItem {
    pub id: Uuid,
    pub orden_trabajo_id: Uuid,
    pub descripcion: String,
    /// `"u"`, `"m"`, `"ml"`, `"gl"`… free text, because the sheet uses whatever fits.
    pub unidad: String,
    pub cantidad: Decimal4,
    pub precio_unitario: Money,
    /// Accumulated across the certificates already issued. Read-only for the user.
    pub porcentaje_anterior: Decimal4,
    /// Progress of the certificate being prepared. Reset to zero when it is issued.
    pub porcentaje_actual: Decimal4,
    /// "The work was done" (RC-11), independent of the money.
    pub ejecutado: bool,
    pub nota: Option<String>,
    /// Position in the sheet. The user reorders lines and the printed order has to match.
    pub orden: i32,
    #[serde(flatten)]
    pub audit: Audit,
}

impl OrdenTrabajoItem {
    /// `docs/06-casos-de-uso-y-formulas.md` §5.1.
    pub fn porcentaje_acumulado(&self) -> Result<Decimal4, DomainError> {
        self.porcentaje_anterior.checked_add(self.porcentaje_actual)
    }

    /// Full value of the line, before any percentage.
    pub fn base(&self) -> Result<Money, DomainError> {
        self.precio_unitario.checked_mul(self.cantidad)
    }

    /// `(subtotal_actual, subtotal_acumulado)`. See `docs/06-casos-de-uso-y-formulas.md` §5.2.
    ///
    /// The order of operations is not free: the product `cantidad × precio_unitario` is rounded
    /// first and the percentage applied to that. Doing it the other way round moves the last
    /// decimal, and the legacy system did it this way, so the historical numbers depend on it.
    pub fn subtotales(&self) -> Result<(Money, Money), DomainError> {
        let base = self.base()?;
        Ok((
            base.percent(self.porcentaje_actual)?,
            base.percent(self.porcentaje_acumulado()?)?,
        ))
    }

    /// What answers RC-11, "why was this left pending?". Shown next to the note.
    pub fn porcentaje_pendiente(&self) -> Result<Decimal4, DomainError> {
        Decimal4::HUNDRED.checked_sub(self.porcentaje_acumulado()?)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;

    fn ahora() -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(0, 0).unwrap()
    }

    fn item(cantidad: &str, precio: &str, anterior: &str, actual: &str) -> OrdenTrabajoItem {
        OrdenTrabajoItem {
            id: Uuid::from_u128(1),
            orden_trabajo_id: Uuid::from_u128(2),
            descripcion: "cableado".into(),
            unidad: "m".into(),
            cantidad: Decimal4::parse(cantidad).unwrap(),
            precio_unitario: Money::parse(precio).unwrap(),
            porcentaje_anterior: Decimal4::parse(anterior).unwrap(),
            porcentaje_actual: Decimal4::parse(actual).unwrap(),
            ejecutado: false,
            nota: None,
            orden: 0,
            audit: Audit::new(ahora()),
        }
    }

    fn orden(items: Vec<OrdenTrabajoItem>, uocra: &str, otros: &str) -> OrdenTrabajo {
        OrdenTrabajo {
            id: Uuid::from_u128(2),
            trabajo_id: Uuid::from_u128(3),
            titulo: "Planilla 1".into(),
            numero_certificado: None,
            fecha: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            observaciones: None,
            ajuste_uocra_porcentaje: Decimal4::parse(uocra).unwrap(),
            otros_descuentos: Money::parse(otros).unwrap(),
            items,
            audit: Audit::new(ahora()),
        }
    }

    /// The case the user brought to the meeting (RC-09), digit for digit.
    #[test]
    fn el_caso_real_de_la_planilla() {
        let i = item("4200", "1000", "0", "60");
        assert_eq!(i.base().unwrap(), Money::parse("4200000").unwrap());
        assert_eq!(
            i.porcentaje_acumulado().unwrap(),
            Decimal4::parse("60").unwrap()
        );
        let (actual, acumulado) = i.subtotales().unwrap();
        assert_eq!(actual, Money::parse("2520000").unwrap());
        assert_eq!(acumulado, Money::parse("2520000").unwrap());
        assert_eq!(
            i.porcentaje_pendiente().unwrap(),
            Decimal4::parse("40").unwrap()
        );
    }

    #[test]
    fn el_acumulado_suma_lo_anterior() {
        let i = item("100", "50", "30", "20");
        let (actual, acumulado) = i.subtotales().unwrap();
        // base 5000: 20 % es 1000, 50 % acumulado es 2500.
        assert_eq!(actual, Money::parse("1000").unwrap());
        assert_eq!(acumulado, Money::parse("2500").unwrap());
        assert_eq!(
            i.porcentaje_pendiente().unwrap(),
            Decimal4::parse("50").unwrap()
        );
    }

    /// A third of an amount keeps its four decimals: the percentage is applied to the amount, not
    /// converted to a fraction first.
    #[test]
    fn el_porcentaje_no_pierde_decimales() {
        let i = item("1", "1000", "0", "33.3333");
        assert_eq!(i.subtotales().unwrap().0, Money::parse("333.3330").unwrap());
    }

    #[test]
    fn el_ajuste_uocra_suma_al_neto() {
        let o = orden(vec![item("4200", "1000", "0", "60")], "8", "20000");
        assert_eq!(
            o.total_certificado().unwrap(),
            Money::parse("2520000").unwrap()
        );
        assert_eq!(o.ajuste_uocra().unwrap(), Money::parse("201600").unwrap());
        // 2520000 + 201600 - 20000 = 2701600
        assert_eq!(o.total_neto().unwrap(), Money::parse("2701600").unwrap());
    }

    #[test]
    fn el_total_presupuestado_neto_calcula_sobre_la_cotizacion_completa() {
        // Presupuesto: 10 * 100 = 1000. UOCRA: 10% = 100. Otros descuentos: 50.
        // Total presupuestado neto = 1000 + 100 - 50 = 1050.
        // Avance actual: 0%. Total certificado = 0.
        let o = orden(vec![item("10", "100", "0", "0")], "10", "50");
        assert_eq!(o.total_presupuestado().unwrap(), Money::parse("1000").unwrap());
        assert_eq!(o.ajuste_uocra_presupuestado().unwrap(), Money::parse("100").unwrap());
        assert_eq!(o.total_presupuestado_neto().unwrap(), Money::parse("1050").unwrap());
        assert_eq!(o.total_certificado().unwrap(), Money::ZERO);
    }

    #[test]
    fn el_total_presupuestado_ignora_los_porcentajes() {
        let o = orden(
            vec![item("10", "100", "0", "10"), item("2", "50", "0", "0")],
            "0",
            "0",
        );
        assert_eq!(
            o.total_presupuestado().unwrap(),
            Money::parse("1100").unwrap()
        );
        assert_eq!(o.total_certificado().unwrap(), Money::parse("100").unwrap());
    }

    #[test]
    fn un_item_borrado_no_suma() {
        let mut borrado = item("10", "100", "0", "100");
        borrado.audit.soft_delete(ahora());
        let o = orden(vec![item("1", "100", "0", "100"), borrado], "0", "0");
        assert_eq!(o.total_certificado().unwrap(), Money::parse("100").unwrap());
    }

    #[test]
    fn sin_avance_no_hay_nada_que_certificar() {
        assert!(!orden(vec![item("10", "100", "50", "0")], "0", "0").tiene_avance());
        assert!(orden(vec![item("10", "100", "50", "1")], "0", "0").tiene_avance());
    }
}
