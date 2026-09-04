use pretty_assertions::assert_eq;
use super::common::*;
use certaro_application::dtos::dashboard::*;

#[tokio::test]
async fn las_alertas_llevan_su_destino_con_el_filtro_aplicado() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    let cliente = f.cliente("Cliente").await;
    f.factura("0001", cliente, "1000.0000", 60).await;
    // A negative balance raises the error-level alert.
    f.movimiento(
        "Gasto",
        "5000.0000",
        "1.0000",
        false,
        3,
        Some(categoria),
        None,
        None,
    )
    .await;

    let alertas = f
        .dashboard
        .alertas(PeriodoDashboard::Mensual)
        .await
        .unwrap();

    let vencidas = alertas
        .iter()
        .find(|a| a.clave == "Dashboard.Alerta.FacturasVencidas")
        .unwrap();
    assert_eq!(vencidas.cantidad, 1);
    assert_eq!(vencidas.destino, "/facturas?estado=vencida");

    let balance = alertas
        .iter()
        .find(|a| a.clave == "Dashboard.Alerta.BalanceNegativo")
        .unwrap();
    assert_eq!(
        balance.monto.map(|m| m.to_decimal_string()),
        Some("-5000.0000".to_owned())
    );
}

#[tokio::test]
async fn el_estado_del_sistema_informa_la_base() {
    let f = fixture().await;
    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();

    assert!(stats.estado_sistema.base_saludable);
    assert_eq!(stats.estado_sistema.estado, "Dashboard.Estado.Saludable");
    assert!(stats.estado_sistema.migraciones > 0);
    assert!(stats.estado_sistema.tamano_bytes > 0);
}

#[tokio::test]
async fn los_ultimos_movimientos_vienen_del_mas_nuevo_al_mas_viejo() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    f.movimiento(
        "Viejo",
        "100.0000",
        "1.0000",
        false,
        20,
        Some(categoria),
        None,
        None,
    )
    .await;
    f.movimiento(
        "Nuevo",
        "200.0000",
        "1.0000",
        false,
        1,
        Some(categoria),
        None,
        None,
    )
    .await;

    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();

    assert_eq!(stats.ultimos_movimientos.len(), 2);
    assert_eq!(stats.ultimos_movimientos[0].concepto, "Nuevo");
}
