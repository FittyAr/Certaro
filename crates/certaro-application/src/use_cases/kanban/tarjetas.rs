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
    pub async fn create_tarjeta(&self, input: CrearTarjetaInput) -> AppResult<KanbanTarjetaDto> {
        let tx = self.uow.begin().await?;
        let existing = tx.kanban_tarjetas().list_by_columna(input.columnaId).await?;
        let max_order = existing.iter().map(|t| t.orden).max().unwrap_or(-1);

        let now = self.clock.now_utc();
        let id = self.id_gen.new_id();

        let tarjeta = KanbanTarjeta {
            id,
            columna_id: input.columnaId,
            titulo: input.titulo,
            descripcion: input.descripcion,
            prioridad: input.prioridad,
            fecha_vencimiento: input.fechaVencimiento,
            orden: max_order + 1,
            trabajo_id: input.trabajoId,
            orden_trabajo_id: input.ordenTrabajoId,
            archivada: false,
            audit: Audit::new(now),
        };

        tx.kanban_tarjetas().insert(&tarjeta).await?;

        if let Some(etiqueta_ids) = input.etiquetaIds {
            for tag_id in etiqueta_ids {
                tx.kanban_etiquetas().assign(id, tag_id).await?;
            }
        }

        let tags = tx.kanban_etiquetas().list_by_tarjeta(id).await?;
        let proyecto_id = match tarjeta.trabajo_id {
            Some(trabajo_id) => tx.trabajos().find_by_id(trabajo_id).await?.map(|tr| tr.proyecto_id),
            None => None,
        };
        tx.commit().await?;

        Ok(KanbanTarjetaDto {
            id: tarjeta.id,
            columnaId: tarjeta.columna_id,
            titulo: tarjeta.titulo,
            descripcion: tarjeta.descripcion,
            prioridad: tarjeta.prioridad,
            fechaVencimiento: tarjeta.fecha_vencimiento,
            orden: tarjeta.orden,
            trabajoId: tarjeta.trabajo_id,
            ordenTrabajoId: tarjeta.orden_trabajo_id,
            proyectoId: proyecto_id,
            archivada: tarjeta.archivada,
            rowVersion: tarjeta.audit.row_version.to_hex(),
            etiquetas: tags.into_iter().map(KanbanEtiquetaDto::from).collect(),
            totalChecklist: 0,
            completadasChecklist: 0,
        })
    }

    pub async fn update_tarjeta(
        &self,
        id: Uuid,
        input: ActualizarTarjetaInput,
    ) -> AppResult<KanbanTarjetaDto> {
        let tx = self.uow.begin().await?;
        let current = tx
            .kanban_tarjetas()
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity: "kanban_tarjetas",
                id: id.to_string(),
            })?;

        let expected = RowVersion::parse_hex(&input.rowVersion).map_err(|_| {
            validation_err("rowVersion", "Validation.RowVersion.Invalid")
        })?;

        if current.audit.row_version != expected {
            return Err(AppError::Concurrency {
                entity: "kanban_tarjetas",
            });
        }

        let now = self.clock.now_utc();
        let mut audit = current.audit;
        audit.touch(now);

        let updated = KanbanTarjeta {
            id,
            columna_id: current.columna_id,
            titulo: input.titulo,
            descripcion: input.descripcion,
            prioridad: input.prioridad,
            fecha_vencimiento: input.fechaVencimiento,
            orden: current.orden,
            trabajo_id: current.trabajo_id,
            orden_trabajo_id: current.orden_trabajo_id,
            archivada: current.archivada,
            audit,
        };

        tx.kanban_tarjetas().update(&updated).await?;

        if let Some(tag_ids) = input.etiquetaIds {
            let current_tags = tx.kanban_etiquetas().list_by_tarjeta(id).await?;
            for ct in current_tags {
                tx.kanban_etiquetas().unassign(id, ct.id).await?;
            }
            for tid in tag_ids {
                tx.kanban_etiquetas().assign(id, tid).await?;
            }
        }

        let tags = tx.kanban_etiquetas().list_by_tarjeta(id).await?;
        let checklist = tx.kanban_checklists().list_by_tarjeta(id).await?;
        let proyecto_id = match updated.trabajo_id {
            Some(trabajo_id) => tx.trabajos().find_by_id(trabajo_id).await?.map(|tr| tr.proyecto_id),
            None => None,
        };
        tx.commit().await?;

        Ok(KanbanTarjetaDto {
            id: updated.id,
            columnaId: updated.columna_id,
            titulo: updated.titulo,
            descripcion: updated.descripcion,
            prioridad: updated.prioridad,
            fechaVencimiento: updated.fecha_vencimiento,
            orden: updated.orden,
            trabajoId: updated.trabajo_id,
            ordenTrabajoId: updated.orden_trabajo_id,
            proyectoId: proyecto_id,
            archivada: updated.archivada,
            rowVersion: updated.audit.row_version.to_hex(),
            etiquetas: tags.into_iter().map(KanbanEtiquetaDto::from).collect(),
            totalChecklist: checklist.len(),
            completadasChecklist: checklist.iter().filter(|c| c.completada).count(),
        })
    }

    pub async fn mover_tarjeta(&self, input: MoverTarjetaInput) -> AppResult<()> {
        let tx = self.uow.begin().await?;
        let current = tx
            .kanban_tarjetas()
            .find_by_id(input.tarjetaId)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity: "kanban_tarjetas",
                id: input.tarjetaId.to_string(),
            })?;

        let target_col = tx
            .kanban_columnas()
            .find_by_id(input.nuevaColumnaId)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity: "kanban_columnas",
                id: input.nuevaColumnaId.to_string(),
            })?;

        let expected = RowVersion::parse_hex(&input.rowVersion).map_err(|_| {
            validation_err("rowVersion", "Validation.RowVersion.Invalid")
        })?;

        if current.audit.row_version != expected {
            return Err(AppError::Concurrency {
                entity: "kanban_tarjetas",
            });
        }

        let mut updated = current.clone();
        let now = self.clock.now_utc();
        let mut audit = updated.audit;
        audit.touch(now);

        updated.columna_id = input.nuevaColumnaId;
        updated.orden = input.nuevoOrden;
        updated.audit = audit;

        tx.kanban_tarjetas().update(&updated).await?;

        // Bidirectional sync: if moving a card linked to a Trabajo or OrdenTrabajo
        if let (Some(trabajo_id), Some(estado_int)) = (current.trabajo_id, target_col.estado_mapeado) {
            if let Ok(estado) = EstadoTrabajo::from_i32(estado_int) {
                if let Some(mut trabajo) = tx.trabajos().find_by_id(trabajo_id).await? {
                    let esperado = trabajo.audit.row_version;
                    trabajo.estado = estado;
                    trabajo.audit.touch(now);
                    tx.trabajos().update(&trabajo, esperado).await?;
                }
            }
        }

        tx.commit().await?;
        Ok(())
    }


    pub async fn reordenar_tarjetas(&self, input: ReordenarTarjetasInput) -> AppResult<()> {
        let tx = self.uow.begin().await?;
        let now = self.clock.now_utc();

        let target_col = tx
            .kanban_columnas()
            .find_by_id(input.destinoColumnaId)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity: "kanban_columnas",
                id: input.destinoColumnaId.to_string(),
            })?;

        if let Some(mut card) = tx.kanban_tarjetas().find_by_id(input.tarjetaId).await? {
            let trabajo_id = card.trabajo_id;
            card.columna_id = input.destinoColumnaId;
            card.orden = input.nuevoOrden;
            card.audit.touch(now);
            tx.kanban_tarjetas().update(&card).await?;

            if let (Some(trab_id), Some(estado_int)) = (trabajo_id, target_col.estado_mapeado) {
                if let Ok(estado) = EstadoTrabajo::from_i32(estado_int) {
                    if let Some(mut trabajo) = tx.trabajos().find_by_id(trab_id).await? {
                        let esperado = trabajo.audit.row_version;
                        trabajo.estado = estado;
                        trabajo.audit.touch(now);
                        tx.trabajos().update(&trabajo, esperado).await?;
                    }
                }
            }
        }

        for (idx, card_id) in input.tarjetaIdsEnDestino.into_iter().enumerate() {
            if let Some(mut c) = tx.kanban_tarjetas().find_by_id(card_id).await? {
                if c.orden != idx as i32 {
                    c.orden = idx as i32;
                    c.audit.touch(now);
                    tx.kanban_tarjetas().update(&c).await?;
                }
            }
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_tarjeta(&self, id: Uuid, row_version_hex: &str) -> AppResult<()> {
        let row_version = RowVersion::parse_hex(row_version_hex).map_err(|_| {
            validation_err("rowVersion", "Validation.RowVersion.Invalid")
        })?;
        let tx = self.uow.begin().await?;
        tx.kanban_tarjetas().delete(id, &row_version).await?;
        tx.commit().await?;
        Ok(())
    }

}
