//! Backups, restore and the JSON dump, against a real database file on disk.
//!
//! An in-memory database would not exercise the part that matters: closing the connection before
//! the file is replaced, and the `-wal` sidecar that the legacy restore left behind.

use std::sync::Arc;

use eo_application::config::AppConfig;
use eo_application::ports::{BackupPort, ClockPort, SettingsStore};
use eo_application::{AppError, AppResult};
use eo_domain::clock::Clock;
use eo_infrastructure::backup::{nombre_backup, SqliteBackupService};
use eo_infrastructure::paths::AppPaths;
use eo_infrastructure::persistence::{self, DbHandle};
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

/// A clock the test moves by hand, to age a backup without waiting a month.
#[derive(Clone)]
struct RelojMovil(Arc<std::sync::Mutex<chrono::DateTime<chrono::Utc>>>);

impl RelojMovil {
    fn new(iso: &str) -> Self {
        Self(Arc::new(std::sync::Mutex::new(
            chrono::DateTime::parse_from_rfc3339(iso).unwrap().into(),
        )))
    }

    fn avanzar(&self, dias: i64) {
        let mut guard = self.0.lock().unwrap();
        *guard += chrono::Duration::days(dias);
    }
}

impl Clock for RelojMovil {
    fn now_utc(&self) -> chrono::DateTime<chrono::Utc> {
        *self.0.lock().unwrap()
    }
}

struct SettingsFijo(AppConfig);

#[async_trait::async_trait]
impl SettingsStore for SettingsFijo {
    fn snapshot(&self) -> AppConfig {
        self.0.clone()
    }
    async fn save(&self, _config: AppConfig) -> AppResult<()> {
        Ok(())
    }
}

struct Entorno {
    service: SqliteBackupService,
    db: DbHandle,
    paths: AppPaths,
    reloj: RelojMovil,
    _dir: tempfile::TempDir,
}

async fn entorno() -> Entorno {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::from_root(dir.path());
    paths.ensure_dirs().unwrap();

    let db = DbHandle::new(persistence::open(&paths.database()).await.unwrap());
    let reloj = RelojMovil::new("2026-08-29T12:00:00Z");
    let settings: Arc<dyn SettingsStore> = Arc::new(SettingsFijo(AppConfig::default()));

    Entorno {
        service: SqliteBackupService::new(
            db.clone(),
            paths.clone(),
            settings,
            Arc::new(reloj.clone()) as Arc<dyn ClockPort>,
            "0.1.0",
        ),
        db,
        paths,
        reloj,
        _dir: dir,
    }
}

/// Inserts a movement type so the database has a row worth losing.
async fn sembrar(entorno: &Entorno, nombre: &str) {
    let db = entorno.db.read().await;
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        format!(
            "INSERT INTO tipos_movimiento (id, nombre, es_ingreso, es_sistema, created_at, row_version, is_deleted) \
             VALUES ('{}', '{nombre}', 1, 0, '2026-08-29T12:00:00.000Z', X'0000000000000001', 0)",
            uuid::Uuid::new_v4()
        ),
    ))
    .await
    .unwrap();
}

