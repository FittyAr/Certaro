use super::common::*;

#[tokio::test]
async fn import_base_vacia() {
    let (_legacy, dir) = create_legacy_db(true).await;
    // Don't populate: empty database.
    let target = dir.path().join("target.db");

    let report = run_import(&dir.path().join("legacy.db"), &target, true, false, false);

    assert_eq!(report["outcome"], "Success");
    // Only seed rows should exist.
    let tipos: i64 = count_rows(&target, "tipos_movimiento").await;
    assert!(tipos >= 4, "expected at least 4 system tipos, got {tipos}");
}

#[tokio::test]
async fn import_base_escalada() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    let report = run_import(&dir.path().join("legacy.db"), &target, true, false, false);

    assert_eq!(report["outcome"], "SuccessWithWarnings");
    // Verify movimientos count.
    let movs: i64 = count_rows(&target, "movimientos").await;
    assert_eq!(movs, 2);
}

#[tokio::test]
async fn import_base_sin_escalar() {
    let (legacy, dir) = create_legacy_db(false).await;
    populate_legacy(&legacy, false).await;
    let target = dir.path().join("target.db");

    let report = run_import(&dir.path().join("legacy.db"), &target, false, false, false);

    assert_eq!(report["outcome"], "SuccessWithWarnings");
    // Check that ESCALA_SIN_DECIMALES warning exists.
    let warnings = report["warnings"].as_array().unwrap();
    assert!(warnings.iter().any(|w| w["code"] == "ESCALA_SIN_DECIMALES"));
}

#[tokio::test]
async fn pago_escala_mixta() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;

    // Add an unscaled payment (raw=4500 against invoice total=121000).
    sqlx::query(
        "INSERT INTO PagosFactura (Id, FacturaId, Fecha, Monto, MedioPago, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('51000000-0000-0000-0000-000000000002', '41000000-0000-0000-0000-000000000001', '2026-03-20 00:00:00', 4500, 0, '2026-03-20 15:00:00', '2026-03-20 15:00:00', X'0000000000000001', 0)"
    ).execute(&legacy).await.unwrap();

    let target = dir.path().join("target.db");
    let report = run_import(&dir.path().join("legacy.db"), &target, true, false, false);

    let warnings = report["warnings"].as_array().unwrap();
    assert!(warnings
        .iter()
        .any(|w| w["code"] == "PAGO_ESCALA_HEURISTICA"));
}

#[tokio::test]
async fn cotizacion_cero_se_importa_null() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;

    // Add a movement with cotizacion = 0.
    sqlx::query(
        "INSERT INTO Movimientos (Id, Fecha, Concepto, Monto, Cantidad, Moneda, CotizacionAplicada, TipoMovimientoId, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('91000000-0000-0000-0000-000000000003', '2026-03-05 10:00:00', 'USD test', 100000, 10000, 1, 0, 'a0000000-0000-0000-0000-000000000001', '2026-03-05 10:00:00', '2026-03-05 10:00:00', X'0000000000000001', 0)"
    ).execute(&legacy).await.unwrap();

    let target = dir.path().join("target.db");
    let report = run_import(&dir.path().join("legacy.db"), &target, true, false, false);

    let warnings = report["warnings"].as_array().unwrap();
    assert!(warnings
        .iter()
        .any(|w| w["code"] == "COTIZACION_CERO_DESCARTADA"));
}
