use serde::{Deserialize, Serialize};
use crate::error::DomainError;

/// `docs/05-dominio-entidades.md` §3.2.
///
/// `PagadaParcial` is new and takes the value 5 rather than slotting in next to `Pagada`: the
/// integers are already on disk and renumbering them would silently reinterpret every stored row.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EstadoFactura {
    #[default]
    Borrador,
    Emitida,
    Pagada,
    Anulada,
    Vencida,
    PagadaParcial,
}

impl EstadoFactura {
    pub const ALL: [Self; 6] = [
        Self::Borrador,
        Self::Emitida,
        Self::Pagada,
        Self::Anulada,
        Self::Vencida,
        Self::PagadaParcial,
    ];

    pub const fn as_i32(self) -> i32 {
        match self {
            Self::Borrador => 0,
            Self::Emitida => 1,
            Self::Pagada => 2,
            Self::Anulada => 3,
            Self::Vencida => 4,
            Self::PagadaParcial => 5,
        }
    }

    pub fn from_i32(value: i32) -> Result<Self, DomainError> {
        match value {
            0 => Ok(Self::Borrador),
            1 => Ok(Self::Emitida),
            2 => Ok(Self::Pagada),
            3 => Ok(Self::Anulada),
            4 => Ok(Self::Vencida),
            5 => Ok(Self::PagadaParcial),
            other => Err(DomainError::UnknownEnumValue {
                enum_name: "EstadoFactura",
                value: other,
            }),
        }
    }

    /// A draft is not a debt yet and an annulled invoice never was one, so both stay out of every
    /// receivables figure. See `docs/08-maquinas-de-estado.md` §2.6.
    pub const fn cuenta_como_deuda(self) -> bool {
        !matches!(self, Self::Borrador | Self::Anulada)
    }

    /// Only an invoice that is out in the world and still owes something takes money.
    pub const fn admite_pagos(self) -> bool {
        matches!(self, Self::Emitida | Self::PagadaParcial | Self::Vencida)
    }
}

/// `docs/05-dominio-entidades.md` §3.3.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EstadoProyecto {
    #[default]
    Activa,
    Pausada,
    Finalizada,
    Cancelada,
}

impl EstadoProyecto {
    pub const ALL: [Self; 4] = [
        Self::Activa,
        Self::Pausada,
        Self::Finalizada,
        Self::Cancelada,
    ];

    pub const fn as_i32(self) -> i32 {
        match self {
            Self::Activa => 0,
            Self::Pausada => 1,
            Self::Finalizada => 2,
            Self::Cancelada => 3,
        }
    }

    pub fn from_i32(value: i32) -> Result<Self, DomainError> {
        match value {
            0 => Ok(Self::Activa),
            1 => Ok(Self::Pausada),
            2 => Ok(Self::Finalizada),
            3 => Ok(Self::Cancelada),
            other => Err(DomainError::UnknownEnumValue {
                enum_name: "EstadoProyecto",
                value: other,
            }),
        }
    }

    /// A site that is paused, closed or dead does not take new jobs; only an active one does.
    pub const fn admite_trabajos_nuevos(self) -> bool {
        matches!(self, Self::Activa)
    }
}

/// `docs/05-dominio-entidades.md` §3.4.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EstadoTrabajo {
    #[default]
    Presupuestado,
    EnProceso,
    Pausado,
    Finalizado,
    Cancelado,
}

impl EstadoTrabajo {
    pub const ALL: [Self; 5] = [
        Self::Presupuestado,
        Self::EnProceso,
        Self::Pausado,
        Self::Finalizado,
        Self::Cancelado,
    ];

    pub const fn as_i32(self) -> i32 {
        match self {
            Self::Presupuestado => 0,
            Self::EnProceso => 1,
            Self::Pausado => 2,
            Self::Finalizado => 3,
            Self::Cancelado => 4,
        }
    }

    pub fn from_i32(value: i32) -> Result<Self, DomainError> {
        match value {
            0 => Ok(Self::Presupuestado),
            1 => Ok(Self::EnProceso),
            2 => Ok(Self::Pausado),
            3 => Ok(Self::Finalizado),
            4 => Ok(Self::Cancelado),
            other => Err(DomainError::UnknownEnumValue {
                enum_name: "EstadoTrabajo",
                value: other,
            }),
        }
    }

    /// A job still open is one that has not been closed one way or the other; the site cannot be
    /// finalised while any of these remain. See `docs/08-maquinas-de-estado.md` §3.3.
    pub const fn esta_abierto(self) -> bool {
        matches!(self, Self::Presupuestado | Self::EnProceso | Self::Pausado)
    }
}
