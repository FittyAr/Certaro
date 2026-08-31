//! The attachment store on the local filesystem. See `docs/13-servicios-externos-y-archivos.md` §1.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use eo_application::error::FieldError;
use eo_application::ports::{
    ArchivoAceptado, ArchivoGuardado, AttachmentStore, ClockPort, SettingsStore,
};
use eo_application::result::AppResult;
use eo_application::AppError;
use eo_domain::entities::{Adjunto, EntidadAdjunto};
use tracing::{info, warn};
use uuid::Uuid;

use super::{mime, name};
use crate::paths::AppPaths;

/// Bytes read to check the signature. Enough for every prefix in §1.4.
const CABECERA: usize = 16;

const MB: u64 = 1024 * 1024;

pub struct FsAttachmentStore {
    paths: AppPaths,
    settings: Arc<dyn SettingsStore>,
    clock: Arc<dyn ClockPort>,
}

impl FsAttachmentStore {
    #[must_use]
    pub fn new(
        paths: AppPaths,
        settings: Arc<dyn SettingsStore>,
        clock: Arc<dyn ClockPort>,
    ) -> Self {
        Self {
            paths,
            settings,
            clock,
        }
    }

    fn absoluta(&self, ruta_relativa: &str) -> PathBuf {
        // The stored value uses `/` on every platform, so it is rebuilt segment by segment rather
        // than handed to `Path` as one string.
        ruta_relativa
            .split('/')
            .fold(self.paths.attachments(), |acc, segmento| acc.join(segmento))
    }
}

#[async_trait]
impl AttachmentStore for FsAttachmentStore {
    async fn accept(&self, origen: &Path, cupo_restante: u64) -> AppResult<ArchivoAceptado> {
        let config = self.settings.snapshot().attachments;

        let metadata = tokio::fs::symlink_metadata(origen).await.map_err(|e| {
            AppError::io(anyhow::anyhow!("attachment.stat {}: {e}", origen.display()))
        })?;
        if !metadata.is_file() {
            // A directory or a symlink: neither is a file to copy, and following the link would
            // read something the user did not pick.
            return Err(validacion("rutaOrigen", "Validation.Adjunto.NoEsArchivo"));
        }

        let nombre = name::sanitize(
            origen
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default(),
        );
        let extension = name::extension_de(&nombre);
        let Some(mime) = mime::de_extension(&extension) else {
            return Err(
                validacion("rutaOrigen", "Validation.Adjunto.ExtensionNoPermitida")
                    .con("extension", &extension),
            );
        };

        let tamano = metadata.len();
        let maximo = u64::from(config.max_size_mb) * MB;
        if tamano > maximo {
            return Err(
                validacion("rutaOrigen", "Validation.Adjunto.DemasiadoGrande")
                    .con("max", &config.max_size_mb.to_string())
                    .con("actual", &format!("{:.1}", tamano as f64 / MB as f64)),
            );
        }
        if tamano > cupo_restante {
            return Err(validacion("rutaOrigen", "Validation.Adjunto.CupoExcedido")
                .con("max", &config.max_total_mb.to_string()));
        }

        let cabecera = leer_cabecera(origen).await?;
        if !mime::contenido_coincide(&extension, &cabecera) {
            return Err(
                validacion("rutaOrigen", "Validation.Adjunto.ContenidoNoCoincide")
                    .con("extension", &extension),
            );
        }

        Ok(ArchivoAceptado {
            nombre,
            mime,
            tamano,
        })
    }

    async fn store(
        &self,
        origen: &Path,
        entidad_tipo: EntidadAdjunto,
        entidad_id: Uuid,
        id: Uuid,
        archivo: ArchivoAceptado,
    ) -> AppResult<ArchivoGuardado> {
        let ruta_relativa = Adjunto::ruta_para(entidad_tipo, entidad_id, id, &archivo.nombre);
        let destino = self.absoluta(&ruta_relativa);
        if let Some(directorio) = destino.parent() {
            tokio::fs::create_dir_all(directorio).await.map_err(|e| {
                AppError::io(anyhow::anyhow!(
                    "attachment.mkdir {}: {e}",
                    directorio.display()
                ))
            })?;
        }
        tokio::fs::copy(origen, &destino).await.map_err(|e| {
            AppError::io(anyhow::anyhow!(
                "attachment.copy {}: {e}",
                destino.display()
            ))
        })?;

        info!(
            entidad = entidad_tipo.as_str(),
            %entidad_id,
            bytes = archivo.tamano,
            "adjunto guardado"
        );

        Ok(ArchivoGuardado {
            archivo,
            ruta_relativa,
        })
    }

