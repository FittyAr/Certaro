//! Contract of the dollar quotes. See `docs/11-contratos-tauri.md` §5.10 and
//! `docs/13-servicios-externos-y-archivos.md` §2.

use chrono::{DateTime, Utc};
use eo_domain::Money;
use serde::{Deserialize, Serialize};

/// One exchange house as the dashboard and the status bar show it.
///
/// `compra` and `venta` are `Money`, so the value that reaches a movement is the same value the
/// API sent: parsing them through `f64` would already have lost digits (doc 04 §1.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cotizacion {
    /// `oficial`, `blue`, … Always lowercase, which is how the filter compares it.
    pub casa: String,
    pub nombre: String,
    pub compra: Money,
    pub venta: Money,
    pub fecha_actualizacion: DateTime<Utc>,
    /// True when this row came from the cache because the request failed. The screen shows the
    /// date instead of hiding the block, so the user knows the number is old rather than wrong.
    #[serde(default)]
    pub desactualizada: bool,
}
