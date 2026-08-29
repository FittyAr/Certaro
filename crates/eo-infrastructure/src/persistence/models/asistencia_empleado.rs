use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "asistencias_empleado")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub empleado_id: String,
    /// A civil date normalised to UTC midnight; the unique index depends on that normalisation.
    pub fecha: String,
    /// `TipoJornada` as its numeric value.
    pub tipo_jornada: i32,
    pub trabajo_id: Option<String>,
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
