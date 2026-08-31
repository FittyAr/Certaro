//! Monetary scaling. See `docs/15-migracion-de-datos.md` §3.2–3.4.

use crate::report::ScaleState;

/// The threshold above which a `CotizacionAplicada` value is considered already scaled.
/// Equals a cotización of 100.0000 × 10_000 = 1_000_000.
pub const UMBRAL_COTIZACION_ESCALADA: i64 = 1_000_000;

/// Applies the scale factor to a monetary value based on the detected scale state.
#[must_use]
pub fn scale_value(raw: i64, state: ScaleState) -> i64 {
    match state {
        ScaleState::AlreadyScaled => raw,
        ScaleState::UnscaledIntegers => raw * 10_000,
        ScaleState::Unknown => raw,
    }
}

/// Handles the special case of `PagosFactura.Monto`: the scale may differ per row.
/// Returns `(scaled_value, was_heuristic)`.
#[must_use]
pub fn scale_pago(raw: i64, invoice_total: i64, state: ScaleState) -> (i64, bool) {
    if state == ScaleState::UnscaledIntegers {
        return (raw * 10_000, false);
    }
    // AlreadyScaled: check if this specific row is actually scaled.
    // A scaled payment is within one order of magnitude of the invoice total.
    // An unscaled payment is ~10_000× smaller, so after scaling it should be comparable.
    if raw >= invoice_total / 10 {
        // Already scaled: the payment is within 1 order of the invoice total.
        (raw, false)
    } else if raw * 10_000 <= invoice_total * 100 {
        // The payment is ~10_000× smaller: unscaled.
        (raw * 10_000, true)
    } else {
        // Improbable: mark for manual review. Import as-is.
        (raw, true)
    }
}

/// Handles `Movimientos.CotizacionAplicada`: range-based disambiguation.
/// Returns `(scaled_value, was_heuristic, is_zero)`.
#[must_use]
pub fn scale_cotizacion(raw: Option<i64>, state: ScaleState) -> (Option<i64>, bool, bool) {
    let Some(raw) = raw else {
        return (None, false, false);
    };
    if raw == 0 {
        // A zero cotización is invalid data, not a scale issue.
        return (None, false, true);
    }
    if state == ScaleState::UnscaledIntegers {
        return (Some(raw * 10_000), false, false);
    }
    // AlreadyScaled: use the threshold.
    if raw >= UMBRAL_COTIZACION_ESCALADA {
        (Some(raw), false, false)
    } else {
        (Some(raw * 10_000), true, false)
    }
}

/// Columns where a raw value of `0` means "created before the column existed" and should be
/// treated as `1.0` (i.e., `10_000` in scaled representation).
#[must_use]
pub fn default_zero_to_one(raw: i64) -> i64 {
    if raw == 0 {
        10_000
    } else {
        raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pago_ya_escalado_no_se_toca() {
        let (val, heuristic) = scale_pago(45_000_000, 120_000_000, ScaleState::AlreadyScaled);
        assert_eq!(val, 45_000_000);
        assert!(!heuristic);
    }

    #[test]
    fn pago_sin_escalar_se_multiplica() {
        let (val, heuristic) = scale_pago(4500, 120_000_000, ScaleState::AlreadyScaled);
        assert_eq!(val, 45_000_000);
        assert!(heuristic);
    }

    #[test]
    fn cotizacion_cero_se_importa_null() {
        let (val, _, is_zero) = scale_cotizacion(Some(0), ScaleState::AlreadyScaled);
        assert!(val.is_none());
        assert!(is_zero);
    }

    #[test]
    fn cotizacion_por_encima_del_umbral_ya_esta_escalada() {
        let (val, heuristic, _) = scale_cotizacion(Some(1_500_000), ScaleState::AlreadyScaled);
        assert_eq!(val, Some(1_500_000));
        assert!(!heuristic);
    }

    #[test]
    fn cotizacion_por_debajo_del_umbral_se_escala() {
        let (val, heuristic, _) = scale_cotizacion(Some(1500), ScaleState::AlreadyScaled);
        assert_eq!(val, Some(15_000_000));
        assert!(heuristic);
    }

    #[test]
    fn cantidad_cero_se_vuelve_uno() {
        assert_eq!(default_zero_to_one(0), 10_000);
        assert_eq!(default_zero_to_one(5000), 5000);
    }
}
