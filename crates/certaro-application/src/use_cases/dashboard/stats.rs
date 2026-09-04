use certaro_domain::Money;
use chrono::Datelike;

use crate::dtos::dashboard::{
    DashboardStats, EstadoSistema, PeriodoDashboard, PuntoSerie, RentabilidadItem, TopCliente,
};
use crate::dtos::movimientos::{MovimientoFiltroDto, MovimientoListItem};
use crate::ports::repositories::{SortDir, UnitOfWork};
use crate::ports::SettingsStore;
use crate::result::AppResult;
use crate::use_cases::shared::finish_read;
use super::ventanas::{calcular_ventanas, umbral_vencimiento};

pub async fn build_stats(
    uow: &dyn UnitOfWork,
    settings: &dyn SettingsStore,
    ahora: chrono::DateTime<chrono::Utc>,
    periodo: PeriodoDashboard,
) -> AppResult<DashboardStats> {
    let config = settings.snapshot();
    let hoy = ahora.date_naive();
    let ventanas = calcular_ventanas(periodo, ahora);
    let umbral = umbral_vencimiento(hoy, config.business.factura_dias_vencimiento_default);
    let ultimos = u32::from(config.dashboard.ultimos_movimientos_cantidad.max(1));
    let top_clientes = u64::from(config.dashboard.top_clientes_cantidad.max(1));
    let top_categorias = u64::from(config.dashboard.top_categorias_cantidad.max(1));
    let ranking_proyectos = u64::from(config.dashboard.proyectos_ranking_cantidad.max(1));

    let tx = uow.begin().await?;
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
            .rentabilidad_proyectos(SortDir::Desc, ranking_proyectos)
            .await?;
        let peores = repo.rentabilidad_proyectos(SortDir::Asc, ranking_proyectos).await?;
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
            proyectos_pausadas: repo.proyectos_pausadas().await?,
            facturas_vencidas: repo.facturas_vencidas(umbral).await?,
            liquidaciones_pendientes: repo
                .liquidaciones_pendientes(hoy.year(), hoy.month())
                .await?,
            serie_mensual: serie.into_iter().map(PuntoSerie::from).collect(),
            top_clientes: clientes.into_iter().map(TopCliente::from).collect(),
            gastos_por_categoria: categorias.into_iter().map(TopCliente::from).collect(),
            mejores_proyectos: mejores.into_iter().map(RentabilidadItem::from).collect(),
            peores_proyectos: peores.into_iter().map(RentabilidadItem::from).collect(),
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
