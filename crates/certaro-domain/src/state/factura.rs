//! `docs/08-maquinas-de-estado.md` §2.

use chrono::NaiveDate;

use crate::entities::Factura;
use crate::enums::EstadoFactura;
use crate::error::DomainError;
use crate::state::StateMachine;

impl StateMachine for EstadoFactura {
    const ENTITY: &'static str = "Factura";

    /// `Pagada`, `PagadaParcial` and `Vencida` are never targets here: they are written by
    /// [`recalcular_estado_factura`] alone, so they cannot show up in a dropdown.
    fn allowed_targets(self) -> &'static [Self] {
        use EstadoFactura::*;
        match self {
            Borrador => &[Emitida, Anulada],
            Emitida => &[Borrador, Anulada],
            PagadaParcial => &[Anulada],
            Vencida => &[Anulada],
            Pagada => &[],
            Anulada => &[],
        }
    }

    fn as_key(self) -> &'static str {
        match self {
            Self::Borrador => "Borrador",
            Self::Emitida => "Emitida",
            Self::Pagada => "Pagada",
            Self::Anulada => "Anulada",
            Self::Vencida => "Vencida",
            Self::PagadaParcial => "PagadaParcial",
        }
    }
}

/// Derives the state of an invoice from its payments and the date (T-F07 … T-F11).
///
/// This is the only place that writes `Pagada`, `PagadaParcial` and `Vencida`. It is idempotent,
/// and it never pulls an invoice out of `Borrador` or `Anulada`: a draft has not been issued and
/// annulling is a human decision that no recalculation gets to undo.
pub fn recalcular_estado_factura(
    factura: &mut Factura,
    hoy: NaiveDate,
    dias_default: u32,
) -> Result<(), DomainError> {
    if matches!(
        factura.estado,
        EstadoFactura::Borrador | EstadoFactura::Anulada
    ) {
        return Ok(());
    }

    let saldo = factura.saldo_pendiente()?;
    let pagado = factura.total_pagado()?;

    factura.estado = if saldo.raw() <= 0 {
        EstadoFactura::Pagada
    } else if factura.esta_vencida(hoy, dias_default)? {
        EstadoFactura::Vencida
    } else if pagado.is_positive() {
        EstadoFactura::PagadaParcial
    } else {
        EstadoFactura::Emitida
    };

    Ok(())
}
