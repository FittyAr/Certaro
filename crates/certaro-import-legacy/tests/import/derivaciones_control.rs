use super::common::*;
use sqlx::sqlite::SqliteConnectOptions;

#[tokio::test]
async fn factura_con_pago_parcial_queda_pagada_parcial() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    let _report = run_import(&dir.path().join("legacy.db"), &target, true, false, false);

    // The invoice had estado=1 (Emitida), total=121000, payment=50000.
    // After reclassification: PagadaParcial (5).
    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let estado: i64 = sqlx::query_scalar(
        "SELECT estado FROM facturas WHERE id = '41000000-0000-0000-0000-000000000001'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(estado, 5, "expected PagadaParcial (5), got {estado}");
}

#[tokio::test]
async fn email_cliente_se_vuelve_contacto_principal() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    run_import(&dir.path().join("legacy.db"), &target, true, false, false);

    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    // The client had email='acme@example.com'. A principal contact should exist.
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM cliente_contactos WHERE cliente_id = 'd0000000-0000-0000-0000-000000000001' AND es_principal = 1"
    ).fetch_one(&pool).await.unwrap();
    assert!(
        count >= 1,
        "expected at least 1 principal contact, got {count}"
    );
}

#[tokio::test]
async fn dry_run_no_escribe() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    let report = run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        true, // dry_run
        false,
    );

    // The report should say dry_run.
    assert_eq!(report["dryRun"], true);
    // The target should not exist or be empty.
    if target.exists() {
        let movs = count_rows(&target, "movimientos").await;
        assert_eq!(movs, 0, "dry run should not write data");
    }
}

#[tokio::test]
async fn reejecucion_sobre_destino_con_datos_aborta() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    // First import.
    run_import(&dir.path().join("legacy.db"), &target, true, false, false);

    // Second import should fail because target has data.
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_certaro-import-legacy"));
    cmd.arg("--source").arg(dir.path().join("legacy.db"));
    cmd.arg("--target").arg(&target);
    cmd.arg("--assume-scaled");
    cmd.arg("--report")
        .arg(target.parent().unwrap().join("import_report2.json"));

    let output = cmd.output().unwrap();
    assert!(!output.status.success(), "second import should fail");
}

#[tokio::test]
async fn tipo_de_sistema_no_se_duplica() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    run_import(&dir.path().join("legacy.db"), &target, true, false, false);

    // System tipos should not be duplicated.
    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tipos_movimiento WHERE id = '00000000-0000-0000-0000-000000000001'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1, "system tipo should exist exactly once");
}

#[tokio::test]
async fn cuit_se_normaliza() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    run_import(&dir.path().join("legacy.db"), &target, true, false, false);

    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let cuit: String = sqlx::query_scalar(
        "SELECT cuit FROM clientes WHERE id = 'd0000000-0000-0000-0000-000000000001'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(cuit, "20123456789", "CUIT should be normalized");
}

#[tokio::test]
async fn color_hex_invalido_se_marca() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;

    // Add a category with invalid color.
    sqlx::query(
        "INSERT INTO Categorias (Id, Nombre, Descripcion, ColorHex, Icono, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('c0000000-0000-0000-0000-000000000002', 'Invalida', NULL, 'rojo', NULL, '2026-01-10 08:00:00', '2026-01-10 08:00:00', X'0000000000000001', 0)"
    ).execute(&legacy).await.unwrap();

    let target = dir.path().join("target.db");
    let report = run_import(&dir.path().join("legacy.db"), &target, true, false, false);

    let warnings = report["warnings"].as_array().unwrap();
    assert!(warnings.iter().any(|w| w["code"] == "COLOR_HEX_INVALIDO"));
}

#[tokio::test]
async fn reporte_tiene_estructura_correcta() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    let report = run_import(&dir.path().join("legacy.db"), &target, true, false, false);

    // Verify report structure.
    assert!(report["toolVersion"].is_string());
    assert!(report["startedAt"].is_string());
    assert!(report["finishedAt"].is_string());
    assert!(report["source"].is_object());
    assert!(report["target"].is_object());
    assert!(report["tables"].is_array());
    assert!(report["derived"].is_object());
    assert!(report["warnings"].is_array());
    assert!(report["blockingIssues"].is_array());
    assert!(report["attachments"].is_object());
}
