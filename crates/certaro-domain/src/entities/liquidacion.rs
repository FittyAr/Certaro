use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::decimal4::Decimal4;
use crate::entities::audit::Audit;
use crate::error::DomainError;
use crate::money::Money;

/// The rules a settlement was computed with. See `docs/06-casos-de-uso-y-formulas.md` §6.1.
///
/// They are copied onto the row so a settlement can always be re-read with the rules that produced
/// it, even after configuration or the employee's card changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReglasLiquidacion {
    pub incluir_sabados: bool,
    pub incluir_domingos: bool,
    pub incluir_feriados: bool,
    pub multiplicador_sabado: Decimal4,
    pub multiplicador_domingo: Decimal4,
    pub multiplicador_feriado: Decimal4,
}

impl Default for ReglasLiquidacion {
    fn default() -> Self {
        Self {
            incluir_sabados: false,
            incluir_domingos: false,
            incluir_feriados: false,
            multiplicador_sabado: Decimal4::ONE,
            multiplicador_domingo: Decimal4::ONE,
            multiplicador_feriado: Decimal4::ONE,
        }
    }
}

impl ReglasLiquidacion {
    /// Multiplier of one calendar day. The order is strict — holiday beats Sunday beats Saturday —
    /// because a Sunday that is also a holiday has to pay as a holiday.
    /// See `docs/06-casos-de-uso-y-formulas.md` §6.3.
    ///
    /// A multiplier of zero or less means the day does not count at all: it adds neither days nor
    /// money.
    pub fn multiplicador_dia(&self, fecha: NaiveDate, feriados: &HashSet<NaiveDate>) -> Decimal4 {
        if feriados.contains(&fecha) {
            return if self.incluir_feriados {
                self.multiplicador_feriado
            } else {
                Decimal4::ZERO
            };
        }
        if fecha.weekday() == Weekday::Sun {
            return if self.incluir_domingos {
                self.multiplicador_domingo
            } else {
                Decimal4::ZERO
            };
        }
        if fecha.weekday() == Weekday::Sat {
            return if self.incluir_sabados {
                self.multiplicador_sabado
            } else {
                Decimal4::ZERO
            };
        }
        Decimal4::ONE
    }

    /// A day recorded as `TipoJornada::Feriado` does not consult the holiday calendar: the person
    /// who loaded it already said it was a holiday.
    pub fn multiplicador_jornada_feriado(&self) -> Decimal4 {
        if self.incluir_feriados {
            self.multiplicador_feriado
        } else {
            Decimal4::ZERO
        }
    }
}

/// A closed settlement. See `docs/05-dominio-entidades.md` §2.15.
///
/// `tarifa_aplicada`, `total_bruto` and `total_adelantos` are frozen copies: the employee was paid
/// this amount, and a later change to the rate must not rewrite history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Liquidacion {
    pub id: Uuid,
    pub empleado_id: Uuid,
    pub fecha_inicio: NaiveDate,
    pub fecha_fin: NaiveDate,
    /// Admits halves, so it is a decimal and not a count.
    pub dias_trabajados: Decimal4,
    pub tarifa_aplicada: Money,
    #[serde(flatten)]
    pub reglas: ReglasLiquidacion,
    pub total_bruto: Money,
    pub total_adelantos: Money,
    pub observaciones: Option<String>,
    /// Set the first time the PDF is handed over; from then on the amounts are read-only.
    pub pdf_generado_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Loaded by the repository, not a column.
    pub adelantos: Vec<LiquidacionAdelanto>,
    #[serde(flatten)]
    pub audit: Audit,
}

impl Liquidacion {
    /// Derived, never stored. A negative net is a real case — the employee took more in advances
    /// than they earned — and is shown as it is rather than clamped to zero.
    pub fn total_neto(&self) -> Result<Money, DomainError> {
        self.total_bruto.checked_sub(self.total_adelantos)
    }

    pub fn total_de_adelantos(&self) -> Result<Money, DomainError> {
        Money::try_sum(self.adelantos.iter().map(|a| a.monto))
    }

    /// Once the PDF is out, the amounts are what the employee was handed.
    pub fn admite_cambio_de_importes(&self) -> bool {
        self.pdf_generado_at.is_none()
    }
}

