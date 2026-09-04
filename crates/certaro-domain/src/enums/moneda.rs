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

/// `docs/05-dominio-entidades.md` §3.8.
///
/// Only fills the dropdown: the column stays `TEXT`, and a historical value outside this list is
/// shown as it was written rather than being normalised into one of these.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum MedioPago {
    #[default]
    Efectivo,
    Transferencia,
    Cheque,
    Deposito,
    Otro,
}

impl MedioPago {
    pub const ALL: [Self; 5] = [
        Self::Efectivo,
        Self::Transferencia,
        Self::Cheque,
        Self::Deposito,
        Self::Otro,
    ];

    /// What gets stored, accents included: it is the text the legacy rows already carry.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Efectivo => "Efectivo",
            Self::Transferencia => "Transferencia",
            Self::Cheque => "Cheque",
            Self::Deposito => "Depósito",
            Self::Otro => "Otro",
        }
    }
}
