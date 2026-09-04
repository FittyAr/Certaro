//! Use cases of the `dashboard`. See `docs/06-casos-de-uso-y-formulas.md` §9.
//!
//! Everything here is a read, and every figure is computed in SQL or in Rust but never in the
//! frontend: two screens that add the same column by hand eventually disagree.

use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::dtos::dashboard::{Alerta, DashboardStats, PeriodoDashboard};
use crate::ports::repositories::UnitOfWork;
use crate::ports::{ClockPort, SettingsStore};
use crate::result::AppResult;

mod alertas;
mod stats;
mod ventanas;

pub use ventanas::{calcular_ventanas, umbral_vencimiento, Ventanas, COMIENZO_DE_LOS_TIEMPOS};

pub struct DashboardService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    settings: Arc<dyn SettingsStore>,
}

impl DashboardService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        settings: Arc<dyn SettingsStore>,
    ) -> Self {
        Self {
            uow,
            clock,
            settings,
        }
    }

    pub fn ventanas(periodo: PeriodoDashboard, ahora: DateTime<Utc>) -> Ventanas {
        calcular_ventanas(periodo, ahora)
    }

    pub async fn stats(&self, periodo: PeriodoDashboard) -> AppResult<DashboardStats> {
        let ahora = self.clock.now_utc();
        stats::build_stats(self.uow.as_ref(), self.settings.as_ref(), ahora, periodo).await
    }

    pub async fn alertas(&self, periodo: PeriodoDashboard) -> AppResult<Vec<Alerta>> {
        let ahora = self.clock.now_utc();
        alertas::build_alertas(self.uow.as_ref(), self.settings.as_ref(), ahora, periodo).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use certaro_domain::{Decimal4, Money};
    use chrono::{Datelike, NaiveDate, TimeZone};
    use crate::dtos::dashboard::{RentabilidadItem, DashboardStats};

    fn instante(y: i32, m: u32, d: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, 12, 0, 0).single().unwrap()
    }

    #[test]
    fn el_periodo_anterior_es_una_ventana_del_mismo_largo() {
        let ahora = instante(2026, 8, 29);
        let v = DashboardService::ventanas(PeriodoDashboard::Mensual, ahora);
        let (desde_ant, hasta_ant) = v.anterior.unwrap();

        assert_eq!(v.hasta, ahora);
        assert_eq!(v.desde, instante(2026, 7, 29));
        // The previous window ends exactly where the current one starts, with no gap and no
        // overlap, and spans the same number of days so the comparison is not the calendar's.
        assert_eq!(hasta_ant, v.desde);
        assert_eq!(hasta_ant - desde_ant, v.hasta - v.desde);
        // 31 days back from 29 July, not "two calendar months back", which would be 29 June.
        assert_eq!(desde_ant, instante(2026, 6, 28));
    }

    #[test]
    fn el_periodo_anual_retrocede_doce_meses() {
        let ahora = instante(2026, 8, 29);
        let v = DashboardService::ventanas(PeriodoDashboard::Anual, ahora);
        assert_eq!(v.desde, instante(2025, 8, 29));
        let (desde_ant, hasta_ant) = v.anterior.unwrap();
        assert_eq!(hasta_ant, v.desde);
        assert_eq!(hasta_ant - desde_ant, v.hasta - v.desde);
    }

    #[test]
    fn el_periodo_total_no_tiene_comparacion() {
        let v = DashboardService::ventanas(PeriodoDashboard::Total, instante(2026, 8, 29));
        assert!(v.anterior.is_none());
        assert_eq!(v.desde.year(), COMIENZO_DE_LOS_TIEMPOS);
    }

    #[test]
    fn la_variacion_sin_base_es_ausente_y_no_infinito() {
        let cero = Money::ZERO;
        let algo = Money::parse("1000.0000").unwrap();

        assert_eq!(DashboardStats::variacion(cero, cero), Some(Decimal4::ZERO));
        assert_eq!(DashboardStats::variacion(cero, algo), None);
    }

    #[test]
    fn la_variacion_se_redondea_a_un_decimal() {
        let anterior = Money::parse("300.0000").unwrap();
        let actual = Money::parse("400.0000").unwrap();
        // 33.333… %
        assert_eq!(
            DashboardStats::variacion(anterior, actual),
            Some(Decimal4::parse("33.3").unwrap())
        );
    }

    #[test]
    fn el_margen_con_ingresos_en_cero_es_cero() {
        let gastos = Money::parse("500.0000").unwrap();
        assert_eq!(
            RentabilidadItem::margen(Money::ZERO, gastos.neg()),
            Decimal4::ZERO
        );
    }

    #[test]
    fn el_margen_se_redondea_a_dos_decimales() {
        let ingresos = Money::parse("3000.0000").unwrap();
        let rentabilidad = Money::parse("1000.0000").unwrap();
        assert_eq!(
            RentabilidadItem::margen(ingresos, rentabilidad),
            Decimal4::parse("33.33").unwrap()
        );
    }

    #[test]
    fn el_umbral_de_vencimiento_usa_los_dias_configurados() {
        let hoy = NaiveDate::from_ymd_opt(2026, 8, 29).unwrap();
        assert_eq!(
            umbral_vencimiento(hoy, 30),
            NaiveDate::from_ymd_opt(2026, 7, 30).unwrap()
        );
        assert_eq!(umbral_vencimiento(hoy, 0), hoy);
    }
}
