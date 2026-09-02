use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "calendario_eventos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub titulo: String,
    pub descripcion: Option<String>,
    pub tipo: String,
    pub inicio: String,
    pub fin: String,
    pub todo_el_dia: bool,
    pub color: Option<String>,
    pub trabajo_id: Option<String>,
    pub kanban_tarjeta_id: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::trabajo::Entity",
        from = "Column::TrabajoId",
        to = "super::trabajo::Column::Id"
    )]
    Trabajo,
    #[sea_orm(
        belongs_to = "super::kanban_tarjeta::Entity",
        from = "Column::KanbanTarjetaId",
        to = "super::kanban_tarjeta::Column::Id"
    )]
    KanbanTarjeta,
    #[sea_orm(has_many = "super::calendario_evento_recurso::Entity")]
    EventoRecursos,
}

impl Related<super::trabajo::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Trabajo.def()
    }
}

impl Related<super::kanban_tarjeta::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::KanbanTarjeta.def()
    }
}

impl Related<super::calendario_evento_recurso::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EventoRecursos.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
