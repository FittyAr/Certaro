//! Use cases of the `dashboard`. See `docs/06-casos-de-uso-y-formulas.md` §9.
//!
//! Everything here is a read, and every figure is computed in SQL or in Rust but never in the
//! frontend: two screens that add the same column by hand eventually disagree.

use std::sync::Arc;

use chrono::{DateTime, Datelike, Months, NaiveDate, TimeZone, Utc};
use certaro_domain::Money;
use tracing::warn;

use crate::dtos::dashboard::{
    Alerta, DashboardStats, EstadoSistema, PeriodoDashboard, PuntoSerie, RentabilidadItem,
    SeveridadAlerta, TipoAlerta, TopCliente,
};
use crate::dtos::movimientos::{MovimientoFiltroDto, MovimientoListItem};
use crate::ports::repositories::{SortDir, UnitOfWork};
use crate::ports::{ClockPort, SettingsStore};
use crate::result::AppResult;
use crate::use_cases::shared::finish_read;

/// Where `Total` starts counting from. The validation floor for any date is the year 2000, so this
/// is safely before every record and, unlike `DateTime::MIN_UTC`, it still formats as a normal
/// timestamp for the comparison the query does.
const COMIENZO_DE_LOS_TIEMPOS: i32 = 1900;

pub struct DashboardService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    settings: Arc<dyn SettingsStore>,
}

