//! Closed enumerations of the domain. See `docs/05-dominio-entidades.md` §3.
//!
//! Each one persists as the integer of the document and is transported to the frontend as its
//! name, so a value read from the database keeps its meaning while the contract stays legible.

use serde::{Deserialize, Serialize};

use crate::error::DomainError;

/// `docs/05-dominio-entidades.md` §3.6.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Moneda {
    #[default]
    Ars,
    Usd,
}

impl Moneda {
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::Ars => 0,
            Self::Usd => 1,
        }
    }

    pub fn from_i32(value: i32) -> Result<Self, DomainError> {
        match value {
            0 => Ok(Self::Ars),
            1 => Ok(Self::Usd),
            other => Err(DomainError::UnknownEnumValue {
                enum_name: "Moneda",
                value: other,
            }),
        }
    }

    pub const fn iso(self) -> &'static str {
        match self {
            Self::Ars => "ARS",
            Self::Usd => "USD",
        }
    }

    /// A foreign-currency amount is meaningless without the rate it was booked at, so the two
    /// travel together or not at all.
    pub const fn requiere_cotizacion(self) -> bool {
        matches!(self, Self::Usd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_moneda_por_defecto_es_el_peso() {
        assert_eq!(Moneda::default(), Moneda::Ars);
    }

    #[test]
    fn ida_y_vuelta_por_el_entero_persistido() {
        for moneda in [Moneda::Ars, Moneda::Usd] {
            assert_eq!(Moneda::from_i32(moneda.as_i32()).unwrap(), moneda);
        }
    }

    #[test]
    fn un_valor_desconocido_no_se_adivina() {
        // A row with a value outside the enum is corrupt data, and silently mapping it to `Ars`
        // would turn dollars into pesos.
        assert!(Moneda::from_i32(7).is_err());
    }

    #[test]
    fn solo_el_dolar_exige_cotizacion() {
        assert!(Moneda::Usd.requiere_cotizacion());
        assert!(!Moneda::Ars.requiere_cotizacion());
    }
}
