use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::error::DomainError;

/// A non-working day of the calendar. See `docs/05-dominio-entidades.md` §2.9.
///
/// The date is the primary key: one holiday per day. There is no soft delete here — removing a
/// holiday is a real delete, because a row that lingers would keep paying a multiplier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feriado {
    pub fecha: NaiveDate,
    pub nombre: String,
    /// Whatever the API reports; free text, and empty for a hand-added one.
    pub tipo: Option<String>,
    pub origen: OrigenFeriado,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Where a holiday came from. A hand-added holiday is never overwritten by a sync: someone typed it
/// because the API did not have it. See `docs/13-servicios-externos-y-archivos.md` §3.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum OrigenFeriado {
    #[default]
    Api,
    Manual,
}

impl OrigenFeriado {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Api => "Api",
            Self::Manual => "Manual",
        }
    }

    pub fn parse(value: &str) -> Result<Self, DomainError> {
        match value {
            "Api" => Ok(Self::Api),
            "Manual" => Ok(Self::Manual),
            _ => Err(DomainError::UnknownEnumValue {
                enum_name: "OrigenFeriado",
                value: -1,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_origen_va_y_vuelve_por_su_texto() {
        for origen in [OrigenFeriado::Api, OrigenFeriado::Manual] {
            assert_eq!(OrigenFeriado::parse(origen.as_str()).unwrap(), origen);
        }
    }

    #[test]
    fn un_origen_desconocido_no_se_adivina() {
        assert!(OrigenFeriado::parse("Ministerio").is_err());
    }
}
