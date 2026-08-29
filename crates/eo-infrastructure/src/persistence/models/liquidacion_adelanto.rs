use sea_orm::entity::prelude::*;

/// Which advances were discounted in a settlement. The unique index on `movimiento_id` is what
/// keeps the same advance from being discounted twice.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "liquidacion_adelantos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub liquidacion_id: String,
    pub movimiento_id: String,
    /// `Money`, frozen so a reprinted PDF is identical.
    pub monto: i64,
    pub fecha: String,
    pub concepto: String,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
