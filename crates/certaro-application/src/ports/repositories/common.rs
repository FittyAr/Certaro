use serde::{Deserialize, Serialize};

/// How a list is sorted. The set of accepted `field` values is validated per module, because an
/// arbitrary column name coming from the frontend would be an injection vector.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SortDir {
    #[default]
    Asc,
    Desc,
}


/// The tables a movement can point at. A closed list rather than a table name as a string,
/// because that string would end up interpolated into SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenciaTabla {
    TipoMovimiento,
    TipoConceptoPago,
    Categoria,
    Cliente,
    Trabajo,
    Empleado,
    Factura,
}

impl ReferenciaTabla {
    /// Field of the input the check belongs to, for the resulting error.
    pub const fn campo(self) -> &'static str {
        match self {
            Self::TipoMovimiento => "tipoMovimientoId",
            Self::TipoConceptoPago => "tipoConceptoPagoId",
            Self::Categoria => "categoriaId",
            Self::Cliente => "clienteId",
            Self::Trabajo => "trabajoId",
            Self::Empleado => "empleadoId",
            Self::Factura => "facturaId",
        }
    }

    pub const fn entidad(self) -> &'static str {
        match self {
            Self::TipoMovimiento => "TipoMovimiento",
            Self::TipoConceptoPago => "TipoConceptoPago",
            Self::Categoria => "Categoria",
            Self::Cliente => "Cliente",
            Self::Trabajo => "Trabajo",
            Self::Empleado => "Empleado",
            Self::Factura => "Factura",
        }
    }
}
