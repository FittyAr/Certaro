use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ordenes_trabajo")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub trabajo_id: String,
    pub titulo: String,
    pub numero_certificado: Option<String>,
    pub fecha: String,
    pub observaciones: Option<String>,
    /// `Decimal4`: a percentage, not an amount.
    pub ajuste_uocra_porcentaje: i64,
    /// `Money`, scaled by 10,000.
    pub otros_descuentos: i64,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
