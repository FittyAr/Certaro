use serde::{Deserialize, Serialize};
use crate::decimal4::Decimal4;
use crate::error::DomainError;

/// `docs/05-dominio-entidades.md` §3.7.
///
/// The factor is what the payroll multiplies the daily rate by. An absence pays nothing whether it
/// is justified or not: justification is a human matter, not a monetary one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TipoJornada {
    #[default]
    Completa,
    Media,
    Falta,
    FaltaJustificada,
    Feriado,
}

impl TipoJornada {
    pub const ALL: [Self; 5] = [
        Self::Completa,
        Self::Media,
        Self::Falta,
        Self::FaltaJustificada,
        Self::Feriado,
    ];

    pub const fn as_i32(self) -> i32 {
        match self {
            Self::Completa => 0,
            Self::Media => 1,
            Self::Falta => 2,
            Self::FaltaJustificada => 3,
            Self::Feriado => 4,
        }
    }

    pub fn from_i32(value: i32) -> Result<Self, DomainError> {
        match value {
            0 => Ok(Self::Completa),
            1 => Ok(Self::Media),
            2 => Ok(Self::Falta),
            3 => Ok(Self::FaltaJustificada),
            4 => Ok(Self::Feriado),
            other => Err(DomainError::UnknownEnumValue {
                enum_name: "TipoJornada",
                value: other,
            }),
        }
    }

    /// Share of a day worked: `1.0`, `0.5` or nothing.
    pub const fn factor(self) -> Decimal4 {
        match self {
            Self::Completa | Self::Feriado => Decimal4::ONE,
            Self::Media => Decimal4::HALF,
            Self::Falta | Self::FaltaJustificada => Decimal4::ZERO,
        }
    }

    /// The click cycle of the attendance grid, where `None` means no record at all.
    /// See `docs/09-modulos-funcionales.md` §3.10: the empty state has to be reachable, otherwise a
    /// cell clicked by mistake can never be cleared.
    pub const fn siguiente(actual: Option<Self>) -> Option<Self> {
        match actual {
            None => Some(Self::Completa),
            Some(Self::Completa) => Some(Self::Media),
            Some(Self::Media) => Some(Self::Falta),
            Some(Self::Falta) => Some(Self::FaltaJustificada),
            Some(Self::FaltaJustificada) => Some(Self::Feriado),
            Some(Self::Feriado) => None,
        }
    }
}

/// `docs/05-dominio-entidades.md` §3.9.
///
/// The divisors only turn a salary into a suggested daily rate; the payroll always uses the rate
/// stored on the employee. `Semanal` divides by six because the working week runs Monday to
/// Saturday.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum FrecuenciaPago {
    Diario,
    Semanal,
    Quincenal,
    #[default]
    Mensual,
}

impl FrecuenciaPago {
    pub const ALL: [Self; 4] = [Self::Diario, Self::Semanal, Self::Quincenal, Self::Mensual];

    pub const fn as_i32(self) -> i32 {
        match self {
            Self::Diario => 0,
            Self::Semanal => 1,
            Self::Quincenal => 2,
            Self::Mensual => 3,
        }
    }

    pub fn from_i32(value: i32) -> Result<Self, DomainError> {
        match value {
            0 => Ok(Self::Diario),
            1 => Ok(Self::Semanal),
            2 => Ok(Self::Quincenal),
            3 => Ok(Self::Mensual),
            other => Err(DomainError::UnknownEnumValue {
                enum_name: "FrecuenciaPago",
                value: other,
            }),
        }
    }

    /// Default divisors; configuration can override them through `Business.DiasPorFrecuencia.*`.
    pub const fn dias_por_periodo(self) -> Decimal4 {
        match self {
            Self::Diario => Decimal4::ONE,
            Self::Semanal => Decimal4::from_raw(60_000),
            Self::Quincenal => Decimal4::from_raw(150_000),
            Self::Mensual => Decimal4::from_raw(300_000),
        }
    }
}
