use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "certificado_items")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub certificado_id: String,
    pub orden_trabajo_item_id: String,
    /// `Decimal4`, copied from the item so a later price change cannot rewrite history.
    pub cantidad: i64,
    /// `Money`, copied from the item for the same reason.
    pub precio_unitario: i64,
    /// `Decimal4`.
    pub porcentaje_anterior: i64,
    /// `Decimal4`.
    pub porcentaje_actual: i64,
    /// `Money`, frozen.
    pub subtotal_actual: i64,
    /// `Money`, frozen.
    pub subtotal_acumulado: i64,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
