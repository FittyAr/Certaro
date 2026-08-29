use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "trabajos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub obra_id: String,
    pub descripcion: String,
    pub fecha_inicio: String,
    pub fecha_fin: Option<String>,
    /// `Money`, scaled by 10,000.
    pub presupuesto: i64,
    /// `EstadoTrabajo` as its numeric value.
    pub estado: i32,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