/// One advance consumed by a settlement. See `docs/05-dominio-entidades.md` §2.16.
///
/// Amount, date and concept are frozen: the PDF lists them line by line with their own date (RC-02)
/// and editing the original movement afterwards must not change a document already delivered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidacionAdelanto {
    pub id: Uuid,
    pub liquidacion_id: Uuid,
    pub movimiento_id: Uuid,
    pub monto: Money,
    pub fecha: NaiveDate,
    pub concepto: String,
    #[serde(flatten)]
    pub audit: Audit,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn dia(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn reglas() -> ReglasLiquidacion {
        ReglasLiquidacion {
            incluir_sabados: true,
            incluir_domingos: true,
            incluir_feriados: true,
            multiplicador_sabado: Decimal4::parse("1.5").unwrap(),
            multiplicador_domingo: Decimal4::from_units(2).unwrap(),
            multiplicador_feriado: Decimal4::from_units(3).unwrap(),
        }
    }

    #[test]
    fn un_dia_habil_vale_uno() {
        // 2026-08-26 is a Wednesday.
        let feriados = HashSet::new();
        assert_eq!(
            reglas().multiplicador_dia(dia(2026, 8, 26), &feriados),
            Decimal4::ONE
        );
    }

    #[test]
    fn el_feriado_gana_sobre_el_domingo() {
        // 2026-08-30 is a Sunday; as a holiday it has to pay the holiday multiplier.
        let feriados = HashSet::from([dia(2026, 8, 30)]);
        assert_eq!(
            reglas().multiplicador_dia(dia(2026, 8, 30), &feriados),
            Decimal4::from_units(3).unwrap()
        );
    }

    #[test]
    fn el_domingo_gana_sobre_el_sabado() {
        let feriados = HashSet::new();
        let r = reglas();
        assert_eq!(
            r.multiplicador_dia(dia(2026, 8, 30), &feriados),
            Decimal4::from_units(2).unwrap()
        );
        assert_eq!(
            r.multiplicador_dia(dia(2026, 8, 29), &feriados),
            Decimal4::parse("1.5").unwrap()
        );
    }

    #[test]
    fn un_dia_excluido_no_computa() {
        let feriados = HashSet::from([dia(2026, 8, 30)]);
        let r = ReglasLiquidacion::default();
        // 29 = Saturday, 30 = Sunday and holiday.
        assert_eq!(
            r.multiplicador_dia(dia(2026, 8, 29), &feriados),
            Decimal4::ZERO
        );
        assert_eq!(
            r.multiplicador_dia(dia(2026, 8, 30), &feriados),
            Decimal4::ZERO
        );
        assert_eq!(r.multiplicador_jornada_feriado(), Decimal4::ZERO);
    }

    fn liquidacion(bruto: &str, adelantos: &str) -> Liquidacion {
        Liquidacion {
            id: Uuid::nil(),
            empleado_id: Uuid::nil(),
            fecha_inicio: dia(2026, 8, 1),
            fecha_fin: dia(2026, 8, 15),
            dias_trabajados: Decimal4::from_units(10).unwrap(),
            tarifa_aplicada: Money::from_units(40_000).unwrap(),
            reglas: ReglasLiquidacion::default(),
            total_bruto: Money::parse(bruto).unwrap(),
            total_adelantos: Money::parse(adelantos).unwrap(),
            observaciones: None,
            pdf_generado_at: None,
            adelantos: Vec::new(),
            audit: Audit::new(Utc.with_ymd_and_hms(2026, 8, 16, 0, 0, 0).unwrap()),
        }
    }

    #[test]
    fn el_neto_resta_los_adelantos() {
        // RC-01: 400 000 - 260 000 = 140 000
        let l = liquidacion("400000", "260000");
        assert_eq!(l.total_neto().unwrap(), Money::from_units(140_000).unwrap());
    }

    #[test]
    fn el_neto_puede_ser_negativo() {
        // Taking more in advances than earned is a real case and is reported as such.
        let l = liquidacion("100000", "150000");
        assert_eq!(l.total_neto().unwrap(), Money::from_units(-50_000).unwrap());
    }

    #[test]
    fn una_liquidacion_entregada_no_admite_cambio_de_importes() {
        let mut l = liquidacion("400000", "0");
        assert!(l.admite_cambio_de_importes());
        l.pdf_generado_at = Some(Utc.with_ymd_and_hms(2026, 8, 17, 12, 0, 0).unwrap());
        assert!(!l.admite_cambio_de_importes());
    }
}
