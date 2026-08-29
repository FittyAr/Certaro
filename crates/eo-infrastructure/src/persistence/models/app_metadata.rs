use sea_orm::entity::prelude::*;

/// Internal key/value store. No audit block and no soft delete: it is not a business record.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "app_metadata")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
