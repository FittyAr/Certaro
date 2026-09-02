use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "calendario_recursos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub grupo_id: Option<String>,
    pub nombre: String,
    pub tipo: String,
    pub empleado_id: Option<String>,
    pub color: Option<String>,
    pub activo: bool,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::calendario_grupo_recurso::Entity",
        from = "Column::GrupoId",
        to = "super::calendario_grupo_recurso::Column::Id"
    )]
    Grupo,
    #[sea_orm(
        belongs_to = "super::empleado::Entity",
        from = "Column::EmpleadoId",
        to = "super::empleado::Column::Id"
    )]
    Empleado,
}

impl Related<super::calendario_grupo_recurso::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Grupo.def()
    }
}

impl Related<super::empleado::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Empleado.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
