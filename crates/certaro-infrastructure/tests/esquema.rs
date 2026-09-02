//! Schema tests of `docs/17-testing.md` §4.
//!
//! The one that matters most is `todo_on_delete_coincide_con_el_documento`: a `CASCADE` where a
//! `RESTRICT` belongs deletes data silently and is not noticed until something is missing.

use std::collections::BTreeMap;

use certaro_infrastructure::persistence::connection::open_in_memory;
use certaro_migration::{Migrator, MigratorTrait};
use pretty_assertions::assert_eq;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};

async fn scalars(db: &DatabaseConnection, sql: &str, column: &str) -> Vec<String> {
    db.query_all(Statement::from_string(DatabaseBackend::Sqlite, sql))
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.try_get::<String>("", column).unwrap())
        .collect()
}

async fn count(db: &DatabaseConnection, sql: &str) -> i64 {
    db.query_one(Statement::from_string(DatabaseBackend::Sqlite, sql))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap()
}

#[tokio::test]
async fn las_migraciones_crean_las_treinta_y_nueve_tablas() {
    let db = open_in_memory().await.unwrap();
    let tablas = scalars(
        &db,
        "SELECT name FROM sqlite_master WHERE type = 'table' \
         AND name NOT LIKE 'sqlite_%' AND name <> 'seaql_migrations' ORDER BY name",
        "name",
    )
    .await;

    assert_eq!(
        tablas,
        [
            "adjuntos",
            "app_metadata",
            "asistencias_empleado",
            "auth_externo",
            "calendario_evento_recursos",
            "calendario_eventos",
            "calendario_grupos_recurso",
            "calendario_recursos",
            "categorias",
            "certificado_items",
            "certificados",
            "cliente_contactos",
            "clientes",
            "empleados",
            "facturas",
            "feriados",
            "kanban_columnas",
            "kanban_etiquetas",
            "kanban_tableros",
            "kanban_tarjeta_asignados",
            "kanban_tarjeta_checklist",
            "kanban_tarjeta_etiquetas",
            "kanban_tarjetas",
            "liquidacion_adelantos",
            "liquidaciones",
            "movimientos",
            "orden_trabajo_items",
            "ordenes_trabajo",
            "pagos_factura",
            "permisos",
            "proyectos",
            "rol_permisos",
            "roles",
            "sesiones",
            "tipos_concepto_pago",
            "tipos_movimiento",
            "trabajos",
            "usuario_roles",
            "usuarios",
        ]
    );
}

#[tokio::test]
async fn la_semilla_crea_tableros_kanban_preset() {
    let db = open_in_memory().await.unwrap();
    let tableros_cnt = count(&db, "SELECT COUNT(*) as n FROM kanban_tableros WHERE es_preset = 1").await;
    assert_eq!(tableros_cnt, 2);

    let columnas_cnt = count(&db, "SELECT COUNT(*) as n FROM kanban_columnas").await;
    assert_eq!(columnas_cnt, 9);

    let etiquetas_cnt = count(&db, "SELECT COUNT(*) as n FROM kanban_etiquetas").await;
    assert_eq!(etiquetas_cnt, 4);
}

#[tokio::test]
async fn la_semilla_crea_grupos_de_recursos_iniciales() {
    let db = open_in_memory().await.unwrap();
    let grupos_cnt = count(&db, "SELECT COUNT(*) as n FROM calendario_grupos_recurso").await;
    assert_eq!(grupos_cnt, 3);
}

#[tokio::test]
async fn la_semilla_crea_super_admin_y_roles_de_sistema() {
    let db = open_in_memory().await.unwrap();
    let super_admin = scalars(
        &db,
        "SELECT email FROM usuarios WHERE id = '00000000-0000-0000-0000-000000000999'",
        "email",
    )
    .await;
    assert_eq!(super_admin, ["admin@certaro.local"]);

    let roles_cnt = count(&db, "SELECT COUNT(*) as n FROM roles WHERE es_sistema = 1").await;
    assert_eq!(roles_cnt, 3);

    let permisos_cnt = count(&db, "SELECT COUNT(*) as n FROM permisos").await;
    assert_eq!(permisos_cnt, 39);
}

