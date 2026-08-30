//! Use cases of the system section: backups, the JSON dump and the maintenance task.
//!
//! See `docs/13-servicios-externos-y-archivos.md` §4, §5 and §6.

use std::path::Path;
use std::sync::Arc;

use serde::Serialize;
use tracing::{info, warn};

use crate::dtos::dashboard::EstadoSistema;
use crate::error::{AppError, FieldError};
use crate::ports::repositories::UnitOfWork;
use crate::ports::{AttachmentStore, BackupItem, BackupPort, ImportResumen, SettingsStore};
use crate::ports::{ClockPort, VerificacionBackup};
use crate::result::AppResult;
use crate::use_cases::shared::finish_read;

/// What the maintenance task did, for the log and for the system section to show.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultadoMantenimiento {
    pub backup_creado: bool,
    pub backups_borrados: u32,
    pub adjuntos_purgados: u32,
}

pub struct SistemaService {
    uow: Arc<dyn UnitOfWork>,
    backup: Arc<dyn BackupPort>,
    attachments: Arc<dyn AttachmentStore>,
    settings: Arc<dyn SettingsStore>,
    clock: Arc<dyn ClockPort>,
}

impl SistemaService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        backup: Arc<dyn BackupPort>,
        attachments: Arc<dyn AttachmentStore>,
        settings: Arc<dyn SettingsStore>,
        clock: Arc<dyn ClockPort>,
    ) -> Self {
        Self {
            uow,
            backup,
            attachments,
            settings,
            clock,
        }
    }

    /// Version, database health, applied migrations and file size: what the system section shows
    /// and what a user reports when something is wrong.
    pub async fn info(&self, version: &str) -> AppResult<EstadoSistema> {
        let tx = self.uow.begin().await?;
        let cargado = tx.dashboard().estado_base().await;
        let base = finish_read(tx, cargado).await?;
        Ok(EstadoSistema {
            version: version.to_owned(),
            base_saludable: base.healthy,
            estado: if base.healthy {
                "Dashboard.Estado.Saludable".to_owned()
            } else {
                "Dashboard.Estado.Error".to_owned()
            },
            migraciones: base.migraciones,
            tamano_bytes: base.tamano_bytes,
        })
    }

    pub async fn backups(&self) -> AppResult<Vec<BackupItem>> {
        self.backup.list().await
    }

    pub async fn backup_create(&self) -> AppResult<BackupItem> {
        self.backup.create().await
    }

    pub async fn backup_verify(&self, nombre: &str) -> AppResult<VerificacionBackup> {
        self.backup.verify(nombre).await
    }

    pub async fn backup_restore(&self, nombre: &str) -> AppResult<()> {
        self.backup.restore(nombre).await
    }

    pub async fn export_json(&self, destino: &str) -> AppResult<ImportResumen> {
        let destino = Path::new(destino);
        validar_destino(destino)?;
        self.backup.export_json(destino).await
    }

    pub async fn import_json(&self, origen: &str) -> AppResult<ImportResumen> {
        let origen = Path::new(origen);
        if !origen.is_file() {
            return Err(validacion("origen", "Validation.Import.ArchivoNoExiste"));
        }
        self.backup.import_json(origen).await
    }

    /// Housekeeping at startup. See `docs/13` §6.
    ///
    /// Nothing here blocks the interface and nothing here fails loudly: each task logs its own
    /// problem and the next one still runs. A failed prune must not stop the trash from emptying.
    pub async fn mantenimiento(&self) -> ResultadoMantenimiento {
        let config = self.settings.snapshot();
        let mut resultado = ResultadoMantenimiento::default();

        if config.backup.enabled {
            match self.backup_por_antiguedad(config.backup.max_age_days).await {
                Ok(creado) => resultado.backup_creado = creado,
                Err(e) => warn!(error = %e, "no se pudo crear el backup automático"),
            }

            match self
                .backup
                .prune(
                    config.backup.retention_days,
                    config.backup.minimo_a_conservar as usize,
                )
                .await
            {
                Ok(borrados) => resultado.backups_borrados = borrados,
                Err(e) => warn!(error = %e, "no se pudieron limpiar los backups viejos"),
            }
        }

        match self
            .attachments
            .purge_trash(config.attachments.trash_retention_days)
            .await
        {
            Ok(purgados) => resultado.adjuntos_purgados = purgados,
            Err(e) => warn!(error = %e, "no se pudo vaciar la papelera de adjuntos"),
        }

        info!(
            backup_creado = resultado.backup_creado,
            backups_borrados = resultado.backups_borrados,
            adjuntos_purgados = resultado.adjuntos_purgados,
            "mantenimiento terminado"
        );
        resultado
    }

    /// Creates a backup only when the newest one is older than `max_age_days`, or when there is
    /// none at all.
    async fn backup_por_antiguedad(&self, max_age_days: u32) -> AppResult<bool> {
        let items = self.backup.list().await?;
        let limite = self.clock.now_utc() - chrono::Duration::days(i64::from(max_age_days));
        let al_dia = items.first().is_some_and(|ultimo| ultimo.creado_en >= limite);
        if al_dia {
            return Ok(false);
        }
        self.backup.create().await?;
        Ok(true)
    }
}

/// The destination of a dump: an existing directory and a `.json` name.
fn validar_destino(destino: &Path) -> AppResult<()> {
    let directorio = destino.parent().unwrap_or(Path::new(""));
    if !directorio.as_os_str().is_empty() && !directorio.is_dir() {
        return Err(validacion("destino", "Validation.Export.DirectorioNoExiste"));
    }
    let extension = destino
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase);
    if extension.as_deref() != Some("json") {
        return Err(validacion("destino", "Validation.Export.ExtensionNoCoincide"));
    }
    Ok(())
}

fn validacion(campo: &str, clave: &str) -> AppError {
    AppError::Validation(vec![FieldError::new(campo, clave)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_destino_sin_extension_json_se_rechaza() {
        let dir = tempfile::tempdir().unwrap();
        assert!(validar_destino(&dir.path().join("volcado.db")).is_err());
        assert!(validar_destino(&dir.path().join("volcado.json")).is_ok());
        // The comparison is case-insensitive: a dialog on Windows may hand back `.JSON`.
        assert!(validar_destino(&dir.path().join("volcado.JSON")).is_ok());
    }

    #[test]
    fn un_directorio_inexistente_se_rechaza_antes_de_escribir() {
        assert!(validar_destino(Path::new("D:/no/existe/volcado.json")).is_err());
    }
}
