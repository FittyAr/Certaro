use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "facturas")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub numero: String,
    pub fecha: String,
    pub fecha_vencimiento: Option<String>,
    pub cliente_id: String,
    /// `EstadoFactura` as its numeric value.
    pub estado: i32,
    /// `Money`, scaled by 10,000.
    pub subtotal: i64,
    /// `Money`: an amount typed by the user, not a percentage and not derived.
    pub iva: i64,
    /// `Money`, scaled by 10,000.
    pub total: i64,
    pub observaciones: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
