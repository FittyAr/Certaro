use std::sync::Arc;
use uuid::Uuid;

use certaro_domain::entities::audit::Audit;
use certaro_domain::entities::kanban::{
    KanbanColumna, KanbanEtiqueta, KanbanTablero, KanbanTarjeta, KanbanTarjetaChecklist,
    PrioridadTarjeta, TipoPresetTablero,
};
use certaro_domain::{EstadoTrabajo, RowVersion};

use crate::dtos::kanban::{
    ActualizarChecklistInput, ActualizarColumnaInput, ActualizarEtiquetaInput,
    ActualizarTableroInput, ActualizarTarjetaInput, CrearChecklistInput, CrearColumnaInput,
    CrearEtiquetaInput, CrearTableroInput, CrearTarjetaInput, KanbanChecklistDto, KanbanColumnaDto,
    KanbanEtiquetaDto, KanbanTableroDetalleDto, KanbanTableroDto, KanbanTarjetaDto,
    MoverTarjetaInput,
};
use crate::error::FieldError;
use crate::ports::clock::ClockPort;
use crate::ports::id_generator::IdGeneratorPort;
use crate::ports::repositories::UnitOfWork;
use crate::{AppError, AppResult};

fn validation_err(field: &'static str, message_key: &'static str) -> AppError {
    AppError::Validation(vec![FieldError::new(field, message_key)])
}

pub struct KanbanService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    id_gen: Arc<dyn IdGeneratorPort>,
}

impl KanbanService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        id_gen: Arc<dyn IdGeneratorPort>,
    ) -> Self {
        Self {
            uow,
            clock,
            id_gen,
        }
    }

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

        let now = self.clock.now_utc();
        let mut audit = current.audit;
        audit.row_version = RowVersion::parse_hex(&input.rowVersion).map_err(|_| {
            validation_err("rowVersion", "Validation.RowVersion.Invalid")
        })?;
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

        let now = self.clock.now_utc();
        let mut audit = current.audit;
        audit.row_version = RowVersion::parse_hex(&input.rowVersion).map_err(|_| {
            validation_err("rowVersion", "Validation.RowVersion.Invalid")
        })?;
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

        let now = self.clock.now_utc();
        let mut audit = current.audit;
        audit.row_version = RowVersion::parse_hex(&input.rowVersion).map_err(|_| {
            validation_err("rowVersion", "Validation.RowVersion.Invalid")
        })?;
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

        let mut updated = current.clone();
        let now = self.clock.now_utc();
        let mut audit = updated.audit;
        audit.row_version = RowVersion::parse_hex(&input.rowVersion).map_err(|_| {
            validation_err("rowVersion", "Validation.RowVersion.Invalid")
        })?;
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

    pub async fn delete_tarjeta(&self, id: Uuid, row_version_hex: &str) -> AppResult<()> {
        let row_version = RowVersion::parse_hex(row_version_hex).map_err(|_| {
            validation_err("rowVersion", "Validation.RowVersion.Invalid")
        })?;
        let tx = self.uow.begin().await?;
        tx.kanban_tarjetas().delete(id, &row_version).await?;
        tx.commit().await?;
        Ok(())
    }

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

        let now = self.clock.now_utc();
        let mut audit = current.audit;
        audit.row_version = RowVersion::parse_hex(&input.rowVersion).map_err(|_| {
            validation_err("rowVersion", "Validation.RowVersion.Invalid")
        })?;
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
