//! V-02 of `docs/07-validaciones.md`.

use eo_domain::constants::limites;
use uuid::Uuid;

use crate::dtos::categorias::CategoriaInput;
use crate::error::FieldError;
use crate::result::AppResult;
use crate::validation::Validator;

/// `#RRGGBB`. Checked by hand rather than with a regex crate: the shape is fixed and this reads
/// as the rule it enforces.
pub fn es_color_hex(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 7 && bytes[0] == b'#' && bytes[1..].iter().all(|b| b.is_ascii_hexdigit())
}

/// `id` is the category being edited, so a create passes `None`.
pub fn validate(input: &CategoriaInput, id: Option<Uuid>) -> AppResult<()> {
    let mut v = Validator::new();

    v.required_text(
        "nombre",
        &input.nombre,
        "Validation.Categoria.NombreRequired",
    );
    v.max_length(
        "nombre",
        &input.nombre,
        limites::NOMBRE_CORTO,
        "Validation.Categoria.NombreMaxLength",
    );
    v.max_length_opt(
        "descripcion",
        input.descripcion.as_deref(),
        limites::DESCRIPCION,
        "Validation.Categoria.DescripcionMaxLength",
    );

    if let Some(color) = input.color_hex.as_deref().filter(|c| !c.trim().is_empty()) {
        v.require(
            es_color_hex(color.trim()),
            FieldError::new("colorHex", "Validation.Categoria.ColorInvalid"),
        );
    }

    v.max_length_opt(
        "icono",
        input.icono.as_deref(),
        limites::ICONO,
        "Validation.Categoria.IconoMaxLength",
    );

    // Only the one-level case is visible from here; a longer cycle (A → B → A) needs the
    // ancestor chain and is checked as a business rule.
    if let (Some(id), Some(padre)) = (id, input.categoria_padre_id) {
        v.require(
            id != padre,
            FieldError::new("categoriaPadreId", "Validation.Categoria.PadreCiclico"),
        );
    }

    v.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppError;

    fn input() -> CategoriaInput {
        CategoriaInput {
            nombre: "Materiales".into(),
            descripcion: None,
            color_hex: None,
            icono: None,
            categoria_padre_id: None,
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
    fn el_nombre_es_obligatorio() {
        let error = validate(
            &CategoriaInput {
                nombre: "  ".into(),
                ..input()
            },
            None,
        )
        .unwrap_err();
        assert_eq!(keys(error), ["Validation.Categoria.NombreRequired"]);
    }

    #[test]
    fn el_color_acepta_solo_seis_digitos_hexadecimales_con_numeral() {
        for valido in ["#FFAA00", "#000000", "#abcdef"] {
            let dto = CategoriaInput {
                color_hex: Some(valido.into()),
                ..input()
            };
            assert!(validate(&dto, None).is_ok(), "{valido}");
        }
        for invalido in ["FFAA00", "#FFF", "#GGGGGG", "#FFAA000"] {
            let dto = CategoriaInput {
                color_hex: Some(invalido.into()),
                ..input()
            };
            assert_eq!(
                keys(validate(&dto, None).unwrap_err()),
                ["Validation.Categoria.ColorInvalid"],
                "{invalido}"
            );
        }
    }

    #[test]
    fn un_color_vacio_es_ausencia_de_color_y_no_un_error() {
        let dto = CategoriaInput {
            color_hex: Some("   ".into()),
            ..input()
        };
        assert!(validate(&dto, None).is_ok());
    }

    #[test]
    fn una_categoria_no_puede_ser_su_propio_padre() {
        let id = Uuid::from_u128(9);
        let dto = CategoriaInput {
            categoria_padre_id: Some(id),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto, Some(id)).unwrap_err()),
            ["Validation.Categoria.PadreCiclico"]
        );
    }

    #[test]
    fn se_informan_todos_los_problemas_de_una_vez() {
        // One submit, every field marked: the alternative is fixing them one round trip at a time.
        let dto = CategoriaInput {
            nombre: String::new(),
            color_hex: Some("rojo".into()),
            icono: Some("x".repeat(51)),
            ..input()
        };
        assert_eq!(keys(validate(&dto, None).unwrap_err()).len(), 3);
    }
}
