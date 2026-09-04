use super::common::*;
use sqlx::sqlite::SqliteConnectOptions;

#[tokio::test]
async fn fecha_civil_no_cambia_de_dia() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    run_import(&dir.path().join("legacy.db"), &target, true, false, false);

    // The asistencia was at 22:30 local. It should be midnight UTC of the same civil day.
    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let fecha: String = sqlx::query_scalar(
        "SELECT fecha FROM asistencias_empleado WHERE id = '71000000-0000-0000-0000-000000000001'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        fecha.starts_with("2026-03-01"),
        "expected 2026-03-01, got {fecha}"
    );
    assert!(fecha.contains("00:00:00"), "expected midnight, got {fecha}");
}

#[tokio::test]
async fn fecha_negocio_con_hora_se_convierte() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    run_import(&dir.path().join("legacy.db"), &target, true, false, false);

    // The movimiento was at 10:00 local (UTC-3) = 13:00 UTC.
    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let fecha: String = sqlx::query_scalar(
        "SELECT fecha FROM movimientos WHERE id = '91000000-0000-0000-0000-000000000001'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(fecha.contains("13:00"), "expected 13:00 UTC, got {fecha}");
}

#[tokio::test]
async fn auditoria_no_se_desplaza() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    run_import(&dir.path().join("legacy.db"), &target, true, false, false);

    // CreatedAt should be preserved as-is (it was already UTC).
    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let created: String = sqlx::query_scalar(
        "SELECT created_at FROM movimientos WHERE id = '91000000-0000-0000-0000-000000000001'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        created.contains("10:00"),
        "expected 10:00 UTC, got {created}"
    );
}

#[tokio::test]
async fn cantidad_cero_se_vuelve_uno() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;

    // Add a movement with cantidad = 0.
    sqlx::query(
        "INSERT INTO Movimientos (Id, Fecha, Concepto, Monto, Cantidad, Moneda, TipoMovimientoId, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('91000000-0000-0000-0000-000000000004', '2026-03-06 10:00:00', 'Test cantidad 0', 50000, 0, 0, 'a0000000-0000-0000-0000-000000000001', '2026-03-06 10:00:00', '2026-03-06 10:00:00', X'0000000000000001', 0)"
    ).execute(&legacy).await.unwrap();

    let target = dir.path().join("target.db");
    run_import(&dir.path().join("legacy.db"), &target, true, false, false);

    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let cantidad: i64 = sqlx::query_scalar(
        "SELECT cantidad FROM movimientos WHERE id = '91000000-0000-0000-0000-000000000004'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(cantidad, 10_000, "expected 10000 (=1.0), got {cantidad}");
}

#[tokio::test]
async fn multiplicador_cero_se_vuelve_uno() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;

    // Add a liquidation with zero multipliers.
    sqlx::query(
        "INSERT INTO Liquidaciones (Id, EmpleadoId, FechaInicio, FechaFin, DiasTrabajados, TarifaAplicada, IncluirSabados, IncluirDomingos, IncluirFeriados, MultiplicadorSabado, MultiplicadorDomingo, MultiplicadorFeriado, TotalBruto, TotalAdelantos, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('81000000-0000-0000-0000-000000000002', '61000000-0000-0000-0000-000000000001', '2026-04-01 00:00:00', '2026-04-15 00:00:00', 100000, 250000, 1, 0, 0, 0, 0, 0, 2500000, 0, '2026-04-16 10:00:00', '2026-04-16 10:00:00', X'0000000000000001', 0)"
    ).execute(&legacy).await.unwrap();

    let target = dir.path().join("target.db");
    run_import(&dir.path().join("legacy.db"), &target, true, false, false);

    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let sab: i64 = sqlx::query_scalar("SELECT multiplicador_sabado FROM liquidaciones WHERE id = '81000000-0000-0000-0000-000000000002'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(sab, 10_000, "expected 10000 (=1.0), got {sab}");
}
