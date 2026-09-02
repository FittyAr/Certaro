use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "rol_permisos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub rol_id: String,
    pub permiso_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::rol::Entity",
        from = "Column::RolId",
        to = "super::rol::Column::Id"
    )]
    Rol,
    #[sea_orm(
        belongs_to = "super::permiso::Entity",
        from = "Column::PermisoId",
        to = "super::permiso::Column::Id"
    )]
    Permiso,
}

impl Related<super::rol::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rol.def()
    }
}

impl Related<super::permiso::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Permiso.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
