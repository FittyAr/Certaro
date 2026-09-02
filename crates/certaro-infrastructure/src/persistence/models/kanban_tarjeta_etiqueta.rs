use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "kanban_tarjeta_etiquetas")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub tarjeta_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub etiqueta_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::kanban_tarjeta::Entity",
        from = "Column::TarjetaId",
        to = "super::kanban_tarjeta::Column::Id"
    )]
    Tarjeta,
    #[sea_orm(
        belongs_to = "super::kanban_etiqueta::Entity",
        from = "Column::EtiquetaId",
        to = "super::kanban_etiqueta::Column::Id"
    )]
    Etiqueta,
}

impl Related<super::kanban_tarjeta::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tarjeta.def()
    }
}

impl Related<super::kanban_etiqueta::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Etiqueta.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
