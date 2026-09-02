use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "permisos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub modulo: String,
    pub accion: String,
    pub recurso: Option<String>,
    #[sea_orm(unique)]
    pub clave: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::rol_permiso::Entity")]
    RolPermiso,
}

impl Related<super::rol_permiso::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RolPermiso.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
