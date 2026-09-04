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
    pub async fn list_tableros(&self) -> AppResult<Vec<KanbanTableroDto>> {
        let tx = self.uow.begin().await?;
        let tableros = tx.kanban_tableros().list_all().await?;
        Ok(tableros.into_iter().map(KanbanTableroDto::from).collect())
    }

    pub async fn get_tablero_detalle(&self, tablero_id: Uuid) -> AppResult<KanbanTableroDetalleDto> {
        // First, if it's a preset board, trigger bidirectional sync
        let _ = self.sincronizar_preset(tablero_id).await;

        let tx = self.uow.begin().await?;
        let tablero = tx
            .kanban_tableros()
            .find_by_id(tablero_id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity: "kanban_tableros",
                id: tablero_id.to_string(),
            })?;

        let columnas = tx.kanban_columnas().list_by_tablero(tablero_id).await?;
        let tarjetas_raw = tx.kanban_tarjetas().list_by_tablero(tablero_id).await?;
        let all_etiquetas = tx.kanban_etiquetas().list_all().await?;

        let mut tarjetas = Vec::new();
        for t in tarjetas_raw {
            let tags = tx.kanban_etiquetas().list_by_tarjeta(t.id).await?;
            let checklist = tx.kanban_checklists().list_by_tarjeta(t.id).await?;
            let total_checklist = checklist.len();
            let completadas_checklist = checklist.iter().filter(|c| c.completada).count();

            let proyecto_id = match t.trabajo_id {
                Some(trabajo_id) => tx.trabajos().find_by_id(trabajo_id).await?.map(|tr| tr.proyecto_id),
                None => None,
            };

            tarjetas.push(KanbanTarjetaDto {
                id: t.id,
                columnaId: t.columna_id,
                titulo: t.titulo,
                descripcion: t.descripcion,
                prioridad: t.prioridad,
                fechaVencimiento: t.fecha_vencimiento,
                orden: t.orden,
                trabajoId: t.trabajo_id,
                ordenTrabajoId: t.orden_trabajo_id,
                proyectoId: proyecto_id,
                archivada: t.archivada,
                rowVersion: t.audit.row_version.to_hex(),
                etiquetas: tags.into_iter().map(KanbanEtiquetaDto::from).collect(),
                totalChecklist: total_checklist,
                completadasChecklist: completadas_checklist,
            });
        }

        Ok(KanbanTableroDetalleDto {
            tablero: KanbanTableroDto::from(tablero),
            columnas: columnas.into_iter().map(KanbanColumnaDto::from).collect(),
            tarjetas,
            etiquetas: all_etiquetas
                .into_iter()
                .map(KanbanEtiquetaDto::from)
                .collect(),
        })
    }

    pub async fn create_tablero(&self, input: CrearTableroInput) -> AppResult<KanbanTableroDto> {
        let tx = self.uow.begin().await?;
        let now = self.clock.now_utc();
        let id = self.id_gen.new_id();

        let tablero = KanbanTablero {
            id,
            nombre: input.nombre,
            descripcion: input.descripcion,
            color: input.color,
            es_preset: false,
            tipo_preset: None,
            activo: true,
            audit: Audit::new(now),
        };

        tx.kanban_tableros().insert(&tablero).await?;

        // Create default standard columns
        let cols = [
            ("Por Hacer", "#64748b", 0),
            ("En Progreso", "#3b82f6", 1),
            ("Listo", "#10b981", 2),
        ];

        for (nombre, color, orden) in cols {
            let col = KanbanColumna {
                id: self.id_gen.new_id(),
                tablero_id: id,
                nombre: nombre.to_string(),
                color: Some(color.to_string()),
                orden,
                limite_wip: None,
                estado_mapeado: None,
                audit: Audit::new(now),
            };
            tx.kanban_columnas().insert(&col).await?;
        }

        tx.commit().await?;
        Ok(KanbanTableroDto::from(tablero))
    }

    pub async fn update_tablero(
        &self,
        id: Uuid,
        input: ActualizarTableroInput,
    ) -> AppResult<KanbanTableroDto> {
        let tx = self.uow.begin().await?;
        let current = tx
            .kanban_tableros()
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity: "kanban_tableros",
                id: id.to_string(),
            })?;

        let expected = RowVersion::parse_hex(&input.rowVersion).map_err(|_| {
            validation_err("rowVersion", "Validation.RowVersion.Invalid")
        })?;

        if current.audit.row_version != expected {
            return Err(AppError::Concurrency {
                entity: "kanban_tableros",
            });
        }

        let now = self.clock.now_utc();
        let mut audit = current.audit;
        audit.touch(now);

        let updated = KanbanTablero {
            id,
            nombre: input.nombre,
            descripcion: input.descripcion,
            color: input.color,
            es_preset: current.es_preset,
            tipo_preset: current.tipo_preset,
            activo: input.activo,
            audit,
        };

        tx.kanban_tableros().update(&updated).await?;
        tx.commit().await?;
        Ok(KanbanTableroDto::from(updated))
    }

    pub async fn delete_tablero(&self, id: Uuid, row_version_hex: &str) -> AppResult<()> {
        let row_version = RowVersion::parse_hex(row_version_hex).map_err(|_| {
            validation_err("rowVersion", "Validation.RowVersion.Invalid")
        })?;
        let tx = self.uow.begin().await?;
        let current = tx
            .kanban_tableros()
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity: "kanban_tableros",
                id: id.to_string(),
            })?;

        if current.es_preset {
            return Err(validation_err(
                "id",
                "Validation.Kanban.CannotDeletePreset",
            ));
        }

        tx.kanban_tableros().delete(id, &row_version).await?;
        tx.commit().await?;
        Ok(())
    }

}
