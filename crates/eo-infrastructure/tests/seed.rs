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
    assert_eq!(resumen.clientes, 4);
    assert_eq!(resumen.obras, 4);
    assert_eq!(resumen.trabajos, 5);
    assert_eq!(resumen.ordenes_trabajo, 3);
    assert_eq!(resumen.movimientos, 9);
    assert_eq!(resumen.facturas, 3);
    assert_eq!(resumen.liquidaciones, 2);

    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM categorias").await, 8);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM empleados").await, 5);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM clientes").await, 4);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM obras").await, 4);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM trabajos").await, 5);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM ordenes_trabajo").await, 3);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM movimientos").await, 9);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM facturas").await, 3);
    assert_eq!(count(&db, "SELECT COUNT(*) AS n FROM liquidaciones").await, 2);
}
