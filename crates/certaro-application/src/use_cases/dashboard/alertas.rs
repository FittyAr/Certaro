use certaro_domain::Money;
use chrono::Datelike;
use crate::dtos::dashboard::{Alerta, DashboardStats, PeriodoDashboard, SeveridadAlerta, TipoAlerta};
use crate::ports::repositories::UnitOfWork;
use crate::ports::SettingsStore;
use crate::result::AppResult;
use crate::use_cases::shared::finish_read;
use super::ventanas::{calcular_ventanas, umbral_vencimiento};

pub async fn build_alertas(
    uow: &dyn UnitOfWork,
    settings: &dyn SettingsStore,
    ahora: chrono::DateTime<chrono::Utc>,
    periodo: PeriodoDashboard,
) -> AppResult<Vec<Alerta>> {
    let config = settings.snapshot();
    let hoy = ahora.date_naive();
    let ventanas = calcular_ventanas(periodo, ahora);
    let umbral = umbral_vencimiento(hoy, config.business.factura_dias_vencimiento_default);

    let tx = uow.begin().await?;
    let outcome = async {
        let repo = tx.dashboard();
        let vencidas = repo.facturas_vencidas(umbral).await?;
        let pausadas = repo.proyectos_pausadas().await?;
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
            tipo: TipoAlerta::ProyectosPausados,
            clave: "Dashboard.Alerta.ProyectosPausados".to_owned(),
            cantidad: pausadas,
            monto: None,
            severidad: SeveridadAlerta::Info,
            destino: "/proyectos?estado=pausada".to_owned(),
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
