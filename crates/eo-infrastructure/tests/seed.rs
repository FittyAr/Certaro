use eo_infrastructure::persistence::{open_in_memory, seed_demo_data};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};

async fn count(db: &DatabaseConnection, sql: &str) -> i64 {
    db.query_one(Statement::from_string(DatabaseBackend::Sqlite, sql))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap()
}

#[tokio::test]
async fn el_sembrado_de_datos_de_prueba_puebla_las_tablas_correctamente() {
    let db = open_in_memory().await.unwrap();

    let resumen = seed_demo_data(&db).await.unwrap();

    assert_eq!(resumen.categorias, 8);
    assert_eq!(resumen.tipos_movimiento, 3);
    assert_eq!(resumen.empleados, 5);
    assert_eq!(resumen.asistencias, 30);
    assert_eq!(resumen.clientes, 4);
    assert_eq!(resumen.contactos, 4);
    assert_eq!(resumen.obras, 4);
    assert_eq!(resumen.trabajos, 5);
    assert_eq!(resumen.ordenes_trabajo, 3);
    assert_eq!(resumen.orden_trabajo_items, 6);
    assert_eq!(resumen.certificados, 2);
    assert_eq!(resumen.certificado_items, 2);
    assert_eq!(resumen.facturas, 3);
    assert_eq!(resumen.pagos_factura, 1);
    assert_eq!(resumen.movimientos, 9);
    assert_eq!(resumen.liquidaciones, 2);
    assert_eq!(resumen.liquidacion_adelantos, 1);
    assert_eq!(resumen.feriados, 9);
    assert_eq!(resumen.adjuntos, 3);

    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM categorias").await, 8);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM empleados").await, 5);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM asistencias_empleado").await, 30);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM clientes").await, 4);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM cliente_contactos").await, 4);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM obras").await, 4);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM trabajos").await, 5);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM ordenes_trabajo").await, 3);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM certificados").await, 2);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM facturas").await, 3);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM pagos_factura").await, 1);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM movimientos").await, 9);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM liquidaciones").await, 2);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM liquidacion_adelantos").await, 1);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM adjuntos").await, 3);
}
