//! Errors carry i18n keys, never translated text, and internal causes never reach the user.
//! See `docs/02-arquitectura.md` §6.

use certaro_application::error::FieldError;
use certaro_application::validation::Validator;
use certaro_application::AppError;
use certaro_domain::DomainError;
use pretty_assertions::assert_eq;

#[test]
fn cada_variante_tiene_codigo_y_clave() {
    let cases: Vec<(AppError, &str, &str)> = vec![
        (
            AppError::Validation(vec![]),
            "VALIDATION",
            "Error.Validation",
        ),
        (
            AppError::not_found("Movimiento", "x"),
            "NOT_FOUND",
            "Error.NotFound",
        ),
        (
            AppError::conflict("OBRA_NUMERO", "Validation.Obra.NumeroDuplicado"),
            "CONFLICT",
            "Validation.Obra.NumeroDuplicado",
        ),
        (
            AppError::Concurrency { entity: "Factura" },
            "CONCURRENCY",
            "Error.Concurrency",
        ),
        (
            AppError::dependency_in_use("CLIENTE_CON_OBRAS", "Error.ClienteConObras"),
            "DEPENDENCY_IN_USE",
            "Error.ClienteConObras",
        ),
        (
            AppError::Domain(DomainError::MoneyOverflow),
            "DOMAIN",
            "Error.Domain",
        ),
        (
            AppError::ExternalUnavailable { service: "dolar" },
            "EXTERNAL_UNAVAILABLE",
            "Error.ExternalUnavailable",
        ),
    ];

    for (error, code, key) in cases {
        assert_eq!(error.code(), code);
        assert_eq!(error.message_key(), key);
    }
}

#[test]
fn los_errores_internos_estan_marcados() {
    assert!(AppError::persistence(anyhow::anyhow!("connection refused")).is_internal());
    assert!(AppError::io(anyhow::anyhow!("disk full")).is_internal());
    assert!(AppError::unexpected(anyhow::anyhow!("boom")).is_internal());

    assert!(!AppError::not_found("Cliente", "x").is_internal());
    assert!(!AppError::Validation(vec![]).is_internal());
}

#[test]
fn not_found_lleva_la_entidad_y_el_id_como_parametros() {
    let error = AppError::not_found("Movimiento", "0192f3a1");
    let params = error.params();
    assert_eq!(params.get("entity").map(String::as_str), Some("Movimiento"));
    assert_eq!(params.get("id").map(String::as_str), Some("0192f3a1"));
}

#[test]
fn solo_validation_expone_campos() {
    let validation = AppError::Validation(vec![FieldError::new(
        "concepto",
        "Validation.Movimiento.ConceptoRequired",
    )]);
    assert_eq!(validation.fields().len(), 1);
    assert_eq!(AppError::not_found("Cliente", "x").fields().len(), 0);
}

#[test]
fn el_validador_acumula_todos_los_problemas_de_una_vez() {
    let mut v = Validator::new();
    v.required_text("concepto", "  ", "Validation.Movimiento.ConceptoRequired");
    v.max_length(
        "concepto",
        &"x".repeat(501),
        500,
        "Validation.Movimiento.ConceptoMaxLength",
    );
    v.require(
        false,
        FieldError::new("monto", "Validation.Movimiento.MontoRequired"),
    );

    let err = v.finish().expect_err("three problems");
    match err {
        // One submit reports everything, instead of making the user fix them one round-trip at a
        // time.
        AppError::Validation(fields) => assert_eq!(fields.len(), 3),
        other => panic!("expected a validation error, got {other:?}"),
    }
}

#[test]
fn el_validador_limpio_devuelve_ok() {
    let mut v = Validator::new();
    v.required_text(
        "concepto",
        "Compra de cable",
        "Validation.Movimiento.ConceptoRequired",
    );
    v.max_length_opt(
        "observaciones",
        None,
        500,
        "Validation.Movimiento.ObservacionesMaxLength",
    );
    assert!(v.is_valid());
    assert!(v.finish().is_ok());
}

#[test]
fn la_longitud_maxima_se_cuenta_en_caracteres_no_en_bytes() {
    // "ñ" is two bytes; a name of five accented characters must not be rejected as ten.
    let mut v = Validator::new();
    v.max_length("nombre", "ññññañ", 6, "Validation.Cliente.NombreMaxLength");
    assert!(v.is_valid());
}

#[test]
fn los_parametros_del_campo_viajan_con_el_error() {
    let field = FieldError::new("concepto", "Validation.Movimiento.ConceptoMaxLength")
        .with_param("max", 500);
    assert_eq!(field.params.get("max").map(String::as_str), Some("500"));
}
