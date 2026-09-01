use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "certificados")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub orden_trabajo_id: String,
    /// Sequential within the work order, starting at 1, and never reused.
    pub numero: i32,
    pub fecha: String,
    pub observaciones: Option<String>,
    /// `Money`, frozen when the certificate is issued.
    pub total_certificado: i64,
    /// `Money`, frozen when the certificate is issued.
    pub ajuste_uocra: i64,
    /// `Money`, frozen when the certificate is issued.
    pub otros_descuentos: i64,
    /// `Money`, frozen when the certificate is issued.
    pub total_neto: i64,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
