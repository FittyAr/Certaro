//! Test harness and helpers for backup integration tests.

use std::sync::Arc;

use certaro_application::config::AppConfig;
use certaro_application::ports::{BackupPort, ClockPort, SettingsStore};
use certaro_application::{AppError, AppResult};
use certaro_domain::clock::Clock;
use certaro_infrastructure::backup::SqliteBackupService;
use certaro_infrastructure::paths::AppPaths;
use certaro_infrastructure::persistence::{self, DbHandle};
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

/// A clock the test moves by hand, to age a backup without waiting a month.
#[derive(Clone)]
pub struct RelojMovil(Arc<std::sync::Mutex<chrono::DateTime<chrono::Utc>>>);

impl RelojMovil {
    pub fn new(iso: &str) -> Self {
        Self(Arc::new(std::sync::Mutex::new(
            chrono::DateTime::parse_from_rfc3339(iso).unwrap().into(),
        )))
    }

    pub fn avanzar(&self, dias: i64) {
        let mut guard = self.0.lock().unwrap();
        *guard += chrono::Duration::days(dias);
    }
}

impl Clock for RelojMovil {
    fn now_utc(&self) -> chrono::DateTime<chrono::Utc> {
        *self.0.lock().unwrap()
    }
}

pub struct SettingsFijo(pub AppConfig);

#[async_trait::async_trait]
impl SettingsStore for SettingsFijo {
    fn snapshot(&self) -> AppConfig {
        self.0.clone()
    }
    async fn save(&self, _config: AppConfig) -> AppResult<()> {
        Ok(())
    }
}

pub struct Entorno {
    pub service: SqliteBackupService,
    pub db: DbHandle,
    pub paths: AppPaths,
    pub reloj: RelojMovil,
    pub _dir: tempfile::TempDir,
}

pub async fn entorno() -> Entorno {
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
pub async fn sembrar(entorno: &Entorno, nombre: &str) {
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

pub async fn cuantos_tipos(entorno: &Entorno) -> i64 {
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

pub trait CuantosBackups {
    async fn backups_count(&self) -> usize;
}

impl CuantosBackups for SqliteBackupService {
    async fn backups_count(&self) -> usize {
        self.list().await.unwrap().len()
    }
}

#[track_caller]
pub fn assert_clave(error: &AppError, esperada: &str) {
    match error {
        AppError::Validation(errores) => assert_eq!(
            errores[0].message_key, esperada,
            "vino {}",
            errores[0].message_key
        ),
        otro => panic!("se esperaba validación con {esperada}, vino {otro:?}"),
    }
}
