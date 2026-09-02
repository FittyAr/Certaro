use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "calendario_evento_recursos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub evento_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub recurso_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::calendario_evento::Entity",
        from = "Column::EventoId",
        to = "super::calendario_evento::Column::Id"
    )]
    Evento,
    #[sea_orm(
        belongs_to = "super::calendario_recurso::Entity",
        from = "Column::RecursoId",
        to = "super::calendario_recurso::Column::Id"
    )]
    Recurso,
}

impl Related<super::calendario_evento::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Evento.def()
    }
}

impl Related<super::calendario_recurso::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Recurso.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
