use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "orden_trabajo_items")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub orden_trabajo_id: String,
    pub descripcion: String,
    pub unidad: String,
    /// `Decimal4`: quantities admit fractions.
    pub cantidad: i64,
    /// `Money`, scaled by 10,000.
    pub precio_unitario: i64,
    /// `Decimal4`: percentage certified before this certificate.
    pub porcentaje_anterior: i64,
    /// `Decimal4`: percentage certified by this certificate.
    pub porcentaje_actual: i64,
    pub ejecutado: bool,
    pub nota: Option<String>,
    pub orden: i32,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
