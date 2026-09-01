use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::decimal4::Decimal4;
use crate::entities::audit::Audit;
use crate::enums::FrecuenciaPago;
use crate::error::DomainError;
use crate::money::Money;

/// Someone on the payroll. See `docs/05-dominio-entidades.md` §2.7.
///
/// `tarifa_diaria` is what the settlement multiplies; `sueldo_base` only exists to suggest that
/// rate when the employee is created. The three multipliers live here rather than only in
/// configuration because a foreman and a helper are not paid the same overtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Empleado {
    pub id: Uuid,
    pub nombre: String,
    pub dni: Option<String>,
    pub cargo: Option<String>,
    pub sueldo_base: Money,
    pub pago_frecuencia: FrecuenciaPago,
    pub tarifa_diaria: Money,
    pub multiplicador_sabado: Decimal4,
    pub multiplicador_domingo: Decimal4,
    pub multiplicador_feriado: Decimal4,
    pub email: Option<String>,
    pub telefono: Option<String>,
    /// Civil dates: what matters is the day.
    pub fecha_ingreso: NaiveDate,
    pub fecha_egreso: Option<NaiveDate>,
    pub activo: bool,
    #[serde(flatten)]
    pub audit: Audit,
}

impl Empleado {
    /// Only a suggestion for the form. The settlement never derives the rate: it reads
    /// `tarifa_diaria`, because that is the number the employee agreed to.
    pub fn tarifa_diaria_sugerida(&self) -> Result<Money, DomainError> {
        self.sueldo_base
            .checked_div(self.pago_frecuencia.dias_por_periodo())
    }

    /// An employee who left cannot be given attendance after that day.
    /// See `docs/08-maquinas-de-estado.md` §5.2.
    pub fn admite_asistencia_en(&self, fecha: NaiveDate) -> bool {
        match self.fecha_egreso {
            Some(egreso) => fecha <= egreso,
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::audit::Audit;
    use chrono::{TimeZone, Utc};

    fn empleado(sueldo: &str, frecuencia: FrecuenciaPago) -> Empleado {
        Empleado {
            id: Uuid::nil(),
            nombre: "Juan".into(),
            dni: None,
            cargo: None,
            sueldo_base: Money::parse(sueldo).unwrap(),
            pago_frecuencia: frecuencia,
            tarifa_diaria: Money::ZERO,
            multiplicador_sabado: Decimal4::ONE,
            multiplicador_domingo: Decimal4::ONE,
            multiplicador_feriado: Decimal4::ONE,
            email: None,
            telefono: None,
            fecha_ingreso: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            fecha_egreso: None,
            activo: true,
            audit: Audit::new(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()),
        }
    }

    #[test]
    fn la_tarifa_sugerida_divide_por_los_dias_del_periodo() {
        // 1 200 000 / 30 = 40 000
        let e = empleado("1200000", FrecuenciaPago::Mensual);
        assert_eq!(
            e.tarifa_diaria_sugerida().unwrap(),
            Money::from_units(40_000).unwrap()
        );
    }

    #[test]
    fn la_frecuencia_semanal_divide_por_seis() {
        // 240 000 / 6 = 40 000
        let e = empleado("240000", FrecuenciaPago::Semanal);
        assert_eq!(
            e.tarifa_diaria_sugerida().unwrap(),
            Money::from_units(40_000).unwrap()
        );
    }

    #[test]
    fn un_empleado_egresado_no_admite_asistencia_posterior() {
        let mut e = empleado("0", FrecuenciaPago::Mensual);
        e.fecha_egreso = Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap());
        assert!(e.admite_asistencia_en(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()));
        assert!(!e.admite_asistencia_en(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()));
    }

    #[test]
    fn sin_fecha_de_egreso_cualquier_dia_es_valido() {
        let e = empleado("0", FrecuenciaPago::Mensual);
        assert!(e.admite_asistencia_en(NaiveDate::from_ymd_opt(2030, 1, 1).unwrap()));
    }
}
