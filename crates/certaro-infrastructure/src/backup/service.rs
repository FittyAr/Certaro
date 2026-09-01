//! Backup and restore over SQLite. See `docs/13-servicios-externos-y-archivos.md` §4.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Duration;
use certaro_application::error::FieldError;
use certaro_application::ports::{
    BackupItem, BackupPort, ClockPort, ImportResumen, SettingsStore, VerificacionBackup,
};
use certaro_application::result::AppResult;
use certaro_application::AppError;
use certaro_migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use tracing::{info, warn};

use super::{instante_de, json, nombre_backup};
use crate::paths::AppPaths;
use crate::persistence::{self, DbHandle};

/// Sidecars of a WAL database. Left behind, they describe a database that no longer exists.
const SIDECARS: [&str; 2] = ["-wal", "-shm"];

pub struct SqliteBackupService {
    db: DbHandle,
    paths: AppPaths,
    settings: Arc<dyn SettingsStore>,
    clock: Arc<dyn ClockPort>,
    app_version: String,
}

impl SqliteBackupService {
    #[must_use]
    pub fn new(
        db: DbHandle,
        paths: AppPaths,
        settings: Arc<dyn SettingsStore>,
        clock: Arc<dyn ClockPort>,
        app_version: impl Into<String>,
    ) -> Self {
        Self {
            db,
            paths,
            settings,
            clock,
            app_version: app_version.into(),
        }
    }

    fn directorio(&self) -> PathBuf {
        self.paths
            .backups(&self.settings.snapshot().backup.directory)
    }

    /// Resolves a backup name to its path, refusing anything that is not a plain file name.
    ///
    /// The name arrives from the frontend, so `..\..\certaro.db` has to be impossible: it would
    /// otherwise let a restore read, and a prune delete, any file on the disk.
    fn ruta_de(&self, nombre: &str) -> AppResult<PathBuf> {
        if instante_de(nombre).is_none() {
            return Err(validacion("nombre", "Validation.Backup.NombreInvalido"));
        }
        let ruta = self.directorio().join(nombre);
        if !ruta.is_file() {
            return Err(validacion("nombre", "Validation.Backup.NoExiste"));
        }
        Ok(ruta)
    }

    /// The last applied migration, which is the schema version the JSON dump records.
    async fn schema_version(&self) -> AppResult<String> {
        let db = self.db.read().await;
        let aplicadas = Migrator::get_applied_migrations(&*db)
            .await
            .map_err(AppError::persistence)?;
        Ok(aplicadas
            .last()
            .map(|m| m.name().to_owned())
            .unwrap_or_default())
    }
}

#[async_trait]
impl BackupPort for SqliteBackupService {
    async fn list(&self) -> AppResult<Vec<BackupItem>> {
        let directorio = self.directorio();
        let mut items = Vec::new();
        let mut entradas = match tokio::fs::read_dir(&directorio).await {
            Ok(e) => e,
            // No directory means no backups yet, which is a fact, not a failure.
            Err(_) => return Ok(items),
        };
        while let Ok(Some(entrada)) = entradas.next_entry().await {
            let nombre = entrada.file_name().to_string_lossy().into_owned();
            let Some(creado_en) = instante_de(&nombre) else {
                continue;
            };
            let bytes = entrada.metadata().await.map(|m| m.len()).unwrap_or(0);
            items.push(BackupItem {
                nombre,
                creado_en,
                bytes,
            });
        }
        items.sort_by_key(|a| std::cmp::Reverse(a.creado_en));
        Ok(items)
    }

    async fn create(&self) -> AppResult<BackupItem> {
        let directorio = self.directorio();
        tokio::fs::create_dir_all(&directorio).await.map_err(|e| {
            AppError::io(anyhow::anyhow!(
                "backup.mkdir {}: {e}",
                directorio.display()
            ))
        })?;

        let nombre = nombre_backup(self.clock.now_utc());
        let destino = directorio.join(&nombre);

        {
            let db = self.db.read().await;
            vacuum_into(&db, &destino).await?;
        }

        let verificacion = self.verify(&nombre).await?;
        if !verificacion.ok {
            // A copy that does not verify is worse than no copy, because it would be trusted.
            let _ = tokio::fs::remove_file(&destino).await;
            return Err(validacion("backup", "Validation.Backup.IntegridadFallida"));
        }

        let bytes = tokio::fs::metadata(&destino)
            .await
            .map(|m| m.len())
            .unwrap_or(0);
        info!(nombre = %nombre, bytes, "backup creado");

        Ok(BackupItem {
            creado_en: instante_de(&nombre).unwrap_or_else(|| self.clock.now_utc()),
            nombre,
            bytes,
        })
    }

    async fn verify(&self, nombre: &str) -> AppResult<VerificacionBackup> {
        let ruta = self.ruta_de(nombre)?;
        let db = persistence::open_readonly(&ruta)
            .await
            .map_err(AppError::persistence)?;
        let detalle = integrity_check(&db).await?;
        db.close().await.map_err(AppError::persistence)?;
        Ok(VerificacionBackup {
            ok: detalle.eq_ignore_ascii_case("ok"),
            detalle,
        })
    }

