//! V-03 of `docs/07-validaciones.md`.
//!
//! The document also lists `color` and `icono` rules for this DTO, but `tipos_movimiento` has no
//! such columns in `docs/03-modelo-de-datos.md` §3.1; those rules belong to `categorias` (V-02)
//! and are validated there.

use eo_domain::constants::limites;

use crate::dtos::tipos_movimiento::TipoMovimientoInput;
use crate::result::AppResult;
use crate::validation::Validator;

pub fn validate(input: &TipoMovimientoInput) -> AppResult<()> {
    let mut v = Validator::new();
    v.required_text(
        "nombre",
        &input.nombre,
        "Validation.TipoMovimiento.NombreRequired",
    );
    v.max_length(
        "nombre",
        &input.nombre,
        limites::NOMBRE_CORTO,
        "Validation.TipoMovimiento.NombreMaxLength",
    );
    v.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppError;

    fn input(nombre: &str) -> TipoMovimientoInput {
        TipoMovimientoInput {
            nombre: nombre.to_owned(),
            descripcion: None,
            es_ingreso: true,
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
    fn a_name_is_required() {
        let error = validate(&input("   ")).unwrap_err();
        assert_eq!(keys(error), ["Validation.TipoMovimiento.NombreRequired"]);
    }

    #[test]
    fn a_name_of_a_hundred_characters_is_accepted_and_one_more_is_not() {
        assert!(validate(&input(&"a".repeat(100))).is_ok());
        let error = validate(&input(&"a".repeat(101))).unwrap_err();
        assert_eq!(keys(error), ["Validation.TipoMovimiento.NombreMaxLength"]);
    }

    #[test]
    fn the_limit_travels_as_a_parameter_so_the_translation_does_not_hardcode_it() {
        let error = validate(&input(&"a".repeat(101))).unwrap_err();
        assert_eq!(
            error.fields()[0].params.get("max").map(String::as_str),
            Some("100")
        );
    }
}
