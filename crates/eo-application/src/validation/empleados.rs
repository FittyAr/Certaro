//! V-13 of `docs/07-validaciones.md`.

use eo_domain::constants::limites;

use crate::dtos::empleados::EmpleadoInput;
use crate::error::FieldError;
use crate::result::AppResult;
use crate::validation::{es_email, Validator};

/// Digits only, with dots and spaces stripped: a document pasted as `20.123.456` is the same
/// document.
#[must_use]
pub fn normalizar_dni(dni: &str) -> String {
    dni.chars().filter(char::is_ascii_digit).collect()
}

pub fn validate(input: &EmpleadoInput) -> AppResult<()> {
    let mut v = Validator::new();

    v.required_text(
        "nombre",
        &input.nombre,
        "Validation.Empleado.NombreRequired",
    );
    v.max_length(
        "nombre",
        &input.nombre,
        limites::NOMBRE_LARGO,
        "Validation.Empleado.NombreMaxLength",
    );

    // The document is optional: the legacy data has employees without one, and refusing to record
    // them would mean not paying them.
    if let Some(dni) = input
        .dni
        .as_deref()
        .map(str::trim)
        .filter(|d| !d.is_empty())
    {
        let digitos = normalizar_dni(dni);
        v.require(
            digitos.chars().count() == dni.chars().filter(|c| !matches!(c, '.' | ' ')).count(),
            FieldError::new("dni", "Validation.Empleado.DniFormat"),
        );
        v.require(
            (7..=9).contains(&digitos.chars().count()),
            FieldError::new("dni", "Validation.Empleado.DniLength"),
        );
    }

    v.max_length_opt(
        "cargo",
        input.cargo.as_deref(),
        limites::NOMBRE_CORTO,
        "Validation.Empleado.CargoMaxLength",
    );

    if let Some(email) = input
        .email
        .as_deref()
        .map(str::trim)
        .filter(|e| !e.is_empty())
    {
        v.require(
            es_email(email),
            FieldError::new("email", "Validation.Empleado.EmailInvalid"),
        );
        v.max_length(
            "email",
            email,
            limites::EMAIL,
            "Validation.Empleado.EmailMaxLength",
        );
    }

    v.max_length_opt(
        "telefono",
        input.telefono.as_deref(),
        limites::TELEFONO,
        "Validation.Empleado.TelefonoMaxLength",
    );

    v.require(
        !input.tarifa_diaria.is_negative(),
        FieldError::new("tarifaDiaria", "Validation.Empleado.TarifaNegative"),
    );
    v.require(
        !input.sueldo_base.is_negative(),
        FieldError::new("sueldoBase", "Validation.Empleado.SueldoNegative"),
    );

    // Zero and values below one are legitimate: a multiplier is a business decision, not a rate
    // with a floor. Only a negative one is nonsense.
    for (campo, valor) in [
        ("multiplicadorSabado", input.multiplicador_sabado),
        ("multiplicadorDomingo", input.multiplicador_domingo),
        ("multiplicadorFeriado", input.multiplicador_feriado),
    ] {
        v.require(
            !valor.is_negative(),
            FieldError::new(campo, "Validation.Empleado.MultiplicadorNegative"),
        );
    }

    if let Some(egreso) = input.fecha_egreso {
        v.require(
            egreso >= input.fecha_ingreso,
            FieldError::new("fechaEgreso", "Validation.Empleado.FechaEgresoInvalid"),
        );
    }

    v.finish()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use eo_domain::{Decimal4, FrecuenciaPago, Money};

    use super::*;
    use crate::AppError;

    fn dia(d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 3, d).unwrap()
    }

    fn input() -> EmpleadoInput {
        EmpleadoInput {
            nombre: "Pablo Báez".into(),
            dni: None,
            cargo: Some("Oficial".into()),
            sueldo_base: Money::parse("1200000").unwrap(),
            pago_frecuencia: FrecuenciaPago::Mensual,
            tarifa_diaria: Money::parse("40000").unwrap(),
            multiplicador_sabado: Decimal4::parse("1.5").unwrap(),
            multiplicador_domingo: Decimal4::from_units(2).unwrap(),
            multiplicador_feriado: Decimal4::from_units(2).unwrap(),
            email: None,
            telefono: None,
            fecha_ingreso: dia(1),
            fecha_egreso: None,
            activo: true,
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
    fn un_empleado_minimo_es_valido() {
        assert!(validate(&input()).is_ok());
    }

    #[test]
    fn el_documento_es_opcional() {
        let dto = EmpleadoInput {
            dni: None,
            ..input()
        };
        assert!(validate(&dto).is_ok());
    }

    #[test]
    fn un_documento_con_puntos_se_normaliza() {
        let dto = EmpleadoInput {
            dni: Some("20.123.456".into()),
            ..input()
        };
        assert!(validate(&dto).is_ok());
    }

    #[test]
    fn un_documento_corto_no_es_valido() {
        let dto = EmpleadoInput {
            dni: Some("123456".into()),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.Empleado.DniLength"]
        );
    }

    #[test]
    fn un_documento_con_letras_no_es_valido() {
        let dto = EmpleadoInput {
            dni: Some("20A23456".into()),
            ..input()
        };
        assert!(keys(validate(&dto).unwrap_err()).contains(&"Validation.Empleado.DniFormat".into()));
    }

    #[test]
    fn los_multiplicadores_en_cero_son_validos() {
        let dto = EmpleadoInput {
            multiplicador_sabado: Decimal4::ZERO,
            multiplicador_domingo: Decimal4::ZERO,
            multiplicador_feriado: Decimal4::ZERO,
            ..input()
        };
        assert!(validate(&dto).is_ok());
    }

    #[test]
    fn un_multiplicador_negativo_no_es_valido() {
        let dto = EmpleadoInput {
            multiplicador_sabado: Decimal4::parse("-1").unwrap(),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.Empleado.MultiplicadorNegative"]
        );
    }

    #[test]
    fn el_egreso_no_puede_preceder_al_ingreso() {
        let dto = EmpleadoInput {
            fecha_egreso: Some(dia(1) - chrono::Duration::days(1)),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.Empleado.FechaEgresoInvalid"]
        );
    }

    #[test]
    fn una_tarifa_negativa_no_es_valida() {
        let dto = EmpleadoInput {
            tarifa_diaria: Money::parse("-1").unwrap(),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.Empleado.TarifaNegative"]
        );
    }

    #[test]
    fn un_email_mal_formado_no_es_valido() {
        let dto = EmpleadoInput {
            email: Some("pablo@".into()),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.Empleado.EmailInvalid"]
        );
    }
}
