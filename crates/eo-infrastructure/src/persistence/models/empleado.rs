use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "empleados")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub nombre: String,
    pub dni: Option<String>,
    pub cargo: Option<String>,
    /// `Money`: informative, only used to suggest the daily rate.
    pub sueldo_base: i64,
    /// `FrecuenciaPago` as its numeric value.
    pub pago_frecuencia: i32,
    /// `Money`: the value of one working day, which is what the payroll uses.
    pub tarifa_diaria: i64,
    /// `Decimal4`, `10000` meaning 1.0. Per employee: a foreman and a helper are not paid the same
    /// weekend, and configuration only provides the default for a new card.
    pub multiplicador_sabado: i64,
    /// `Decimal4`, `10000` meaning 1.0.
    pub multiplicador_domingo: i64,
    /// `Decimal4`, `10000` meaning 1.0.
    pub multiplicador_feriado: i64,
    pub email: Option<String>,
    pub telefono: Option<String>,
    pub fecha_ingreso: String,
    pub fecha_egreso: Option<String>,
    pub activo: bool,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
