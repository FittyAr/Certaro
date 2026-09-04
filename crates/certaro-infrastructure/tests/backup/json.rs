use certaro_application::ports::BackupPort;
use certaro_infrastructure::backup::SqliteBackupService;
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

use super::common::{assert_clave, cuantos_tipos, entorno, sembrar};

#[tokio::test]
async fn export_import_ida_y_vuelta() {
    let entorno = entorno().await;
    sembrar(&entorno, "Sólo en el origen").await;
    let esperados = cuantos_tipos(&entorno).await;

    let destino = entorno.paths.root().join("volcado.json");
    let resumen = entorno.service.export_json(&destino).await.unwrap();
    assert!(resumen.filas > 0);
    assert!(destino.is_file());

    // Emptied on purpose: the import has to rebuild it, not merge into it.
    {
        let db = entorno.db.read().await;
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "DELETE FROM tipos_movimiento".to_owned(),
        ))
        .await
        .unwrap();
    }
    assert_eq!(cuantos_tipos(&entorno).await, 0);

    let vuelta = entorno.service.import_json(&destino).await.unwrap();

    assert_eq!(cuantos_tipos(&entorno).await, esperados);
    assert_eq!(vuelta.filas, resumen.filas);
}

#[tokio::test]
async fn import_json_rechaza_tabla_desconocida() {
    let entorno = entorno().await;
    let destino = entorno.paths.root().join("volcado.json");
    entorno.service.export_json(&destino).await.unwrap();

    let mut documento: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&destino).unwrap()).unwrap();
    documento["tables"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "name": "sqlite_master",
            "columns": ["name"],
            "rows": [["tipos_movimiento"]]
        }));
    std::fs::write(&destino, serde_json::to_vec(&documento).unwrap()).unwrap();

    let error = entorno.service.import_json(&destino).await.unwrap_err();
    assert_clave(&error, "Validation.Import.TablaDesconocida");
}

#[tokio::test]
async fn import_json_rechaza_columna_desconocida() {
    let entorno = entorno().await;
    let destino = entorno.paths.root().join("volcado.json");
    entorno.service.export_json(&destino).await.unwrap();

    let mut documento: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&destino).unwrap()).unwrap();
    for tabla in documento["tables"].as_array_mut().unwrap() {
        if tabla["name"] == "tipos_movimiento" {
            tabla["columns"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!("columna_inventada"));
        }
    }
    std::fs::write(&destino, serde_json::to_vec(&documento).unwrap()).unwrap();

    let error = entorno.service.import_json(&destino).await.unwrap_err();
    assert_clave(&error, "Validation.Import.ColumnaDesconocida");
}

#[tokio::test]
async fn import_json_es_atomico() {
    let entorno = entorno().await;
    sembrar(&entorno, "Tiene que sobrevivir").await;
    let esperados = cuantos_tipos(&entorno).await;

    let destino = entorno.paths.root().join("volcado.json");
    entorno.service.export_json(&destino).await.unwrap();

    // A row that is short one value fails halfway through the load. Before this was one
    // transaction, the tables already processed stayed replaced and the rest stale.
    let mut documento: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&destino).unwrap()).unwrap();
    for tabla in documento["tables"].as_array_mut().unwrap() {
        if tabla["name"] == "movimientos" {
            tabla["rows"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!(["fila", "incompleta"]));
        }
    }
    std::fs::write(&destino, serde_json::to_vec(&documento).unwrap()).unwrap();

    let error = entorno.service.import_json(&destino).await.unwrap_err();
    assert_clave(&error, "Validation.Import.FilaIncompleta");
    assert_eq!(
        cuantos_tipos(&entorno).await,
        esperados,
        "el import dejó la base a medio reemplazar"
    );
}

#[tokio::test]
async fn import_json_rechaza_un_esquema_distinto() {
    let entorno = entorno().await;
    let destino = entorno.paths.root().join("volcado.json");
    entorno.service.export_json(&destino).await.unwrap();

    let mut documento: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&destino).unwrap()).unwrap();
    documento["schemaVersion"] = serde_json::json!("m20990101_000001_del_futuro");
    std::fs::write(&destino, serde_json::to_vec(&documento).unwrap()).unwrap();

    let error = entorno.service.import_json(&destino).await.unwrap_err();
    assert_clave(&error, "Validation.Import.EsquemaIncompatible");
}

#[tokio::test]
async fn import_json_rechaza_un_formato_distinto() {
    let entorno = entorno().await;
    let destino = entorno.paths.root().join("volcado.json");
    entorno.service.export_json(&destino).await.unwrap();

    let mut documento: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&destino).unwrap()).unwrap();
    documento["formatVersion"] = serde_json::json!(1);
    std::fs::write(&destino, serde_json::to_vec(&documento).unwrap()).unwrap();

    let error = entorno.service.import_json(&destino).await.unwrap_err();
    assert_clave(&error, "Validation.Import.FormatoIncompatible");
}

/// The dump is a mirror of the schema, so a table added to the model without adding it here would
/// silently stop being backed up.
#[tokio::test]
async fn el_volcado_cubre_todas_las_tablas_del_esquema() {
    let entorno = entorno().await;
    let db = entorno.db.read().await;
    let filas = db
        .query_all(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'"
                .to_owned(),
        ))
        .await
        .unwrap();

    let ignoradas = ["seaql_migrations", "app_metadata"];
    for fila in filas {
        let nombre: String = fila.try_get("", "name").unwrap();
        if ignoradas.contains(&nombre.as_str()) {
            continue;
        }
        assert!(
            certaro_infrastructure::backup::json::TABLAS.contains(&nombre.as_str()),
            "la tabla {nombre} no está en el volcado"
        );
    }
}