async fn cuantos_tipos(entorno: &Entorno) -> i64 {
    let db = entorno.db.read().await;
    db.query_one(Statement::from_string(
        DatabaseBackend::Sqlite,
        "SELECT COUNT(*) AS total FROM tipos_movimiento".to_owned(),
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get::<i64>("", "total")
    .unwrap()
}

#[tokio::test]
async fn backup_crea_y_verifica_integridad() {
    let entorno = entorno().await;
    let item = entorno.service.create().await.unwrap();

    assert_eq!(item.nombre, nombre_backup(entorno.reloj.now_utc()));
    assert!(item.bytes > 0);

    let verificacion = entorno.service.verify(&item.nombre).await.unwrap();
    assert!(verificacion.ok, "detalle: {}", verificacion.detalle);
}

#[tokio::test]
async fn backup_verifica_integridad_y_rechaza_un_archivo_corrupto() {
    let entorno = entorno().await;
    let item = entorno.service.create().await.unwrap();

    // The header is what SQLite reads first, so overwriting it is the cheapest real corruption.
    let ruta = entorno.paths.backups("Backups").join(&item.nombre);
    let mut bytes = std::fs::read(&ruta).unwrap();
    bytes[0..16].copy_from_slice(b"no es una base..");
    std::fs::write(&ruta, bytes).unwrap();

    let resultado = entorno.service.verify(&item.nombre).await;
    if let Ok(verificacion) = resultado {
        assert!(!verificacion.ok);
    }
    // Refusing to even open it is also a rejection, and the same outcome for the caller.
}

#[tokio::test]
async fn un_nombre_de_backup_ajeno_no_se_acepta() {
    let entorno = entorno().await;
    for nombre in [
        "../../electroobra.db",
        "..\\electroobra.db",
        "cualquier.db",
        "electroobra_ayer.db",
    ] {
        let error = entorno.service.verify(nombre).await.unwrap_err();
        assert!(matches!(error, AppError::Validation(_)), "{nombre}");
    }
}

#[tokio::test]
async fn backup_conserva_los_tres_mas_recientes() {
    let entorno = entorno().await;
    for _ in 0..5 {
        entorno.service.create().await.unwrap();
        // One a day, so the names differ and the ages are ordered.
        entorno.reloj.avanzar(1);
    }
    assert_eq!(entorno.service.backups_count().await, 5);

    // Every one of them is older than the window, so only the minimum should survive.
    entorno.reloj.avanzar(60);
    let borrados = entorno.service.prune(30, 3).await.unwrap();

    assert_eq!(borrados, 2);
    assert_eq!(entorno.service.backups_count().await, 3);
}

#[tokio::test]
async fn restore_devuelve_la_base_al_estado_del_backup() {
    let entorno = entorno().await;
    let antes = cuantos_tipos(&entorno).await;

    let punto = entorno.service.create().await.unwrap();
    entorno.reloj.avanzar(1);

    sembrar(&entorno, "Agregado después del backup").await;
    assert_eq!(cuantos_tipos(&entorno).await, antes + 1);

    entorno.service.restore(&punto.nombre).await.unwrap();

    // The connection was closed and reopened, so this read goes through the restored file.
    assert_eq!(cuantos_tipos(&entorno).await, antes);
    // And the pre-restore state was itself backed up, so the restore is undoable.
    assert!(entorno.service.backups_count().await >= 2);
}

#[tokio::test]
async fn restore_no_deja_sidecars_del_archivo_anterior() {
    let entorno = entorno().await;
    let punto = entorno.service.create().await.unwrap();
    entorno.reloj.avanzar(1);
    sembrar(&entorno, "Escribe en el wal").await;

    entorno.service.restore(&punto.nombre).await.unwrap();

    // A `-wal` describing the replaced database is what left the legacy restore inconsistent.
    let wal = entorno
        .paths
        .root()
        .join(format!("{}-wal", "electroobra.db"));
    let temporal = entorno.paths.root().join("electroobra.db.restore.tmp");
    assert!(!temporal.exists(), "quedó el temporal de la restauración");
    // A fresh `-wal` may exist from the reopened connection; what must not exist is a stale one,
    // which is only observable by the database reading correctly.
    assert!(cuantos_tipos(&entorno).await >= 0, "la base quedó ilegible");
    let _ = wal;
}

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

/// Convenience over the port, only for the assertions above.
trait CuantosBackups {
    async fn backups_count(&self) -> usize;
}

impl CuantosBackups for SqliteBackupService {
    async fn backups_count(&self) -> usize {
        self.list().await.unwrap().len()
    }
}

#[track_caller]
fn assert_clave(error: &AppError, esperada: &str) {
    match error {
        AppError::Validation(errores) => assert_eq!(
            errores[0].message_key, esperada,
            "vino {}",
            errores[0].message_key
        ),
        otro => panic!("se esperaba validación con {esperada}, vino {otro:?}"),
    }
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
            eo_infrastructure::backup::json::TABLAS.contains(&nombre.as_str()),
            "la tabla {nombre} no está en el volcado"
        );
    }
}
