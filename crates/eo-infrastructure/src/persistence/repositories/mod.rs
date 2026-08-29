//! SeaORM implementations of the repository ports.

pub mod categoria;
pub mod certificado;
pub mod cliente;
pub mod factura;
pub mod movimiento;
pub mod obra;
pub mod orden_trabajo;
pub mod tipo_movimiento;
pub mod trabajo;

use eo_domain::EstadoFactura;

/// The invoice states that count as debt, as the integers the column stores.
///
/// The list is derived from the domain rather than written out, so adding a state cannot leave the
/// receivables queries reading a stale set of numbers.
pub(crate) fn estado_deuda_ids() -> Vec<i32> {
    EstadoFactura::ALL
        .iter()
        .filter(|e| e.cuenta_como_deuda())
        .map(|e| e.as_i32())
        .collect()
}
