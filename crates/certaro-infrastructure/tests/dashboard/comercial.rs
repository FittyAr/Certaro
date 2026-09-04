use pretty_assertions::assert_eq;
use super::common::*;
use certaro_application::dtos::comercial::*;
use certaro_application::dtos::dashboard::*;
use certaro_domain::Money;
use uuid::Uuid;

#[tokio::test]
async fn la_cuenta_corriente_deriva_el_saldo_y_la_mora() {
    let f = fixture().await;
    let cliente = f.cliente("Deudor").await;
    let factura = f.factura_vencida("0001", cliente, "1000.0000", 45).await;
    f.pagar(factura, "400.0000").await;

    let cuenta = f
        .comercial
        .cuenta_corriente(CuentaCorrienteQuery {
            cliente_id: cliente,
            incluir_pagadas: false,
        })
        .await
        .unwrap();

    assert_eq!(cuenta.cliente_nombre, "Deudor");
    assert_eq!(cuenta.facturas.len(), 1);
    assert_eq!(cuenta.saldo.to_decimal_string(), "600.0000");
    assert_eq!(cuenta.total_facturado.to_decimal_string(), "1000.0000");
    assert_eq!(cuenta.total_pagado.to_decimal_string(), "400.0000");
    assert_eq!(cuenta.facturas[0].dias_mora, 45);
}

#[tokio::test]
async fn una_factura_saldada_sale_de_la_cuenta_corriente() {
    let f = fixture().await;
    let cliente = f.cliente("Al día").await;
    let factura = f.factura("0002", cliente, "500.0000", 10).await;
    f.pagar(factura, "500.0000").await;

    let cuenta = f
        .comercial
        .cuenta_corriente(CuentaCorrienteQuery {
            cliente_id: cliente,
            incluir_pagadas: false,
        })
        .await
        .unwrap();
    assert!(cuenta.facturas.is_empty());
    assert_eq!(cuenta.saldo, Money::ZERO);

    // Asked for explicitly, it comes back with no arrears.
    let con_pagadas = f
        .comercial
        .cuenta_corriente(CuentaCorrienteQuery {
            cliente_id: cliente,
            incluir_pagadas: true,
        })
        .await
        .unwrap();
    assert_eq!(con_pagadas.facturas.len(), 1);
    assert_eq!(con_pagadas.facturas[0].dias_mora, 0);
}

#[tokio::test]
async fn un_cliente_inexistente_da_una_cuenta_vacia_y_no_un_error() {
    let f = fixture().await;

    let cuenta = f
        .comercial
        .cuenta_corriente(CuentaCorrienteQuery {
            cliente_id: Uuid::nil(),
            incluir_pagadas: false,
        })
        .await
        .unwrap();

    assert_eq!(cuenta.cliente_id, Uuid::nil());
    assert_eq!(cuenta.cliente_nombre, "");
    assert!(cuenta.facturas.is_empty());
}

#[tokio::test]
async fn los_bordes_de_los_buckets_caen_en_la_columna_documentada() {
    let f = fixture().await;
    let cliente = f.cliente("Deudor").await;

    // One invoice per boundary, each of 100, so the column it lands in is unmistakable.
    for (i, dias) in [30_i64, 31, 60, 61, 90, 91].into_iter().enumerate() {
        f.factura_vencida(&format!("B{i}"), cliente, "100.0000", dias)
            .await;
    }

    let aging = f
        .comercial
        .antiguedad_deuda(AntiguedadDeudaQuery {
            fecha_corte: None,
            cliente_id: None,
        })
        .await
        .unwrap();

    assert_eq!(aging.bucket0a30.to_decimal_string(), "100.0000");
    assert_eq!(aging.bucket31a60.to_decimal_string(), "200.0000");
    assert_eq!(aging.bucket61a90.to_decimal_string(), "200.0000");
    assert_eq!(aging.bucket_mas90.to_decimal_string(), "100.0000");
    assert_eq!(aging.total.to_decimal_string(), "600.0000");

    // The invariant the report lives by.
    let suma = Money::try_sum([
        aging.bucket0a30,
        aging.bucket31a60,
        aging.bucket61a90,
        aging.bucket_mas90,
    ])
    .unwrap();
    assert_eq!(suma, aging.total);
    assert_eq!(aging.limites, vec![30, 60, 90]);
}

#[tokio::test]
async fn la_antiguedad_desglosa_por_cliente_y_respeta_la_fecha_de_corte() {
    let f = fixture().await;
    let uno = f.cliente("Uno").await;
    let dos = f.cliente("Dos").await;
    f.factura_vencida("0001", uno, "1000.0000", 20).await;
    f.factura_vencida("0002", dos, "300.0000", 20).await;

    let aging = f
        .comercial
        .antiguedad_deuda(AntiguedadDeudaQuery {
            fecha_corte: None,
            cliente_id: None,
        })
        .await
        .unwrap();
    assert_eq!(aging.detalle.len(), 2);
    // Sorted by how much each one owes.
    assert_eq!(aging.detalle[0].cliente_id, uno);
    assert_eq!(aging.detalle[0].bucket0a30.to_decimal_string(), "1000.0000");

    // Moving the cut-off date forward ages the debt into the next bucket.
    let corrido = f
        .comercial
        .antiguedad_deuda(AntiguedadDeudaQuery {
            fecha_corte: Some(hoy() + chrono::Duration::days(15)),
            cliente_id: Some(uno),
        })
        .await
        .unwrap();
    assert_eq!(corrido.bucket0a30, Money::ZERO);
    assert_eq!(corrido.bucket31a60.to_decimal_string(), "1000.0000");
    assert_eq!(corrido.detalle.len(), 1);
}

#[tokio::test]
async fn las_facturas_vencidas_exigen_saldo_pendiente() {
    let f = fixture().await;
    let cliente = f.cliente("Cliente").await;
    // Older than the thirty-day threshold, so it counts.
    let vieja = f.factura("0001", cliente, "1000.0000", 60).await;
    // Recent: not overdue yet.
    f.factura("0002", cliente, "500.0000", 5).await;

    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();
    assert_eq!(stats.facturas_vencidas, 1);

    // Once collected it stops being a debt even though its date did not change.
    f.pagar(vieja, "1000.0000").await;
    let stats = f.dashboard.stats(PeriodoDashboard::Mensual).await.unwrap();
    assert_eq!(stats.facturas_vencidas, 0);
}