    async fn trash(&self, ruta_relativa: &str) -> AppResult<()> {
        let origen = self.absoluta(ruta_relativa);
        if !origen.exists() {
            // The row is about to be deleted anyway; a file that is already gone is not a reason to
            // refuse, and the warning is the record of it.
            warn!(ruta = ruta_relativa, "el adjunto ya no estaba en disco");
            return Ok(());
        }

        let papelera = self.paths.attachments_trash();
        tokio::fs::create_dir_all(&papelera).await.map_err(|e| {
            AppError::io(anyhow::anyhow!(
                "attachment.mkdir {}: {e}",
                papelera.display()
            ))
        })?;

        // The timestamp prefix keeps two files of the same name apart and is what the purge reads,
        // so the trash does not depend on the filesystem's own timestamps.
        let nombre = origen
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("archivo");
        let destino = papelera.join(format!(
            "{}_{nombre}",
            self.clock.now_utc().format("%Y%m%dT%H%M%S")
        ));

        if tokio::fs::rename(&origen, &destino).await.is_err() {
            // Across volumes rename fails; copy and remove is the fallback.
            tokio::fs::copy(&origen, &destino).await.map_err(|e| {
                AppError::io(anyhow::anyhow!(
                    "attachment.trash {}: {e}",
                    origen.display()
                ))
            })?;
            tokio::fs::remove_file(&origen).await.map_err(|e| {
                AppError::io(anyhow::anyhow!(
                    "attachment.remove {}: {e}",
                    origen.display()
                ))
            })?;
        }

        limpiar_directorio_vacio(origen.parent()).await;
        Ok(())
    }

    fn resolve(&self, ruta_relativa: &str) -> AppResult<PathBuf> {
        let ruta = self.absoluta(ruta_relativa);
        if ruta.is_file() {
            Ok(ruta)
        } else {
            Err(validacion("id", "Validation.Adjunto.ArchivoNoEncontrado")
                .con("ruta", ruta_relativa))
        }
    }

    async fn usado_por(&self, entidad_tipo: EntidadAdjunto, entidad_id: Uuid) -> AppResult<u64> {
        let directorio = self
            .paths
            .attachments()
            .join(entidad_tipo.as_str())
            .join(entidad_id.to_string());
        let mut total = 0;
        let mut entradas = match tokio::fs::read_dir(&directorio).await {
            Ok(e) => e,
            // No directory means nothing attached yet, which is a quota of zero, not an error.
            Err(_) => return Ok(0),
        };
        while let Ok(Some(entrada)) = entradas.next_entry().await {
            if let Ok(metadata) = entrada.metadata().await {
                if metadata.is_file() {
                    total += metadata.len();
                }
            }
        }
        Ok(total)
    }

    async fn purge_trash(&self, dias: u32) -> AppResult<u32> {
        let limite = self.clock.now_utc() - Duration::days(i64::from(dias));
        let papelera = self.paths.attachments_trash();
        let mut borrados = 0;
        let mut entradas = match tokio::fs::read_dir(&papelera).await {
            Ok(e) => e,
            Err(_) => return Ok(0),
        };
        while let Ok(Some(entrada)) = entradas.next_entry().await {
            let nombre = entrada.file_name();
            let Some(nombre) = nombre.to_str() else {
                continue;
            };
            let Some(marca) = marca_de(nombre) else {
                continue;
            };
            // Inclusive, so a retention of zero days means «empty it now» rather than «almost».
            if marca <= limite && tokio::fs::remove_file(entrada.path()).await.is_ok() {
                borrados += 1;
            }
        }
        if borrados > 0 {
            info!(borrados, dias, "papelera de adjuntos vaciada");
        }
        Ok(borrados)
    }
}

/// The instant encoded in a trashed file's prefix.
fn marca_de(nombre: &str) -> Option<DateTime<Utc>> {
    let (marca, _) = nombre.split_once('_')?;
    chrono::NaiveDateTime::parse_from_str(marca, "%Y%m%dT%H%M%S")
        .ok()
        .map(|naive| naive.and_utc())
}

async fn leer_cabecera(origen: &Path) -> AppResult<Vec<u8>> {
    use tokio::io::AsyncReadExt;
    let mut archivo = tokio::fs::File::open(origen)
        .await
        .map_err(|e| AppError::io(anyhow::anyhow!("attachment.open {}: {e}", origen.display())))?;
    let mut buffer = vec![0_u8; CABECERA];
    let leidos = archivo
        .read(&mut buffer)
        .await
        .map_err(|e| AppError::io(anyhow::anyhow!("attachment.read {}: {e}", origen.display())))?;
    buffer.truncate(leidos);
    Ok(buffer)
}

/// Removes the entity directory once its last attachment is gone (§1.6).
async fn limpiar_directorio_vacio(directorio: Option<&Path>) {
    let Some(directorio) = directorio else { return };
    if let Ok(mut entradas) = tokio::fs::read_dir(directorio).await {
        if matches!(entradas.next_entry().await, Ok(None)) {
            let _ = tokio::fs::remove_dir(directorio).await;
        }
    }
}

fn validacion(campo: &str, clave: &str) -> AppError {
    AppError::Validation(vec![FieldError::new(campo, clave)])
}

/// Adds a parameter to a single-field validation error, so the message can name the offending value.
trait ConParam {
    fn con(self, nombre: &str, valor: &str) -> AppError;
}

impl ConParam for AppError {
    fn con(self, nombre: &str, valor: &str) -> AppError {
        match self {
            AppError::Validation(mut errores) => {
                if let Some(primero) = errores.first_mut() {
                    primero.params.insert(nombre.to_owned(), valor.to_owned());
                }
                AppError::Validation(errores)
            }
            otro => otro,
        }
    }
}