#[tokio::test]
async fn las_migraciones_son_reversibles() {
    let db = open_in_memory().await.unwrap();
    Migrator::down(&db, None).await.unwrap();

    let tablas = scalars(
        &db,
        "SELECT name FROM sqlite_master WHERE type = 'table' \
         AND name NOT LIKE 'sqlite_%' AND name <> 'seaql_migrations'",
        "name",
    )
    .await;
    assert!(tablas.is_empty(), "quedaron tablas: {tablas:?}");
}

#[tokio::test]
async fn las_foreign_keys_estan_activas() {
    let db = open_in_memory().await.unwrap();
    let activas = db
        .query_one(Statement::from_string(
            DatabaseBackend::Sqlite,
            "PRAGMA foreign_keys",
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i32>("", "foreign_keys")
        .unwrap();
    assert_eq!(activas, 1);
}

#[tokio::test]
async fn todo_on_delete_coincide_con_el_documento() {
    let db = open_in_memory().await.unwrap();

    // (tabla, columna) -> acción, de `docs/03-modelo-de-datos.md` §4.
    let esperado: BTreeMap<(&str, &str), &str> = BTreeMap::from([
        (("categorias", "categoria_padre_id"), "RESTRICT"),
        (("cliente_contactos", "cliente_id"), "CASCADE"),
        (("proyectos", "cliente_id"), "RESTRICT"),
        (("trabajos", "proyecto_id"), "RESTRICT"),
        (("ordenes_trabajo", "trabajo_id"), "CASCADE"),
        (("orden_trabajo_items", "orden_trabajo_id"), "CASCADE"),
        (("certificados", "orden_trabajo_id"), "CASCADE"),
        (("certificado_items", "certificado_id"), "CASCADE"),
        (("certificado_items", "orden_trabajo_item_id"), "RESTRICT"),
        (("facturas", "cliente_id"), "RESTRICT"),
        (("pagos_factura", "factura_id"), "CASCADE"),
        (("asistencias_empleado", "empleado_id"), "CASCADE"),
        (("asistencias_empleado", "trabajo_id"), "SET NULL"),
        (("liquidaciones", "empleado_id"), "CASCADE"),
        (("liquidacion_adelantos", "liquidacion_id"), "CASCADE"),
        (("liquidacion_adelantos", "movimiento_id"), "RESTRICT"),
        (("movimientos", "tipo_movimiento_id"), "RESTRICT"),
        (("movimientos", "categoria_id"), "RESTRICT"),
        (("movimientos", "tipo_concepto_pago_id"), "SET NULL"),
        (("movimientos", "factura_id"), "SET NULL"),
        (("movimientos", "cliente_id"), "SET NULL"),
        (("movimientos", "trabajo_id"), "SET NULL"),
        (("movimientos", "empleado_id"), "SET NULL"),
    ]);

    let tablas: Vec<String> = esperado.keys().map(|(t, _)| (*t).to_owned()).collect();
    let mut real: BTreeMap<(String, String), String> = BTreeMap::new();

    for tabla in tablas {
        let filas = db
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!("PRAGMA foreign_key_list({tabla})"),
            ))
            .await
            .unwrap();
        for fila in filas {
            let columna: String = fila.try_get("", "from").unwrap();
            let accion: String = fila.try_get("", "on_delete").unwrap();
            real.insert((tabla.clone(), columna), accion);
        }
    }

    let esperado: BTreeMap<(String, String), String> = esperado
        .into_iter()
        .map(|((t, c), a)| ((t.to_owned(), c.to_owned()), a.to_owned()))
        .collect();

    assert_eq!(real, esperado);
}

