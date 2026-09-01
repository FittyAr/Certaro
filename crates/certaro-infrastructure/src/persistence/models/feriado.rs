use sea_orm::entity::prelude::*;

/// A calendar, not a business record: the date is the key, there is no soft delete, and removing
/// a holiday is a real `DELETE`.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "feriados")]
pub struct Model {
    /// `YYYY-MM-DD`, a civil date.
    #[sea_orm(primary_key, auto_increment = false)]
    pub fecha: String,
    pub nombre: String,
    pub tipo: Option<String>,
    /// `Api` or `Manual`. A sync never overwrites a manual row.
    pub origen: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
