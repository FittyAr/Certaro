//! Use cases of attachments. See `docs/13-servicios-externos-y-archivos.md` §1.
//!
//! The row and the file are written together: the row inside the transaction, the file just before
//! committing it. If the commit fails the file is trashed, so the two never disagree — the legacy
//! code wrote the file first and let the row fail on its own.

use std::path::Path;
use std::sync::Arc;

use eo_domain::entities::{Adjunto, Audit, EntidadAdjunto};
use tracing::info;
use uuid::Uuid;

use crate::dtos::adjuntos::{AdjuntoInput, AdjuntoItem};
use crate::error::{AppError, FieldError};
use crate::ports::repositories::UnitOfWork;
use crate::ports::{AttachmentStore, ClockPort, IdGeneratorPort, OpenerPort, SettingsStore};
use crate::result::AppResult;
use crate::use_cases::shared::{finish_read, finish_write};

const ENTITY: &str = "Adjunto";

const MB: u64 = 1024 * 1024;

pub struct AdjuntosService {
    uow: Arc<dyn UnitOfWork>,
    store: Arc<dyn AttachmentStore>,
    opener: Arc<dyn OpenerPort>,
    clock: Arc<dyn ClockPort>,
    ids: Arc<dyn IdGeneratorPort>,
    settings: Arc<dyn SettingsStore>,
}

impl AdjuntosService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        store: Arc<dyn AttachmentStore>,
        opener: Arc<dyn OpenerPort>,
        clock: Arc<dyn ClockPort>,
        ids: Arc<dyn IdGeneratorPort>,
        settings: Arc<dyn SettingsStore>,
    ) -> Self {
        Self {
            uow,
            store,
            opener,
            clock,
            ids,
            settings,
        }
    }

    pub async fn list(
        &self,
        entidad_tipo: EntidadAdjunto,
        entidad_id: Uuid,
    ) -> AppResult<Vec<AdjuntoItem>> {
        let tx = self.uow.begin().await?;
        let cargado = tx.adjuntos().de_entidad(entidad_tipo, entidad_id).await;
        let filas = finish_read(tx, cargado).await?;
        Ok(filas.into_iter().map(AdjuntoItem::from).collect())
    }

    pub async fn add(&self, input: AdjuntoInput) -> AppResult<AdjuntoItem> {
        let origen = Path::new(&input.ruta_origen);
        let cupo = self
            .cupo_restante(input.entidad_tipo, input.entidad_id)
            .await?;
        let aceptado = self.store.accept(origen, cupo).await?;

        let id = self.ids.new_id();
        let guardado = self
            .store
            .store(origen, input.entidad_tipo, input.entidad_id, id, aceptado)
            .await?;

        let entity = Adjunto {
            id,
            entidad_tipo: input.entidad_tipo,
            entidad_id: input.entidad_id,
            nombre_archivo: guardado.archivo.nombre.clone(),
            ruta_relativa: guardado.ruta_relativa.clone(),
            mime: guardado.archivo.mime.to_owned(),
            tamano: guardado.archivo.tamano,
            audit: Audit::new(self.clock.now_utc()),
        };

        let tx = self.uow.begin().await?;
        let escrito = tx.adjuntos().insert(&entity).await;
        let resultado = finish_write(tx, escrito).await;

        if resultado.is_err() {
            // The file is already on disk, so it goes to the trash rather than being left orphaned
            // in the attachments tree where nothing would ever reference it.
            let _ = self.store.trash(&guardado.ruta_relativa).await;
            resultado?;
        }

        info!(
            %id,
            entidad = entity.entidad_tipo.as_str(),
            "adjunto agregado"
        );
        Ok(entity.into())
    }

    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        let adjunto = self.cargar(id).await?;

        let tx = self.uow.begin().await?;
        let borrado = tx.adjuntos().soft_delete(id, self.clock.now_utc()).await;
        finish_write(tx, borrado).await?;

        // Only after the row is committed: a file in the trash whose row survived would come back
        // as a broken link, which is exactly the legacy failure this reverses.
        self.store.trash(&adjunto.ruta_relativa).await?;
        info!(%id, "adjunto enviado a la papelera");
        Ok(())
    }

    pub async fn open(&self, id: Uuid) -> AppResult<()> {
        let adjunto = self.cargar(id).await?;
        let ruta = self.store.resolve(&adjunto.ruta_relativa)?;
        self.opener.open(&ruta)
    }

    pub async fn reveal(&self, id: Uuid) -> AppResult<()> {
        let adjunto = self.cargar(id).await?;
        let ruta = self.store.resolve(&adjunto.ruta_relativa)?;
        self.opener.reveal(&ruta)
    }

    async fn cargar(&self, id: Uuid) -> AppResult<eo_domain::entities::Adjunto> {
        let tx = self.uow.begin().await?;
        let cargado = tx.adjuntos().find_by_id(id).await;
        finish_read(tx, cargado)
            .await?
            .ok_or_else(|| AppError::not_found(ENTITY, id))
    }

    /// Bytes still available for this record. Read from the rows rather than from the disk so a
    /// stray file in the tree does not consume the user's quota.
    async fn cupo_restante(
        &self,
        entidad_tipo: EntidadAdjunto,
        entidad_id: Uuid,
    ) -> AppResult<u64> {
        let total = u64::from(self.settings.snapshot().attachments.max_total_mb) * MB;
        let tx = self.uow.begin().await?;
        let cargado = tx.adjuntos().de_entidad(entidad_tipo, entidad_id).await;
        let usado: u64 = finish_read(tx, cargado)
            .await?
            .iter()
            .map(|a| a.tamano)
            .sum();
        if usado >= total {
            return Err(AppError::Validation(vec![FieldError::new(
                "rutaOrigen",
                "Validation.Adjunto.CupoExcedido",
            )
            .with_param("max", self.settings.snapshot().attachments.max_total_mb)]));
        }
        Ok(total - usado)
    }
}