#[tokio::test]
async fn la_semilla_inserta_los_tipos_de_sistema() {
    let db = open_in_memory().await.unwrap();

    let ingresos = scalars(
        &db,
        "SELECT id || ':' || nombre || ':' || es_ingreso AS fila \
         FROM tipos_movimiento WHERE es_sistema = 1 ORDER BY id",
        "fila",
    )
    .await;

    assert_eq!(
        ingresos,
        [
            "00000000-0000-0000-0000-000000000001:Ingreso:1",
            "00000000-0000-0000-0000-000000000002:Gasto:0",
            "00000000-0000-0000-0000-000000000003:Adelanto:0",
            // Deliberate: an adjustment carries its own sign in the amount and adds to the balance.
            "00000000-0000-0000-0000-000000000004:Ajuste:1",
        ]
    );
    assert_eq!(
        count(&db, "SELECT COUNT(*) AS n FROM tipos_concepto_pago").await,
        4
    );
}

#[tokio::test]
async fn la_semilla_es_idempotente() {
    let db = open_in_memory().await.unwrap();
    Migrator::up(&db, None).await.unwrap();

    assert_eq!(
        count(&db, "SELECT COUNT(*) AS n FROM tipos_movimiento").await,
        4
    );
}

#[tokio::test]
async fn no_se_puede_borrar_un_tipo_de_movimiento_en_uso() {
    let db = open_in_memory().await.unwrap();
    db.execute_unprepared(
        "INSERT INTO categorias (id, nombre, created_at) \
             VALUES ('c1','Materiales','2026-01-01T00:00:00.000Z');
         INSERT INTO movimientos (id, fecha, concepto, monto, tipo_movimiento_id, categoria_id, created_at) \
             VALUES ('m1','2026-01-01T00:00:00.000Z','Cable',10000, \
                     '00000000-0000-0000-0000-000000000002','c1','2026-01-01T00:00:00.000Z');",
    )
    .await
    .unwrap();

    let error = db
        .execute_unprepared(
            "DELETE FROM tipos_movimiento WHERE id = '00000000-0000-0000-0000-000000000002'",
        )
        .await
        .unwrap_err();

    assert!(
        error.to_string().to_lowercase().contains("foreign key"),
        "se esperaba un rechazo por clave foránea, llegó: {error}"
    );
}

#[tokio::test]
async fn un_numero_de_proyecto_borrado_sigue_reservado() {
    let db = open_in_memory().await.unwrap();
    db.execute_unprepared(
        "INSERT INTO clientes (id, nombre, created_at) VALUES ('cl1','Acme','2026-01-01T00:00:00.000Z');
         INSERT INTO proyectos (id, numero, nombre, cliente_id, created_at, is_deleted) \
             VALUES ('o1', 7, 'Proyecto vieja', 'cl1', '2026-01-01T00:00:00.000Z', 1);",
    )
    .await
    .unwrap();

    let error = db
        .execute_unprepared(
            "INSERT INTO proyectos (id, numero, nombre, cliente_id, created_at) \
             VALUES ('o2', 7, 'Proyecto nueva', 'cl1', '2026-01-01T00:00:00.000Z')",
        )
        .await
        .unwrap_err();

    assert!(error.to_string().to_lowercase().contains("unique"));
}

#[tokio::test]
async fn el_nombre_de_un_tipo_borrado_se_puede_reutilizar() {
    let db = open_in_memory().await.unwrap();
    db.execute_unprepared(
        "INSERT INTO tipos_movimiento (id, nombre, created_at, is_deleted) \
             VALUES ('t1','Viáticos','2026-01-01T00:00:00.000Z',1);
         INSERT INTO tipos_movimiento (id, nombre, created_at) \
             VALUES ('t2','Viáticos','2026-01-01T00:00:00.000Z');",
    )
    .await
    .unwrap();

    assert_eq!(
        count(
            &db,
            "SELECT COUNT(*) AS n FROM tipos_movimiento WHERE nombre = 'Viáticos'"
        )
        .await,
        2
    );
}
