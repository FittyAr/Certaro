use pretty_assertions::assert_eq;
use super::common::*;
use certaro_application::dtos::dashboard::*;
use certaro_domain::{Decimal4, Money};

#[tokio::test]
async fn los_kpis_agregan_solo_la_ventana_del_periodo() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;

    // Inside the monthly window.
    f.movimiento(
        "Cobro",
        "1000.0000",
        "1.0000",
        true,
        5,
        Some(categoria),
        None,
        None,
    )
    .await;
    f.movimiento(
        "Compra",
        "300.0000",
        "1.0000",
        false,
        5,
        Some(categoria),
        None,
        None,
    )
    .await;
    // Inside the previous window, not the current one.
    f.movimiento(
        "Cobro viejo",
        "500.0000",
        "1.0000",
        true,
        45,
        Some(categoria),
        None,
        None,
    )
    .await;

    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();

    assert_eq!(stats.total_ingresos.to_decimal_string(), "1000.0000");
    assert_eq!(stats.total_gastos.to_decimal_string(), "300.0000");
    assert_eq!(stats.balance.to_decimal_string(), "700.0000");
    assert_eq!(stats.cantidad_movimientos, 2);
    // The older income is the basis of the comparison, not part of the total.
    assert_eq!(stats.anterior_ingresos.to_decimal_string(), "500.0000");
    assert_eq!(
        stats.variacion_ingresos,
        Some(Decimal4::parse("100.0").unwrap())
    );
}

#[tokio::test]
async fn el_periodo_total_no_publica_comparacion() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    f.movimiento(
        "Cobro",
        "1000.0000",
        "1.0000",
        true,
        400,
        Some(categoria),
        None,
        None,
    )
    .await;

    let stats = f.dashboard.stats(PeriodoDashboard::Total).await.unwrap();

    // A movement more than a year old is still counted by `Total`.
    assert_eq!(stats.total_ingresos.to_decimal_string(), "1000.0000");
    assert_eq!(stats.variacion_ingresos, None);
    assert_eq!(stats.variacion_gastos, None);
}

#[tokio::test]
async fn el_total_del_periodo_usa_el_producto_y_no_el_monto_unitario() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    f.movimiento(
        "Cable",
        "1500.5000",
        "2.0000",
        false,
        3,
        Some(categoria),
        None,
        None,
    )
    .await;

    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();
    assert_eq!(stats.total_gastos.to_decimal_string(), "3001.0000");
}

#[tokio::test]
async fn la_serie_mensual_trae_los_doce_meses() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    f.movimiento(
        "Cobro",
        "800.0000",
        "1.0000",
        true,
        3,
        Some(categoria),
        None,
        None,
    )
    .await;

    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();

    assert_eq!(stats.serie_mensual.len(), 12);
    assert_eq!(stats.serie_mensual[0].mes, 1);
    let agosto = &stats.serie_mensual[7];
    assert_eq!(agosto.ingresos.to_decimal_string(), "800.0000");
    // An empty month is zero, not a gap the chart has to invent.
    assert_eq!(stats.serie_mensual[0].ingresos, Money::ZERO);
}

#[tokio::test]
async fn el_top_de_clientes_agrupa_por_id_y_solo_cuenta_ingresos() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    let uno = f.cliente("Cliente Uno").await;
    let dos = f.cliente("Cliente Dos").await;

    f.movimiento(
        "A",
        "1000.0000",
        "1.0000",
        true,
        3,
        Some(categoria),
        Some(uno),
        None,
    )
    .await;
    f.movimiento(
        "B",
        "400.0000",
        "1.0000",
        true,
        4,
        Some(categoria),
        Some(dos),
        None,
    )
    .await;
    // An expense charged to the customer must not inflate their billing.
    f.movimiento(
        "C",
        "9000.0000",
        "1.0000",
        false,
        4,
        Some(categoria),
        Some(dos),
        None,
    )
    .await;

    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();

    assert_eq!(stats.top_clientes.len(), 2);
    assert_eq!(stats.top_clientes[0].id, Some(uno));
    assert_eq!(stats.top_clientes[0].total.to_decimal_string(), "1000.0000");
    assert_eq!(stats.top_clientes[1].total.to_decimal_string(), "400.0000");
    assert_eq!(stats.clientes_activos, 2);
}

#[tokio::test]
async fn los_gastos_por_categoria_ordenan_de_mayor_a_menor() {
    let f = fixture().await;
    let materiales = f.categoria("Materiales").await;
    let combustible = f.categoria("Combustible").await;

    f.movimiento(
        "A",
        "100.0000",
        "1.0000",
        false,
        2,
        Some(materiales),
        None,
        None,
    )
    .await;
    f.movimiento(
        "B",
        "700.0000",
        "1.0000",
        false,
        2,
        Some(combustible),
        None,
        None,
    )
    .await;
    f.movimiento(
        "C",
        "999.0000",
        "1.0000",
        true,
        2,
        Some(materiales),
        None,
        None,
    )
    .await;

    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();

    assert_eq!(stats.gastos_por_categoria.len(), 2);
    assert_eq!(stats.gastos_por_categoria[0].nombre, "Combustible");
    assert_eq!(
        stats.gastos_por_categoria[0].total.to_decimal_string(),
        "700.0000"
    );
}
