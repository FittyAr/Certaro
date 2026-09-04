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
    pub async fn sincronizar_preset(&self, tablero_id: Uuid) -> AppResult<()> {
        let tx = self.uow.begin().await?;
        let tablero = match tx.kanban_tableros().find_by_id(tablero_id).await? {
            Some(t) if t.es_preset => t,
            _ => return Ok(()),
        };

        let columnas = tx.kanban_columnas().list_by_tablero(tablero_id).await?;
        let now = self.clock.now_utc();

        match tablero.tipo_preset {
            Some(TipoPresetTablero::Trabajos) => {
                let trabajos = tx.trabajos().lookup(None, None, 10000).await?;
                for trabajo in trabajos {
                    let target_col = columnas
                        .iter()
                        .find(|c| c.estado_mapeado == Some(trabajo.estado.as_i32()))
                        .or_else(|| columnas.first());

                    if let Some(col) = target_col {
                        if let Some(mut tarjeta) = tx.kanban_tarjetas().find_by_trabajo_id(trabajo.id).await? {
                            if tarjeta.columna_id != col.id || tarjeta.titulo != trabajo.descripcion {
                                tarjeta.columna_id = col.id;
                                tarjeta.titulo = trabajo.descripcion;
                                tarjeta.audit.touch(now);
                                tx.kanban_tarjetas().update(&tarjeta).await?;
                            }
                        } else {
                            let new_tarjeta = KanbanTarjeta {
                                id: self.id_gen.new_id(),
                                columna_id: col.id,
                                titulo: trabajo.descripcion,
                                descripcion: Some(format!("Presupuesto: {}", trabajo.presupuesto)),
                                prioridad: PrioridadTarjeta::Normal,
                                fecha_vencimiento: trabajo.fecha_fin,
                                orden: 0,
                                trabajo_id: Some(trabajo.id),
                                orden_trabajo_id: None,
                                archivada: false,
                                audit: Audit::new(now),
                            };
                            tx.kanban_tarjetas().insert(&new_tarjeta).await?;
                        }
                    }
                }
            }
            Some(TipoPresetTablero::Ordenes) => {
                let ordenes = tx.ordenes_trabajo().lookup(None, None, 10000).await?;
                for orden in ordenes {
                    let default_col = columnas.first();
                    if let Some(col) = default_col {
                        if let Some(mut tarjeta) = tx.kanban_tarjetas().find_by_orden_trabajo_id(orden.id).await? {
                            if tarjeta.titulo != orden.titulo {
                                tarjeta.titulo = orden.titulo;
                                tarjeta.audit.touch(now);
                                tx.kanban_tarjetas().update(&tarjeta).await?;
                            }
                        } else {
                            let new_tarjeta = KanbanTarjeta {
                                id: self.id_gen.new_id(),
                                columna_id: col.id,
                                titulo: orden.titulo,
                                descripcion: orden.observaciones,
                                prioridad: PrioridadTarjeta::Normal,
                                fecha_vencimiento: None,
                                orden: 0,
                                trabajo_id: None,
                                orden_trabajo_id: Some(orden.id),
                                archivada: false,
                                audit: Audit::new(now),
                            };
                            tx.kanban_tarjetas().insert(&new_tarjeta).await?;
                        }
                    }
                }
            }
            None => {}
        }

        tx.commit().await?;
        Ok(())
    }

    // --- Etiquetas ---

}
