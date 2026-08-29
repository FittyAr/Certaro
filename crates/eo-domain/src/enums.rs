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
pub enum EstadoObra {
    #[default]
    Activa,
    Pausada,
    Finalizada,
    Cancelada,
}

impl EstadoObra {
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
                enum_name: "EstadoObra",
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

    #[test]
    fn los_estados_van_y_vuelven_por_su_entero() {
        for estado in EstadoFactura::ALL {
            assert_eq!(EstadoFactura::from_i32(estado.as_i32()).unwrap(), estado);
        }
        for estado in EstadoObra::ALL {
            assert_eq!(EstadoObra::from_i32(estado.as_i32()).unwrap(), estado);
        }
        for estado in EstadoTrabajo::ALL {
            assert_eq!(EstadoTrabajo::from_i32(estado.as_i32()).unwrap(), estado);
        }
    }

    #[test]
    fn pagada_parcial_conserva_el_cinco() {
        // The value is load-bearing: every stored row was written with these integers.
        assert_eq!(EstadoFactura::PagadaParcial.as_i32(), 5);
        assert_eq!(EstadoFactura::Pagada.as_i32(), 2);
    }

    #[test]
    fn solo_las_facturas_vivas_con_saldo_admiten_pagos() {
        assert!(EstadoFactura::Emitida.admite_pagos());
        assert!(EstadoFactura::PagadaParcial.admite_pagos());
        assert!(EstadoFactura::Vencida.admite_pagos());
        assert!(!EstadoFactura::Borrador.admite_pagos());
        assert!(!EstadoFactura::Pagada.admite_pagos());
        assert!(!EstadoFactura::Anulada.admite_pagos());
    }

    #[test]
    fn el_borrador_y_la_anulada_no_son_deuda() {
        assert!(!EstadoFactura::Borrador.cuenta_como_deuda());
        assert!(!EstadoFactura::Anulada.cuenta_como_deuda());
        assert!(EstadoFactura::Emitida.cuenta_como_deuda());
        assert!(EstadoFactura::PagadaParcial.cuenta_como_deuda());
    }

    #[test]
    fn un_trabajo_cerrado_no_esta_abierto() {
        assert!(EstadoTrabajo::Presupuestado.esta_abierto());
        assert!(EstadoTrabajo::EnProceso.esta_abierto());
        assert!(EstadoTrabajo::Pausado.esta_abierto());
        assert!(!EstadoTrabajo::Finalizado.esta_abierto());
        assert!(!EstadoTrabajo::Cancelado.esta_abierto());
    }
}
