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
    pub async fn create_columna(&self, input: CrearColumnaInput) -> AppResult<KanbanColumnaDto> {
        let tx = self.uow.begin().await?;
        let existing = tx
            .kanban_columnas()
            .list_by_tablero(input.tableroId)
            .await?;
        let max_order = existing.iter().map(|c| c.orden).max().unwrap_or(-1);

        let now = self.clock.now_utc();
        let id = self.id_gen.new_id();

        let columna = KanbanColumna {
            id,
            tablero_id: input.tableroId,
            nombre: input.nombre,
            color: input.color,
            orden: max_order + 1,
            limite_wip: input.limiteWip,
            estado_mapeado: None,
            audit: Audit::new(now),
        };

        tx.kanban_columnas().insert(&columna).await?;
        tx.commit().await?;
        Ok(KanbanColumnaDto::from(columna))
    }

    pub async fn update_columna(
        &self,
        id: Uuid,
        input: ActualizarColumnaInput,
    ) -> AppResult<KanbanColumnaDto> {
        let tx = self.uow.begin().await?;
        let current = tx
            .kanban_columnas()
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity: "kanban_columnas",
                id: id.to_string(),
            })?;

        let expected = RowVersion::parse_hex(&input.rowVersion).map_err(|_| {
            validation_err("rowVersion", "Validation.RowVersion.Invalid")
        })?;

        if current.audit.row_version != expected {
            return Err(AppError::Concurrency {
                entity: "kanban_columnas",
            });
        }

        let now = self.clock.now_utc();
        let mut audit = current.audit;
        audit.touch(now);

        let updated = KanbanColumna {
            id,
            tablero_id: current.tablero_id,
            nombre: input.nombre,
            color: input.color,
            orden: input.orden,
            limite_wip: input.limiteWip,
            estado_mapeado: current.estado_mapeado,
            audit,
        };

        tx.kanban_columnas().update(&updated).await?;
        tx.commit().await?;
        Ok(KanbanColumnaDto::from(updated))
    }

    pub async fn delete_columna(&self, id: Uuid, row_version_hex: &str) -> AppResult<()> {
        let row_version = RowVersion::parse_hex(row_version_hex).map_err(|_| {
            validation_err("rowVersion", "Validation.RowVersion.Invalid")
        })?;
        let tx = self.uow.begin().await?;
        tx.kanban_columnas().delete(id, &row_version).await?;
        tx.commit().await?;
        Ok(())
    }


    pub async fn reordenar_columnas(&self, input: ReordenarColumnasInput) -> AppResult<()> {
        let tx = self.uow.begin().await?;
        let now = self.clock.now_utc();
        for (idx, col_id) in input.columnaIds.into_iter().enumerate() {
            if let Some(mut col) = tx.kanban_columnas().find_by_id(col_id).await? {
                if col.orden != idx as i32 {
                    col.orden = idx as i32;
                    col.audit.touch(now);
                    tx.kanban_columnas().update(&col).await?;
                }
            }
        }
        tx.commit().await?;
        Ok(())
    }

}
