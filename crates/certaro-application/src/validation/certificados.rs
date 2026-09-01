//! V-10 of `docs/07-validaciones.md`.

use certaro_domain::constants::limites;
use uuid::Uuid;

use crate::dtos::certificados::CertificadoInput;
use crate::error::FieldError;
use crate::result::AppResult;
use crate::validation::movimientos::ContextoFecha;
use crate::validation::Validator;

/// Shape check of the issuing form. The accumulated ceiling (doc 07 §5.3) and the sequential
/// number (INV-15) need the database and live in the use case.
pub fn validate(input: &CertificadoInput, fechas: &ContextoFecha) -> AppResult<()> {
    let mut v = Validator::new();

    v.require(
        input.orden_trabajo_id != Uuid::nil(),
        FieldError::new(
            "ordenTrabajoId",
            "Validation.Certificado.OrdenTrabajoRequired",
        ),
    );

    v.require(
        !input.items.is_empty(),
        FieldError::new("items", "Validation.Certificado.ItemsRequired"),
    );

    let maxima = fechas.hoy + chrono::Duration::days(fechas.max_dias_futuro);
    v.require(
        input.fecha >= fechas.minima && input.fecha <= maxima,
        FieldError::new("fecha", "Validation.Common.FechaOutOfRange")
            .with_param("minima", fechas.minima)
            .with_param("maxima", maxima),
    );

    v.max_length_opt(
        "observaciones",
        input.observaciones.as_deref(),
        limites::OBSERVACIONES,
        "Validation.Certificado.ObservacionesMaxLength",
    );

    for (i, item) in input.items.iter().enumerate() {
        v.require(
            item.orden_trabajo_item_id != Uuid::nil(),
            FieldError::new(
                format!("items[{i}].ordenTrabajoItemId"),
                "Validation.Certificado.ItemsRequired",
            ),
        );
        v.require(
            item.porcentaje_actual.is_valid_percentage(),
            FieldError::new(
                format!("items[{i}].porcentajeActual"),
                "Validation.Certificado.PorcentajeInvalid",
            ),
        );
    }

    v.finish()
}

/// The notes are the only field an issued certificate lets through (doc 08 §5.1), so they get
/// their own check rather than reusing the one above with a dummy body.
pub fn validate_observaciones(observaciones: Option<&str>) -> AppResult<()> {
    let mut v = Validator::new();
    v.max_length_opt(
        "observaciones",
        observaciones,
        limites::OBSERVACIONES,
        "Validation.Certificado.ObservacionesMaxLength",
    );
    v.finish()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use certaro_domain::Decimal4;

    use super::*;
    use crate::dtos::certificados::CertificadoInputItem;
    use crate::AppError;

    fn contexto() -> ContextoFecha {
        ContextoFecha {
            hoy: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            minima: NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
            max_dias_futuro: 365,
        }
    }

    fn input() -> CertificadoInput {
        CertificadoInput {
            orden_trabajo_id: Uuid::from_u128(1),
            fecha: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            observaciones: None,
            items: vec![CertificadoInputItem {
                orden_trabajo_item_id: Uuid::from_u128(2),
                porcentaje_actual: Decimal4::parse("60").unwrap(),
            }],
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
    fn un_certificado_minimo_es_valido() {
        assert!(validate(&input(), &contexto()).is_ok());
    }

    #[test]
    fn sin_items_no_hay_certificado() {
        let dto = CertificadoInput {
            items: vec![],
            ..input()
        };
        assert_eq!(
            keys(validate(&dto, &contexto()).unwrap_err()),
            ["Validation.Certificado.ItemsRequired"]
        );
    }

    #[test]
    fn el_porcentaje_esta_acotado() {
        let dto = CertificadoInput {
            items: vec![CertificadoInputItem {
                orden_trabajo_item_id: Uuid::from_u128(2),
                porcentaje_actual: Decimal4::parse("100.5").unwrap(),
            }],
            ..input()
        };
        assert_eq!(
            keys(validate(&dto, &contexto()).unwrap_err()),
            ["Validation.Certificado.PorcentajeInvalid"]
        );
    }

    #[test]
    fn una_fecha_de_hace_veinte_anios_se_rechaza() {
        let dto = CertificadoInput {
            fecha: NaiveDate::from_ymd_opt(1999, 12, 31).unwrap(),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto, &contexto()).unwrap_err()),
            ["Validation.Common.FechaOutOfRange"]
        );
    }
}
