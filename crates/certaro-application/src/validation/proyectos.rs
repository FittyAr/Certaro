//! V-06 of `docs/07-validaciones.md`.
//!
//! The document also lists `fechaFin`, `presupuesto` and `observaciones` rules for this DTO, but
//! `proyectos` has none of those columns (`docs/03-modelo-de-datos.md` §3.6) and neither does the
//! entity, so there is nothing to validate. Those rules live on `Trabajo`, which does have them.

use certaro_domain::constants::limites;
use uuid::Uuid;

use crate::dtos::proyectos::ProyectoInput;
use crate::error::FieldError;
use crate::result::AppResult;
use crate::validation::Validator;

pub fn validate(input: &ProyectoInput) -> AppResult<()> {
    let mut v = Validator::new();

    v.required_text("nombre", &input.nombre, "Validation.Proyecto.NombreRequired");
    v.max_length(
        "nombre",
        &input.nombre,
        limites::NOMBRE_LARGO,
        "Validation.Proyecto.NombreMaxLength",
    );

    v.require(
        input.cliente_id != Uuid::nil(),
        FieldError::new("clienteId", "Validation.Proyecto.ClienteRequired"),
    );

    // The number is what the customer says on the phone, so zero and negatives are typing
    // accidents rather than valid identifiers.
    v.require(
        input.numero > 0,
        FieldError::new("numero", "Validation.Proyecto.NumeroRequired"),
    );

    v.max_length_opt(
        "direccion",
        input.direccion.as_deref(),
        limites::DIRECCION,
        "Validation.Proyecto.DireccionMaxLength",
    );
    v.max_length_opt(
        "localidad",
        input.localidad.as_deref(),
        limites::NOMBRE_LARGO,
        "Validation.Proyecto.LocalidadMaxLength",
    );

    v.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppError;

    fn input() -> ProyectoInput {
        ProyectoInput {
            numero: 1892,
            nombre: "Tecnocasa Mercedes".into(),
            direccion: None,
            localidad: None,
            cliente_id: Uuid::from_u128(1),
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
    fn un_proyecto_minimo_es_valido() {
        assert!(validate(&input()).is_ok());
    }

    #[test]
    fn el_numero_tiene_que_ser_positivo() {
        for numero in [0, -1] {
            let dto = ProyectoInput { numero, ..input() };
            assert_eq!(
                keys(validate(&dto).unwrap_err()),
                ["Validation.Proyecto.NumeroRequired"]
            );
        }
    }

    #[test]
    fn el_cliente_es_obligatorio() {
        let dto = ProyectoInput {
            cliente_id: Uuid::nil(),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.Proyecto.ClienteRequired"]
        );
    }

    #[test]
    fn el_nombre_en_el_limite_pasa() {
        let dto = ProyectoInput {
            nombre: "x".repeat(limites::NOMBRE_LARGO),
            ..input()
        };
        assert!(validate(&dto).is_ok());

        let dto = ProyectoInput {
            nombre: "x".repeat(limites::NOMBRE_LARGO + 1),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.Proyecto.NombreMaxLength"]
        );
    }
}
