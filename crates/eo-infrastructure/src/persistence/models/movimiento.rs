use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "movimientos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub fecha: String,
    pub concepto: String,
    /// `Money`, scaled by 10,000. The total is `monto * cantidad` and is never stored.
    pub monto: i64,
    /// `Decimal4`, defaulting to `10000`, that is 1.0.
    pub cantidad: i64,
    pub tipo_movimiento_id: String,
    /// `Moneda` as its numeric value.
    pub moneda: i32,
    /// `Money`: only filled in when the currency is USD.
    pub cotizacion_aplicada: Option<i64>,
    pub tipo_concepto_pago_id: Option<String>,
    /// Nullable in the schema even though new rows require it: historical rows have none.
    pub categoria_id: Option<String>,
    pub cliente_id: Option<String>,
    pub trabajo_id: Option<String>,
    pub empleado_id: Option<String>,
    pub factura_id: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

/// Only the two relations the listing joins. The rest of the foreign keys are read as plain
/// identifiers, because loading the whole graph for a table of thousands of rows is what made the
/// legacy listing slow.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tipo_movimiento::Entity",
        from = "Column::TipoMovimientoId",
        to = "super::tipo_movimiento::Column::Id"
    )]
    TipoMovimiento,
    #[sea_orm(
        belongs_to = "super::categoria::Entity",
        from = "Column::CategoriaId",
        to = "super::categoria::Column::Id"
    )]
    Categoria,
}

impl ActiveModelBehavior for ActiveModel {}
