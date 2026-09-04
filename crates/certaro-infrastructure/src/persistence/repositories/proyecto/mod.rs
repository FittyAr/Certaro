use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use certaro_application::ports::repositories::{
    ProyectoConResumen, ProyectoFiltro, ProyectoRepository, SortDir,
};
use certaro_application::{AppError, AppResult, PageRequest, PagedResult};
use certaro_domain::entities::{Proyecto, Trabajo};
use certaro_domain::RowVersion;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::persistence::mappers::proyecto as mapper;
use crate::persistence::models::proyecto::{Column, Entity};

mod mutation;
mod query;

pub struct SeaOrmProyectoRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmProyectoRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[async_trait]
impl ProyectoRepository for SeaOrmProyectoRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Proyecto>> {
        let found = Entity::find_by_id(id.to_string())
            .filter(query::alive())
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(mapper::to_domain).transpose()
    }

    async fn find_detalle(&self, id: Uuid) -> AppResult<Option<ProyectoConResumen>> {
        let found = query::base_query()
            .filter(query::alive())
            .filter(Column::Id.eq(id.to_string()))
            .into_model::<query::RowConResumen>()
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(ProyectoConResumen::try_from).transpose()
    }

    async fn search(
        &self,
        filtro: &ProyectoFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<ProyectoConResumen>> {
        query::search_proyectos(self.conn(), filtro, page, sort_by, sort_dir).await
    }

    async fn lookup(
        &self,
        cliente_id: Option<Uuid>,
        texto: Option<&str>,
        limite: u64,
    ) -> AppResult<Vec<Proyecto>> {
        query::lookup_proyectos(self.conn(), cliente_id, texto, limite).await
    }

    async fn numero_ocupado(&self, numero: i32, excluir: Option<Uuid>) -> AppResult<bool> {
        mutation::numero_ocupado(self.conn(), numero, excluir).await
    }

    async fn siguiente_numero(&self) -> AppResult<i32> {
        mutation::siguiente_numero(self.conn()).await
    }

    async fn insert(&self, entity: &Proyecto) -> AppResult<()> {
        mutation::insert_proyecto(self.conn(), entity).await
    }

    async fn update(&self, entity: &Proyecto, esperado: RowVersion) -> AppResult<()> {
        mutation::update_proyecto(self.conn(), entity, esperado).await
    }

    async fn soft_delete(
        &self,
        id: Uuid,
        esperado: RowVersion,
        at: DateTime<Utc>,
    ) -> AppResult<()> {
        mutation::soft_delete_proyecto(self.conn(), id, esperado, at).await
    }

    async fn count_trabajos(&self, id: Uuid) -> AppResult<u64> {
        mutation::count_trabajos(self.conn(), id).await
    }

    async fn trabajos_abiertos(&self, id: Uuid) -> AppResult<Vec<Trabajo>> {
        mutation::trabajos_abiertos(self.conn(), id).await
    }
}
