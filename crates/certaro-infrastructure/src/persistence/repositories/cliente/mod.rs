use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use certaro_application::ports::repositories::{
    ClienteConResumen, ClienteFiltro, ClienteRepository, SortDir,
};
use certaro_application::{AppError, AppResult, PageRequest, PagedResult};
use certaro_domain::entities::{Cliente, ClienteContacto};
use certaro_domain::RowVersion;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use crate::persistence::mappers::cliente as mapper;
use crate::persistence::models::cliente::Entity;
use crate::persistence::models::cliente_contacto;

mod mutation;
mod query;

pub struct SeaOrmClienteRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmClienteRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }

    async fn contactos_de(&self, cliente_id: Uuid) -> AppResult<Vec<ClienteContacto>> {
        let rows = cliente_contacto::Entity::find()
            .filter(cliente_contacto::Column::ClienteId.eq(cliente_id.to_string()))
            .filter(cliente_contacto::Column::IsDeleted.eq(false))
            .order_by_desc(cliente_contacto::Column::EsPrincipal)
            .order_by_asc(cliente_contacto::Column::Etiqueta)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::contacto_to_domain).collect()
    }
}

#[async_trait]
impl ClienteRepository for SeaOrmClienteRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Cliente>> {
        let found = Entity::find_by_id(id.to_string())
            .filter(query::alive())
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(mapper::to_domain).transpose()
    }

    async fn find_con_contactos(&self, id: Uuid) -> AppResult<Option<Cliente>> {
        let Some(mut cliente) = self.find_by_id(id).await? else {
            return Ok(None);
        };
        cliente.contactos = self.contactos_de(id).await?;
        Ok(Some(cliente))
    }

    async fn search(
        &self,
        filtro: &ClienteFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<ClienteConResumen>> {
        query::search_clientes(self.conn(), filtro, page, sort_by, sort_dir).await
    }

    async fn lookup(&self, texto: Option<&str>, limite: u64) -> AppResult<Vec<Cliente>> {
        query::lookup_clientes(self.conn(), texto, limite).await
    }

    async fn insert(&self, entity: &Cliente) -> AppResult<()> {
        mutation::insert_cliente(self.conn(), entity).await
    }

    async fn update(&self, entity: &Cliente, esperado: RowVersion) -> AppResult<()> {
        mutation::update_cliente(self.conn(), entity, esperado).await
    }

    async fn soft_delete(
        &self,
        id: Uuid,
        esperado: RowVersion,
        at: DateTime<Utc>,
    ) -> AppResult<()> {
        mutation::soft_delete_cliente(self.conn(), id, esperado, at).await
    }

    async fn insert_contacto(&self, entity: &ClienteContacto) -> AppResult<()> {
        mutation::insert_contacto(self.conn(), entity).await
    }

    async fn update_contacto(&self, entity: &ClienteContacto) -> AppResult<()> {
        mutation::update_contacto(self.conn(), entity).await
    }

    async fn soft_delete_contactos_excepto(
        &self,
        cliente_id: Uuid,
        conservar: &[Uuid],
        at: DateTime<Utc>,
    ) -> AppResult<()> {
        mutation::soft_delete_contactos_excepto(self.conn(), cliente_id, conservar, at).await
    }

    async fn count_proyectos(&self, id: Uuid) -> AppResult<u64> {
        query::count_proyectos(self.conn(), id).await
    }

    async fn count_facturas(&self, id: Uuid) -> AppResult<u64> {
        query::count_facturas(self.conn(), id).await
    }

    async fn count_movimientos(&self, id: Uuid) -> AppResult<u64> {
        query::count_movimientos(self.conn(), id).await
    }
}
