use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "kanban_tarjetas")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub columna_id: String,
    pub titulo: String,
    pub descripcion: Option<String>,
    pub prioridad: i32,
    pub fecha_vencimiento: Option<String>,
    pub orden: i32,
    pub trabajo_id: Option<String>,
    pub orden_trabajo_id: Option<String>,
    pub archivada: bool,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::kanban_columna::Entity",
        from = "Column::ColumnaId",
        to = "super::kanban_columna::Column::Id"
    )]
    Columna,
    #[sea_orm(has_many = "super::kanban_tarjeta_checklist::Entity")]
    Checklists,
}

impl Related<super::kanban_columna::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Columna.def()
    }
}

impl Related<super::kanban_tarjeta_checklist::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Checklists.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
