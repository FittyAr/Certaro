use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "liquidaciones")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub empleado_id: String,
    pub fecha_inicio: String,
    pub fecha_fin: String,
    /// `Decimal4`: half days and multipliers make this fractional.
    pub dias_trabajados: i64,
    /// `Money`: the rate frozen at the moment of settling.
    pub tarifa_aplicada: i64,
    pub incluir_sabados: bool,
    pub incluir_domingos: bool,
    pub incluir_feriados: bool,
    /// `Decimal4`, `10000` meaning 1.0.
    pub multiplicador_sabado: i64,
    /// `Decimal4`, `10000` meaning 1.0.
    pub multiplicador_domingo: i64,
    /// `Decimal4`, `10000` meaning 1.0.
    pub multiplicador_feriado: i64,
    /// `Money`, frozen.
    pub total_bruto: i64,
    /// `Money`, frozen. The net total is derived and deliberately not stored.
    pub total_adelantos: i64,
    pub observaciones: Option<String>,
    /// Set the first time the PDF is handed over; from then on the amounts are read-only.
    pub pdf_generado_at: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

/// Only the employee: the listing always shows their name, and the advances are read on demand.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::empleado::Entity",
        from = "Column::EmpleadoId",
        to = "super::empleado::Column::Id"
    )]
    Empleado,
}

impl ActiveModelBehavior for ActiveModel {}
