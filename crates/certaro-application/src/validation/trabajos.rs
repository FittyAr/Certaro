//! V-07 of `docs/07-validaciones.md`.

use certaro_domain::constants::limites;
use uuid::Uuid;

use crate::dtos::trabajos::TrabajoInput;
use crate::error::FieldError;
use crate::result::AppResult;
use crate::validation::Validator;

pub fn validate(input: &TrabajoInput) -> AppResult<()> {
    let mut v = Validator::new();

    v.required_text(
        "descripcion",
        &input.descripcion,
        "Validation.Trabajo.DescripcionRequired",
    );
    v.max_length(
        "descripcion",
        &input.descripcion,
        limites::DESCRIPCION,
        "Validation.Trabajo.DescripcionMaxLength",
    );

    // The job hangs off the site, not off the customer: the customer is reached through the site.
    v.require(
        input.obra_id != Uuid::nil(),
        FieldError::new("obraId", "Validation.Trabajo.ObraRequired"),
    );

    if let Some(fin) = input.fecha_fin {
        v.require(
            fin >= input.fecha_inicio,
            FieldError::new("fechaFin", "Validation.Trabajo.FechaFinInvalid"),
        );
    }

    v.require(
        !input.presupuesto.is_negative(),
        FieldError::new("presupuesto", "Validation.Trabajo.PresupuestoNegative"),
    );

    v.finish()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use certaro_domain::Money;

    use super::*;
    use crate::AppError;

    fn dia(d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 3, d).unwrap()
    }

    fn input() -> TrabajoInput {
        TrabajoInput {
            obra_id: Uuid::from_u128(1),
            descripcion: "Tendido de cañerías".into(),
            fecha_inicio: dia(1),
            fecha_fin: None,
            presupuesto: Money::parse("100000").unwrap(),
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
    fn un_trabajo_minimo_es_valido() {
        assert!(validate(&input()).is_ok());
    }

    #[test]
    fn la_fecha_de_fin_no_puede_preceder_a_la_de_inicio() {
        let dto = TrabajoInput {
            fecha_fin: Some(dia(1) - chrono::Duration::days(1)),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.Trabajo.FechaFinInvalid"]
        );
    }

    #[test]
    fn empezar_y_terminar_el_mismo_dia_es_valido() {
        let dto = TrabajoInput {
            fecha_fin: Some(dia(1)),
            ..input()
        };
        assert!(validate(&dto).is_ok());
    }

    #[test]
    fn el_presupuesto_no_puede_ser_negativo() {
        let dto = TrabajoInput {
            presupuesto: Money::parse("-1").unwrap(),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.Trabajo.PresupuestoNegative"]
        );
    }

    #[test]
    fn un_presupuesto_en_cero_es_valido() {
        // A job quoted at zero is a job done as a favour, and the legacy data has plenty.
        let dto = TrabajoInput {
            presupuesto: Money::ZERO,
            ..input()
        };
        assert!(validate(&dto).is_ok());
    }

    #[test]
    fn la_obra_es_obligatoria() {
        let dto = TrabajoInput {
            obra_id: Uuid::nil(),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.Trabajo.ObraRequired"]
        );
    }
}