/// The current window and the one it is compared against. `Total` has no previous window, and its
/// comparison is reported as absent rather than as zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ventanas {
    pub desde: DateTime<Utc>,
    pub hasta: DateTime<Utc>,
    pub anterior: Option<(DateTime<Utc>, DateTime<Utc>)>,
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

    /// Rolling windows, not calendar months: "monthly" means the last thirty-odd days, which is
    /// what the legacy `AddMonths(-1)` did and what the user reads on the card.
    ///
    /// The previous window is the current one shifted back by its own length rather than another
    /// `AddMonths`. Stepping back two calendar months would compare 31 days against 30 and report
    /// a three-percent drop that is only the calendar; doc 17 §3.4 requires the two windows to
    /// span the same number of days.
    pub fn ventanas(periodo: PeriodoDashboard, ahora: DateTime<Utc>) -> Ventanas {
        let atras = |meses: u32| {
            ahora
                .checked_sub_months(Months::new(meses))
                .unwrap_or(ahora)
        };
        let con_anterior = |desde: DateTime<Utc>| Ventanas {
            desde,
            hasta: ahora,
            anterior: Some((desde - (ahora - desde), desde)),
        };

        match periodo {
            PeriodoDashboard::Mensual => con_anterior(atras(1)),
            PeriodoDashboard::Anual => con_anterior(atras(12)),
            PeriodoDashboard::Total => Ventanas {
                desde: Utc
                    .with_ymd_and_hms(COMIENZO_DE_LOS_TIEMPOS, 1, 1, 0, 0, 0)
                    .single()
                    .unwrap_or(ahora),
                hasta: ahora,
                anterior: None,
            },
        }
    }

    pub async fn stats(&self, periodo: PeriodoDashboard) -> AppResult<DashboardStats> {
        let config = self.settings.snapshot();
        let ahora = self.clock.now_utc();
        let hoy = ahora.date_naive();
        let ventanas = Self::ventanas(periodo, ahora);
        let umbral = umbral_vencimiento(hoy, config.business.factura_dias_vencimiento_default);
        let ultimos = u32::from(config.dashboard.ultimos_movimientos_cantidad.max(1));
        let top_clientes = u64::from(config.dashboard.top_clientes_cantidad.max(1));
        let top_categorias = u64::from(config.dashboard.top_categorias_cantidad.max(1));
        let ranking_obras = u64::from(config.dashboard.obras_ranking_cantidad.max(1));

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.dashboard();
            let actual = repo.resumen_rango(ventanas.desde, ventanas.hasta).await?;

            let anterior = match ventanas.anterior {
                Some((desde, hasta)) => Some(repo.resumen_rango(desde, hasta).await?),
                None => None,
            };
            let anterior_ingresos = anterior.map_or(Money::ZERO, |r| r.total_ingresos);
            let anterior_gastos = anterior.map_or(Money::ZERO, |r| r.total_gastos);
            let anterior_balance = anterior.map_or(Money::ZERO, |r| r.balance);

            let serie = repo.serie_mensual(hoy.year()).await?;
            let clientes = repo
                .top_clientes(ventanas.desde, ventanas.hasta, top_clientes)
                .await?;
            let categorias = repo
                .gastos_por_categoria(ventanas.desde, ventanas.hasta, top_categorias)
                .await?;
            let mejores = repo
                .rentabilidad_obras(SortDir::Desc, ranking_obras)
                .await?;
            let peores = repo.rentabilidad_obras(SortDir::Asc, ranking_obras).await?;
            let base = repo.estado_base().await?;

            let filtro: crate::ports::repositories::MovimientoFiltro =
                MovimientoFiltroDto::default().into();
            let movimientos = tx
                .movimientos()
                .search(
                    &filtro,
                    crate::paging::PageRequest::new(1, ultimos),
                    None,
                    SortDir::Desc,
                )
                .await?;
            let ultimos_movimientos = movimientos
                .items
                .into_iter()
                .map(MovimientoListItem::try_from)
                .collect::<AppResult<Vec<_>>>()?;

            // Comparison only makes sense against a window that exists.
            let (variacion_ingresos, variacion_gastos, variacion_balance) =
                if ventanas.anterior.is_some() {
                    (
                        DashboardStats::variacion(anterior_ingresos, actual.total_ingresos),
                        DashboardStats::variacion(anterior_gastos, actual.total_gastos),
                        DashboardStats::variacion(anterior_balance, actual.balance),
                    )
                } else {
                    (None, None, None)
                };

            Ok(DashboardStats {
                periodo,
                desde: ventanas.desde,
                hasta: ventanas.hasta,
                total_ingresos: actual.total_ingresos,
                total_gastos: actual.total_gastos,
                balance: actual.balance,
                cantidad_movimientos: actual.cantidad,
                rentabilidad: RentabilidadItem::margen(actual.total_ingresos, actual.balance),
                anterior_ingresos,
                anterior_gastos,
                variacion_ingresos,
                variacion_gastos,
                variacion_balance,
                clientes_activos: repo
                    .clientes_activos(ventanas.desde, ventanas.hasta)
                    .await?,
                trabajos_pendientes: repo.trabajos_pendientes().await?,
                obras_pausadas: repo.obras_pausadas().await?,
                facturas_vencidas: repo.facturas_vencidas(umbral).await?,
                liquidaciones_pendientes: repo
                    .liquidaciones_pendientes(hoy.year(), hoy.month())
                    .await?,
                serie_mensual: serie.into_iter().map(PuntoSerie::from).collect(),
                top_clientes: clientes.into_iter().map(TopCliente::from).collect(),
                gastos_por_categoria: categorias.into_iter().map(TopCliente::from).collect(),
                mejores_obras: mejores.into_iter().map(RentabilidadItem::from).collect(),
                peores_obras: peores.into_iter().map(RentabilidadItem::from).collect(),
                ultimos_movimientos,
                estado_sistema: EstadoSistema {
                    version: env!("CARGO_PKG_VERSION").to_owned(),
                    base_saludable: base.healthy,
                    estado: if base.healthy {
                        "Dashboard.Estado.Saludable".to_owned()
                    } else {
                        "Dashboard.Estado.Error".to_owned()
                    },
                    migraciones: base.migraciones,
                    tamano_bytes: base.tamano_bytes,
                },
            })
        }
        .await;

        finish_read(tx, outcome).await
    }

    /// The alerts of the selected period. Two of the five depend on the window, so the period
    /// travels here as well even though the cards refresh on their own schedule.
    pub async fn alertas(&self, periodo: PeriodoDashboard) -> AppResult<Vec<Alerta>> {
        let config = self.settings.snapshot();
        let ahora = self.clock.now_utc();
        let hoy = ahora.date_naive();
        let ventanas = Self::ventanas(periodo, ahora);
        let umbral = umbral_vencimiento(hoy, config.business.factura_dias_vencimiento_default);

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.dashboard();
            let vencidas = repo.facturas_vencidas(umbral).await?;
            let pausadas = repo.obras_pausadas().await?;
            let liquidaciones = repo
                .liquidaciones_pendientes(hoy.year(), hoy.month())
                .await?;
            let actual = repo.resumen_rango(ventanas.desde, ventanas.hasta).await?;
            let anterior = match ventanas.anterior {
                Some((desde, hasta)) => Some(repo.resumen_rango(desde, hasta).await?),
                None => None,
            };
            Ok((vencidas, pausadas, liquidaciones, actual, anterior))
        }
        .await;

        let (vencidas, pausadas, liquidaciones, actual, anterior) =
            finish_read(tx, outcome).await?;
        let mut alertas = Vec::new();

        if vencidas > 0 {
            alertas.push(Alerta {
                tipo: TipoAlerta::FacturasVencidas,
                clave: "Dashboard.Alerta.FacturasVencidas".to_owned(),
                cantidad: vencidas,
                monto: None,
                severidad: SeveridadAlerta::Warning,
                destino: "/facturas?estado=vencida".to_owned(),
            });
        }
        if actual.balance.is_negative() {
            alertas.push(Alerta {
                tipo: TipoAlerta::BalanceNegativo,
                clave: "Dashboard.Alerta.BalanceNegativo".to_owned(),
                cantidad: 0,
                monto: Some(actual.balance),
                severidad: SeveridadAlerta::Error,
                destino: "/movimientos".to_owned(),
            });
        }
        if pausadas > 0 {
            alertas.push(Alerta {
                tipo: TipoAlerta::ObrasPausadas,
                clave: "Dashboard.Alerta.ObrasPausadas".to_owned(),
                cantidad: pausadas,
                monto: None,
                severidad: SeveridadAlerta::Info,
                destino: "/obras?estado=pausada".to_owned(),
            });
        }
        if liquidaciones > 0 {
            alertas.push(Alerta {
                tipo: TipoAlerta::LiquidacionesPendientes,
                clave: "Dashboard.Alerta.LiquidacionesPendientes".to_owned(),
                cantidad: liquidaciones,
                monto: None,
                severidad: SeveridadAlerta::Warning,
                destino: "/liquidaciones".to_owned(),
            });
        }
        if let Some(anterior) = anterior {
            let variacion =
                DashboardStats::variacion(anterior.total_ingresos, actual.total_ingresos);
            let limite = config.dashboard.alerta_caida_ingresos_pct.abs().neg();
            if variacion.is_some_and(|v| v < limite) {
                alertas.push(Alerta {
                    tipo: TipoAlerta::CaidaIngresos,
                    clave: "Dashboard.Alerta.CaidaIngresos".to_owned(),
                    cantidad: 0,
                    monto: None,
                    severidad: SeveridadAlerta::Warning,
                    destino: "/movimientos".to_owned(),
                });
            }
        }

        Ok(alertas)
    }
}

/// Invoices issued on or before this date and still unpaid count as overdue.
fn umbral_vencimiento(hoy: NaiveDate, dias: u32) -> NaiveDate {
    hoy.checked_sub_days(chrono::Days::new(u64::from(dias)))
        .unwrap_or_else(|| {
            warn!(dias, "el umbral de vencimiento se sale del calendario");
            hoy
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use certaro_domain::Decimal4;

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
