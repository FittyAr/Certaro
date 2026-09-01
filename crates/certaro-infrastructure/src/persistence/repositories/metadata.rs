//! The internal key/value store. See `docs/03-modelo-de-datos.md` §3.17.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use certaro_application::ports::repositories::MetadataRepository;
use certaro_application::{AppError, AppResult};
use certaro_domain::time;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue::Set, DatabaseTransaction, EntityTrait};

use crate::persistence::models::app_metadata::{ActiveModel, Column, Entity};

pub struct SeaOrmMetadataRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmMetadataRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[async_trait]
impl MetadataRepository for SeaOrmMetadataRepository {
    async fn get(&self, key: &str) -> AppResult<Option<(String, DateTime<Utc>)>> {
        let found = Entity::find_by_id(key.to_owned())
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;

        found
            .map(|row| {
                let escrito = time::from_storage(&row.updated_at).map_err(AppError::from)?;
                Ok((row.value, escrito))
            })
            .transpose()
    }

    async fn set(&self, key: &str, value: &str, at: DateTime<Utc>) -> AppResult<()> {
        let stored = time::to_storage(at);
        let entity = ActiveModel {
            key: Set(key.to_owned()),
            value: Set(value.to_owned()),
            updated_at: Set(stored.clone()),
        };

        Entity::insert(entity)
            .on_conflict(
                OnConflict::column(Column::Key)
                    .update_columns([Column::Value, Column::UpdatedAt])
                    .to_owned(),
            )
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }
}