    async fn restore(&self, nombre: &str) -> AppResult<()> {
        let origen = self.ruta_de(nombre)?;

        let verificacion = self.verify(nombre).await?;
        if !verificacion.ok {
            return Err(validacion("nombre", "Validation.Backup.IntegridadFallida"));
        }

        // A backup written by a newer version of the application carries a schema this one does not
        // know how to read, and migrations only go forward.
        if !self.esquema_compatible(&origen).await? {
            return Err(validacion(
                "nombre",
                "Validation.Backup.VersionIncompatible",
            ));
        }

        // Undo for the restore itself.
        let previo = self.create().await?;

        let destino = self.paths.database();
        self.db.disconnect().await.map_err(AppError::persistence)?;

        let resultado = reemplazar_archivo(&origen, &destino).await;

        // The connection is reopened whether or not the copy worked: leaving the handle
        // disconnected would take the application down with the failed restore.
        let db = persistence::open(&destino)
            .await
            .map_err(AppError::persistence)?;
        self.db.replace(db).await;

        resultado?;
        warn!(restaurado = %nombre, respaldo_previo = %previo.nombre, "base restaurada desde backup");
        Ok(())
    }

    async fn prune(&self, dias: u32, minimo: usize) -> AppResult<u32> {
        let items = self.list().await?;
        let limite = self.clock.now_utc() - Duration::days(i64::from(dias));
        let directorio = self.directorio();
        let mut borrados = 0;

        // Newest first, so skipping the first `minimo` keeps exactly those.
        for item in items.into_iter().skip(minimo) {
            if item.creado_en < limite
                && tokio::fs::remove_file(directorio.join(&item.nombre))
                    .await
                    .is_ok()
            {
                borrados += 1;
            }
        }
        if borrados > 0 {
            info!(borrados, dias, minimo, "backups viejos eliminados");
        }
        Ok(borrados)
    }

    async fn export_json(&self, destino: &Path) -> AppResult<ImportResumen> {
        let schema_version = self.schema_version().await?;
        let exportado = self.clock.now_utc();

        let (documento, resumen) = {
            let db = self.db.read().await;
            json::exportar(
                &db,
                &self.app_version,
                schema_version,
                certaro_domain::time::to_storage(exportado),
            )
            .await?
        };

        let bytes = serde_json::to_vec_pretty(&documento)
            .map_err(|e| AppError::io(anyhow::anyhow!("export.json serialize: {e}")))?;
        tokio::fs::write(destino, bytes).await.map_err(|e| {
            AppError::io(anyhow::anyhow!(
                "export.json write {}: {e}",
                destino.display()
            ))
        })?;

        Ok(resumen)
    }

    async fn import_json(&self, origen: &Path) -> AppResult<ImportResumen> {
        let bytes = tokio::fs::read(origen).await.map_err(|e| {
            AppError::io(anyhow::anyhow!(
                "import.json read {}: {e}",
                origen.display()
            ))
        })?;
        let documento: json::Documento = serde_json::from_slice(&bytes)
            .map_err(|_| validacion("archivo", "Validation.Import.ArchivoInvalido"))?;

        // Before touching anything: the import replaces every table.
        self.create().await?;

        let schema_version = self.schema_version().await?;
        let db = self.db.read().await;
        json::importar(&db, &documento, &schema_version).await
    }
}

impl SqliteBackupService {
    /// Whether the backup's schema is one this build can open: its last migration has to be one we
    /// know, and not something added after this version was built.
    async fn esquema_compatible(&self, ruta: &Path) -> AppResult<bool> {
        let db = persistence::open_readonly(ruta)
            .await
            .map_err(AppError::persistence)?;
        let aplicadas = Migrator::get_applied_migrations(&db).await;
        let cerrada = db.close().await;

        let aplicadas = aplicadas.map_err(AppError::persistence)?;
        cerrada.map_err(AppError::persistence)?;

        let conocidas: Vec<String> = Migrator::migrations()
            .iter()
            .map(|m| m.name().to_owned())
            .collect();
        Ok(aplicadas
            .last()
            .map_or(true, |ultima| conocidas.iter().any(|c| c == ultima.name())))
    }
}

/// `VACUUM INTO` produces a consistent copy without closing the application, which is why it is
/// used instead of copying the file.
async fn vacuum_into(db: &DatabaseConnection, destino: &Path) -> AppResult<()> {
    // The only text interpolation in a SQL statement in the whole system, because `VACUUM INTO`
    // takes no parameters. The path is built from the data directory and a timestamp, never from
    // user input, and single quotes are doubled all the same.
    let ruta = destino.display().to_string().replace('\'', "''");
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        format!("VACUUM INTO '{ruta}'"),
    ))
    .await
    .map_err(AppError::persistence)?;
    Ok(())
}

async fn integrity_check(db: &DatabaseConnection) -> AppResult<String> {
    let fila = db
        .query_one(Statement::from_string(
            DatabaseBackend::Sqlite,
            "PRAGMA integrity_check".to_owned(),
        ))
        .await
        .map_err(AppError::persistence)?;
    Ok(fila
        .and_then(|f| f.try_get::<String>("", "integrity_check").ok())
        .unwrap_or_else(|| "sin respuesta".to_owned()))
}

/// Copies through a temporary file and clears the sidecars, so a failure halfway does not leave a
/// database file that is neither the old one nor the new one.
async fn reemplazar_archivo(origen: &Path, destino: &Path) -> AppResult<()> {
    let temporal = destino.with_extension("db.restore.tmp");

    tokio::fs::copy(origen, &temporal)
        .await
        .map_err(|e| AppError::io(anyhow::anyhow!("restore.copy: {e}")))?;

    for sufijo in SIDECARS {
        let sidecar = PathBuf::from(format!("{}{sufijo}", destino.display()));
        let _ = tokio::fs::remove_file(sidecar).await;
    }

    let resultado = tokio::fs::rename(&temporal, destino)
        .await
        .map_err(|e| AppError::io(anyhow::anyhow!("restore.rename: {e}")));
    if resultado.is_err() {
        let _ = tokio::fs::remove_file(&temporal).await;
    }
    resultado
}

fn validacion(campo: &str, clave: &str) -> AppError {
    AppError::Validation(vec![FieldError::new(campo, clave)])
}
