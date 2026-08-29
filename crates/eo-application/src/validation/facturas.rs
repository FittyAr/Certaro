//! V-11 and V-12 of `docs/07-validaciones.md`.

use eo_domain::constants::limites;
use uuid::Uuid;

use crate::dtos::facturas::{FacturaInput, PagoFacturaInput};
use crate::error::FieldError;
use crate::result::AppResult;
use crate::validation::movimientos::ContextoFecha;
use crate::validation::Validator;

pub fn validate_factura(input: &FacturaInput) -> AppResult<()> {
    let mut v = Validator::new();

    v.required_text("numero", &input.numero, "Validation.Factura.NumeroRequired");
    v.max_length(
        "numero",
        &input.numero,
        limites::NUMERO_FACTURA,
        "Validation.Factura.NumeroMaxLength",
    );

    v.require(
        input.cliente_id != Uuid::nil(),
        FieldError::new("clienteId", "Validation.Factura.ClienteRequired"),
    );

    v.require(
        !input.subtotal.is_negative(),
        FieldError::new("subtotal", "Validation.Factura.SubtotalInvalid"),
    );
    v.require(
        !input.iva.is_negative(),
        FieldError::new("iva", "Validation.Factura.IvaInvalid"),
    );

    // The use case recomputes the total anyway. This rule exists to catch a frontend that has
    // drifted out of step, which is how the legacy system ended up storing totals that did not
    // add up to their own parts.
    if let Ok(esperado) = input.subtotal.checked_add(input.iva) {
        v.require(
            input.total == esperado,
            FieldError::new("total", "Validation.Factura.TotalMismatch")
                .with_param("esperado", esperado.to_decimal_string()),
        );
    }

    // INV-16.
    if let Some(vencimiento) = input.fecha_vencimiento {
        v.require(
            vencimiento >= input.fecha,
            FieldError::new(
                "fechaVencimiento",
                "Validation.Factura.FechaVencimientoInvalid",
            ),
        );
    }

    v.max_length_opt(
        "observaciones",
        input.observaciones.as_deref(),
        limites::OBSERVACIONES,
        "Validation.Factura.ObservacionesMaxLength",
    );

    v.finish()
}

