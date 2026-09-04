use uuid::Uuid;
use certaro_domain::entities::audit::Audit;
use certaro_domain::entities::kanban::{
    KanbanColumna, KanbanEtiqueta, KanbanTablero, KanbanTarjeta, KanbanTarjetaChecklist,
    PrioridadTarjeta, TipoPresetTablero,
};
use certaro_domain::{EstadoTrabajo, RowVersion};

use crate::dtos::kanban::*;
use crate::{AppError, AppResult};
use super::{validation_err, KanbanService};

impl KanbanService {
    pub async fn list_etiquetas(&self) -> AppResult<Vec<KanbanEtiquetaDto>> {
        let tx = self.uow.begin().await?;
        let tags = tx.kanban_etiquetas().list_all().await?;
        Ok(tags.into_iter().map(KanbanEtiquetaDto::from).collect())
    }

    pub async fn create_etiqueta(&self, input: CrearEtiquetaInput) -> AppResult<KanbanEtiquetaDto> {
        let tx = self.uow.begin().await?;
        let now = self.clock.now_utc();
        let id = self.id_gen.new_id();

        let etiqueta = KanbanEtiqueta {
            id,
            nombre: input.nombre,
            color: input.color,
            audit: Audit::new(now),
        };

        tx.kanban_etiquetas().insert(&etiqueta).await?;
        tx.commit().await?;
        Ok(KanbanEtiquetaDto::from(etiqueta))
    }

    pub async fn update_etiqueta(
        &self,
        id: Uuid,
        input: ActualizarEtiquetaInput,
    ) -> AppResult<KanbanEtiquetaDto> {
        let tx = self.uow.begin().await?;
        let current = tx
            .kanban_etiquetas()
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity: "kanban_etiquetas",
                id: id.to_string(),
            })?;

        let expected = RowVersion::parse_hex(&input.rowVersion).map_err(|_| {
            validation_err("rowVersion", "Validation.RowVersion.Invalid")
        })?;

        if current.audit.row_version != expected {
            return Err(AppError::Concurrency {
                entity: "kanban_etiquetas",
            });
        }

        let now = self.clock.now_utc();
        let mut audit = current.audit;
        audit.touch(now);

        let updated = KanbanEtiqueta {
            id,
            nombre: input.nombre,
            color: input.color,
            audit,
        };

        tx.kanban_etiquetas().update(&updated).await?;
        tx.commit().await?;
        Ok(KanbanEtiquetaDto::from(updated))
    }

    pub async fn delete_etiqueta(&self, id: Uuid, row_version_hex: &str) -> AppResult<()> {
        let row_version = RowVersion::parse_hex(row_version_hex).map_err(|_| {
            validation_err("rowVersion", "Validation.RowVersion.Invalid")
        })?;
        let tx = self.uow.begin().await?;
        tx.kanban_etiquetas().delete(id, &row_version).await?;
        tx.commit().await?;
        Ok(())
    }

    // --- Checklist ---

    pub async fn list_checklist(&self, tarjeta_id: Uuid) -> AppResult<Vec<KanbanChecklistDto>> {
        let tx = self.uow.begin().await?;
        let items = tx.kanban_checklists().list_by_tarjeta(tarjeta_id).await?;
        Ok(items.into_iter().map(KanbanChecklistDto::from).collect())
    }

    pub async fn add_checklist_item(
        &self,
        input: CrearChecklistInput,
    ) -> AppResult<KanbanChecklistDto> {
        let tx = self.uow.begin().await?;
        let existing = tx.kanban_checklists().list_by_tarjeta(input.tarjetaId).await?;
        let max_order = existing.iter().map(|c| c.orden).max().unwrap_or(-1);

        let now = self.clock.now_utc();
        let id = self.id_gen.new_id();

        let item = KanbanTarjetaChecklist {
            id,
            tarjeta_id: input.tarjetaId,
            titulo: input.titulo,
            completada: false,
            orden: max_order + 1,
            audit: Audit::new(now),
        };

        tx.kanban_checklists().insert(&item).await?;
        tx.commit().await?;
        Ok(KanbanChecklistDto::from(item))
    }

    pub async fn update_checklist_item(
        &self,
        id: Uuid,
        input: ActualizarChecklistInput,
    ) -> AppResult<KanbanChecklistDto> {
        let tx = self.uow.begin().await?;
        let now = self.clock.now_utc();
        let row_version = RowVersion::parse_hex(&input.rowVersion).map_err(|_| {
            validation_err("rowVersion", "Validation.RowVersion.Invalid")
        })?;

        let mut audit = Audit::new(now);
        audit.row_version = row_version;
        audit.touch(now);

        let updated = KanbanTarjetaChecklist {
            id,
            tarjeta_id: self.id_gen.new_id(), // mapper won't touch if update only sets fields
            titulo: input.titulo,
            completada: input.completada,
            orden: input.orden,
            audit,
        };

        tx.kanban_checklists().update(&updated).await?;
        tx.commit().await?;
        Ok(KanbanChecklistDto::from(updated))
    }

    pub async fn delete_checklist_item(&self, id: Uuid) -> AppResult<()> {
        let tx = self.uow.begin().await?;
        tx.kanban_checklists().delete_by_id(id).await?;
        tx.commit().await?;
        Ok(())
    }
}
