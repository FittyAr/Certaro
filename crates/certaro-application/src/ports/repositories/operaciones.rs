use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;
use certaro_domain::entities::*;
use certaro_domain::{Decimal4, EstadoFactura, EstadoProyecto, EstadoTrabajo, Moneda, Money, RowVersion};
use crate::paging::{PageRequest, PagedResult};
use crate::result::AppResult;
use super::common::*;

#[async_trait]
pub trait KanbanTableroRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<KanbanTablero>>;
    async fn list_all(&self) -> AppResult<Vec<KanbanTablero>>;
    async fn insert(&self, entity: &KanbanTablero) -> AppResult<()>;
    async fn update(&self, entity: &KanbanTablero) -> AppResult<()>;
    async fn delete(&self, id: Uuid, row_version: &RowVersion) -> AppResult<()>;
}

#[async_trait]
pub trait KanbanColumnaRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<KanbanColumna>>;
    async fn list_by_tablero(&self, tablero_id: Uuid) -> AppResult<Vec<KanbanColumna>>;
    async fn insert(&self, entity: &KanbanColumna) -> AppResult<()>;
    async fn update(&self, entity: &KanbanColumna) -> AppResult<()>;
    async fn delete(&self, id: Uuid, row_version: &RowVersion) -> AppResult<()>;
}

#[async_trait]
pub trait KanbanTarjetaRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<KanbanTarjeta>>;
    async fn list_by_tablero(&self, tablero_id: Uuid) -> AppResult<Vec<KanbanTarjeta>>;
    async fn list_by_columna(&self, columna_id: Uuid) -> AppResult<Vec<KanbanTarjeta>>;
    async fn find_by_trabajo_id(&self, trabajo_id: Uuid) -> AppResult<Option<KanbanTarjeta>>;
    async fn find_by_orden_trabajo_id(&self, orden_id: Uuid) -> AppResult<Option<KanbanTarjeta>>;
    async fn insert(&self, entity: &KanbanTarjeta) -> AppResult<()>;
    async fn update(&self, entity: &KanbanTarjeta) -> AppResult<()>;
    async fn delete(&self, id: Uuid, row_version: &RowVersion) -> AppResult<()>;
}

#[async_trait]
pub trait KanbanEtiquetaRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<KanbanEtiqueta>>;
    async fn list_all(&self) -> AppResult<Vec<KanbanEtiqueta>>;
    async fn list_by_tarjeta(&self, tarjeta_id: Uuid) -> AppResult<Vec<KanbanEtiqueta>>;
    async fn assign(&self, tarjeta_id: Uuid, etiqueta_id: Uuid) -> AppResult<()>;
    async fn unassign(&self, tarjeta_id: Uuid, etiqueta_id: Uuid) -> AppResult<()>;
    async fn insert(&self, entity: &KanbanEtiqueta) -> AppResult<()>;
    async fn update(&self, entity: &KanbanEtiqueta) -> AppResult<()>;
    async fn delete(&self, id: Uuid, row_version: &RowVersion) -> AppResult<()>;
}

#[async_trait]
pub trait KanbanChecklistRepository: Send + Sync {
    async fn list_by_tarjeta(&self, tarjeta_id: Uuid) -> AppResult<Vec<KanbanTarjetaChecklist>>;
    async fn insert(&self, entity: &KanbanTarjetaChecklist) -> AppResult<()>;
    async fn update(&self, entity: &KanbanTarjetaChecklist) -> AppResult<()>;
    async fn delete_by_id(&self, id: Uuid) -> AppResult<()>;
}

#[async_trait]
pub trait CalendarioGrupoRecursoRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<CalendarioGrupoRecurso>>;
    async fn list_all(&self) -> AppResult<Vec<CalendarioGrupoRecurso>>;
    async fn insert(&self, entity: &CalendarioGrupoRecurso) -> AppResult<()>;
    async fn update(&self, entity: &CalendarioGrupoRecurso) -> AppResult<()>;
    async fn delete(&self, id: Uuid, row_version: &RowVersion) -> AppResult<()>;
}

#[async_trait]
pub trait CalendarioRecursoRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<CalendarioRecurso>>;
    async fn find_by_empleado_id(&self, empleado_id: Uuid) -> AppResult<Option<CalendarioRecurso>>;
    async fn list_all(&self) -> AppResult<Vec<CalendarioRecurso>>;
    async fn list_activos(&self) -> AppResult<Vec<CalendarioRecurso>>;
    async fn insert(&self, entity: &CalendarioRecurso) -> AppResult<()>;
    async fn update(&self, entity: &CalendarioRecurso) -> AppResult<()>;
    async fn delete(&self, id: Uuid, row_version: &RowVersion) -> AppResult<()>;
}

#[async_trait]
pub trait CalendarioEventoRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<CalendarioEvento>>;
    async fn list_en_rango(&self, desde: DateTime<Utc>, hasta: DateTime<Utc>) -> AppResult<Vec<CalendarioEvento>>;
    async fn list_por_recurso(&self, recurso_id: Uuid, desde: DateTime<Utc>, hasta: DateTime<Utc>) -> AppResult<Vec<CalendarioEvento>>;
    async fn insert(&self, entity: &CalendarioEvento) -> AppResult<()>;
    async fn update(&self, entity: &CalendarioEvento) -> AppResult<()>;
    async fn delete(&self, id: Uuid, row_version: &RowVersion) -> AppResult<()>;
    async fn assign_recurso(&self, evento_id: Uuid, recurso_id: Uuid) -> AppResult<()>;
    async fn unassign_recursos(&self, evento_id: Uuid) -> AppResult<()>;
    async fn get_recursos_ids(&self, evento_id: Uuid) -> AppResult<Vec<Uuid>>;
}