pub fn validate_pago(input: &PagoFacturaInput, fechas: &ContextoFecha) -> AppResult<()> {
    let mut v = Validator::new();

    v.require(
        input.factura_id != Uuid::nil(),
        FieldError::new("facturaId", "Validation.PagoFactura.FacturaRequired"),
    );

    // A payment of zero records nothing and a negative one is a refund, which this table does not
    // model: it would silently increase the balance owed.
    v.require(
        input.monto.is_positive(),
        FieldError::new("monto", "Validation.PagoFactura.MontoRequired"),
    );

    v.required_text(
        "medioPago",
        &input.medio_pago,
        "Validation.PagoFactura.MedioPagoRequired",
    );
    v.max_length(
        "medioPago",
        &input.medio_pago,
        limites::MEDIO_PAGO,
        "Validation.PagoFactura.MedioPagoMaxLength",
    );

    let maxima = fechas.hoy + chrono::Duration::days(fechas.max_dias_futuro);
    v.require(
        input.fecha >= fechas.minima && input.fecha <= maxima,
        FieldError::new("fecha", "Validation.Common.FechaOutOfRange")
            .with_param("min", fechas.minima.to_string())
            .with_param("max", maxima.to_string()),
    );

    v.finish()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use eo_domain::Money;

    use super::*;
    use crate::AppError;

    fn dia(d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 5, d).unwrap()
    }

    fn factura() -> FacturaInput {
        FacturaInput {
            numero: "0001-00000123".into(),
            fecha: dia(1),
            fecha_vencimiento: None,
            cliente_id: Uuid::from_u128(1),
            subtotal: Money::parse("1000").unwrap(),
            iva: Money::parse("210").unwrap(),
            total: Money::parse("1210").unwrap(),
            observaciones: None,
        }
    }

    fn contexto() -> ContextoFecha {
        ContextoFecha {
            hoy: dia(10),
            minima: NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
            max_dias_futuro: 365,
        }
    }

    fn pago() -> PagoFacturaInput {
        PagoFacturaInput {
            factura_id: Uuid::from_u128(1),
            fecha: dia(5),
            monto: Money::parse("500").unwrap(),
            medio_pago: "Transferencia".into(),
        }
    }

    fn keys(error: AppError) -> Vec<String> {
        error
            .fields()
            .iter()
            .map(|f| f.message_key.clone())
            .collect()
    }

    #[test]
    fn una_factura_coherente_es_valida() {
        assert!(validate_factura(&factura()).is_ok());
    }

    #[test]
    fn el_total_tiene_que_cerrar_con_sus_partes() {
        let dto = FacturaInput {
            total: Money::parse("1200").unwrap(),
            ..factura()
        };
        let error = validate_factura(&dto).unwrap_err();
        assert_eq!(keys(error), ["Validation.Factura.TotalMismatch"]);
    }

    #[test]
    fn el_total_esperado_viaja_en_los_params() {
        let dto = FacturaInput {
            total: Money::ZERO,
            ..factura()
        };
        let error = validate_factura(&dto).unwrap_err();
        assert_eq!(
            error.fields()[0].params.get("esperado").map(String::as_str),
            Some("1210.0000")
        );
    }

    #[test]
    fn el_iva_puede_ser_cero() {
        let dto = FacturaInput {
            iva: Money::ZERO,
            total: Money::parse("1000").unwrap(),
            ..factura()
        };
        assert!(validate_factura(&dto).is_ok());
    }

    #[test]
    fn el_vencimiento_no_puede_preceder_a_la_emision() {
        let dto = FacturaInput {
            fecha_vencimiento: Some(NaiveDate::from_ymd_opt(2026, 4, 30).unwrap()),
            ..factura()
        };
        assert_eq!(
            keys(validate_factura(&dto).unwrap_err()),
            ["Validation.Factura.FechaVencimientoInvalid"]
        );
    }

    #[test]
    fn vencer_el_mismo_dia_de_emision_es_valido() {
        let dto = FacturaInput {
            fecha_vencimiento: Some(dia(1)),
            ..factura()
        };
        assert!(validate_factura(&dto).is_ok());
    }

    #[test]
    fn un_pago_valido_pasa() {
        assert!(validate_pago(&pago(), &contexto()).is_ok());
    }

    #[test]
    fn el_monto_del_pago_tiene_que_ser_positivo() {
        for monto in ["0", "-100"] {
            let dto = PagoFacturaInput {
                monto: Money::parse(monto).unwrap(),
                ..pago()
            };
            assert_eq!(
                keys(validate_pago(&dto, &contexto()).unwrap_err()),
                ["Validation.PagoFactura.MontoRequired"]
            );
        }
    }

    #[test]
    fn una_fecha_de_pago_absurda_se_rechaza_con_los_limites() {
        let dto = PagoFacturaInput {
            fecha: NaiveDate::from_ymd_opt(1999, 12, 31).unwrap(),
            ..pago()
        };
        let error = validate_pago(&dto, &contexto()).unwrap_err();
        assert_eq!(
            error.fields()[0].message_key,
            "Validation.Common.FechaOutOfRange"
        );
        assert_eq!(
            error.fields()[0].params.get("min").map(String::as_str),
            Some("2000-01-01")
        );
    }

    #[test]
    fn el_medio_de_pago_es_obligatorio() {
        let dto = PagoFacturaInput {
            medio_pago: "  ".into(),
            ..pago()
        };
        assert_eq!(
            keys(validate_pago(&dto, &contexto()).unwrap_err()),
            ["Validation.PagoFactura.MedioPagoRequired"]
        );
    }
}
