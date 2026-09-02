use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "trabajos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub proyecto_id: String,
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

/// The site a job hangs off. Declared so a listing can resolve the site's name in the same query:
/// a movement is charged to a site through its job, never directly (doc 06 §7.1).
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::proyecto::Entity",
        from = "Column::ProyectoId",
        to = "super::proyecto::Column::Id"
    )]
    Proyecto,
}

impl ActiveModelBehavior for ActiveModel {}
