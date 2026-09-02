//! Fixed values of the domain: seeded identifiers and length limits.
//!
//! The system identifiers are constants rather than lookups by name because the payroll filters
//! advances by the `Adelanto` identifier, and a user renaming that row must not break the
//! calculation. See `docs/03-modelo-de-datos.md` §5.

use uuid::{uuid, Uuid};

/// The four seeded rows of `tipos_movimiento`.
pub mod tipos_movimiento {
    use super::*;

    pub const INGRESO: Uuid = uuid!("00000000-0000-0000-0000-000000000001");
    pub const GASTO: Uuid = uuid!("00000000-0000-0000-0000-000000000002");
    /// The one the payroll filters by to find an employee's advances.
    pub const ADELANTO: Uuid = uuid!("00000000-0000-0000-0000-000000000003");
    /// Seeded with `es_ingreso = true`: an adjustment carries its own sign in the amount.
    pub const AJUSTE: Uuid = uuid!("00000000-0000-0000-0000-000000000004");

    pub const TODOS: [Uuid; 4] = [INGRESO, GASTO, ADELANTO, AJUSTE];
}

/// The four seeded rows of `tipos_concepto_pago`.
pub mod tipos_concepto_pago {
    use super::*;

    pub const ADELANTO: Uuid = uuid!("00000000-0000-0000-0000-000000000101");
    pub const QUINCENA: Uuid = uuid!("00000000-0000-0000-0000-000000000102");
    pub const LIQUIDACION: Uuid = uuid!("00000000-0000-0000-0000-000000000103");
    pub const VIATICO: Uuid = uuid!("00000000-0000-0000-0000-000000000104");

    pub const TODOS: [Uuid; 4] = [ADELANTO, QUINCENA, LIQUIDACION, VIATICO];
}

/// Accepted values of `adjuntos.entidad_tipo`. The relation is polymorphic and has no foreign
/// key, so this list is the only thing keeping a typo from creating an unreachable attachment.
pub mod entidad_adjunto {
    pub const PROYECTO: &str = "Proyecto";
    pub const TRABAJO: &str = "Trabajo";
    pub const FACTURA: &str = "Factura";
    pub const MOVIMIENTO: &str = "Movimiento";
    pub const EMPLEADO: &str = "Empleado";

    pub const TODOS: [&str; 5] = [PROYECTO, TRABAJO, FACTURA, MOVIMIENTO, EMPLEADO];

    pub fn es_valido(valor: &str) -> bool {
        TODOS.contains(&valor)
    }
}

/// Maximum lengths, in characters, taken from the column comments of `docs/03-modelo-de-datos.md`.
pub mod limites {
    pub const NOMBRE_CORTO: usize = 100;
    pub const NOMBRE_LARGO: usize = 200;
    pub const DESCRIPCION: usize = 500;
    pub const OBSERVACIONES: usize = 1000;
    pub const EMAIL: usize = 254;
    pub const TELEFONO: usize = 30;
    pub const CUIT: usize = 13;
    pub const DNI: usize = 15;
    pub const COLOR_HEX: usize = 7;
    pub const ICONO: usize = 50;
    pub const UNIDAD: usize = 20;
    pub const NUMERO_COMPROBANTE: usize = 50;
    pub const NUMERO_FACTURA: usize = 50;
    pub const NUMERO_CERTIFICADO: usize = 50;
    pub const DIRECCION: usize = 500;
    pub const MEDIO_PAGO: usize = 100;
    pub const NOMBRE_ARCHIVO: usize = 255;
    pub const MIME: usize = 100;
    pub const METADATA_KEY: usize = 100;
    pub const METADATA_VALUE: usize = 500;
}
