//! V-04 and V-05 of `docs/07-validaciones.md`.

use certaro_domain::constants::limites;

use crate::dtos::clientes::{ClienteContactoInput, ClienteInput};
use crate::error::FieldError;
use crate::result::AppResult;
use crate::validation::{es_cuit, es_email, Validator};

pub fn validate(input: &ClienteInput) -> AppResult<()> {
    let mut v = Validator::new();

    v.required_text("nombre", &input.nombre, "Validation.Cliente.NombreRequired");
    v.max_length(
        "nombre",
        &input.nombre,
        limites::NOMBRE_LARGO,
        "Validation.Cliente.NombreMaxLength",
    );

    if let Some(email) = presente(input.email.as_deref()) {
        v.require(
            es_email(email),
            FieldError::new("email", "Validation.Cliente.EmailInvalid"),
        );
        v.max_length(
            "email",
            email,
            limites::EMAIL,
            "Validation.Cliente.EmailMaxLength",
        );
    }

    if let Some(cuit) = presente(input.cuit.as_deref()) {
        v.require(
            es_cuit(cuit),
            FieldError::new("cuit", "Validation.Cliente.CuitInvalid"),
        );
    }

    v.max_length_opt(
        "telefono",
        input.telefono.as_deref(),
        limites::TELEFONO,
        "Validation.Cliente.TelefonoMaxLength",
    );
    v.max_length_opt(
        "direccion",
        input.direccion.as_deref(),
        limites::DIRECCION,
        "Validation.Cliente.DireccionMaxLength",
    );
    v.max_length_opt(
        "condicionIva",
        input.condicion_iva.as_deref(),
        limites::NOMBRE_CORTO,
        "Validation.Cliente.CondicionIvaMaxLength",
    );

    for (i, contacto) in input.contactos.iter().enumerate() {
        validar_contacto(&mut v, i, contacto);
    }

    // RC-13: the main contact is the one the mail action preselects, so exactly one of them can
    // hold that role.
    let principales = input.contactos.iter().filter(|c| c.es_principal).count();
    v.require(
        principales <= 1,
        FieldError::new("contactos", "Validation.Cliente.ContactoPrincipalUnico")
            .with_param("count", principales),
    );

    // Two rows with the same address are the same contact typed twice, and the unique index would
    // reject them anyway with an error nobody can read.
    let mut vistos: Vec<String> = Vec::new();
    for (i, contacto) in input.contactos.iter().enumerate() {
        let email = contacto.email.trim().to_lowercase();
        if email.is_empty() {
            continue;
        }
        if vistos.contains(&email) {
            v.push(
                FieldError::new(
                    format!("contactos[{i}].email"),
                    "Validation.Cliente.ContactoEmailDuplicado",
                )
                .with_param("email", email.clone()),
            );
        }
        vistos.push(email);
    }

    v.finish()
}

fn validar_contacto(v: &mut Validator, i: usize, contacto: &ClienteContactoInput) {
    let campo = |name: &str| format!("contactos[{i}].{name}");

    v.required_text(
        &campo("etiqueta"),
        &contacto.etiqueta,
        "Validation.Cliente.ContactoEtiquetaRequired",
    );
    v.max_length(
        &campo("etiqueta"),
        &contacto.etiqueta,
        limites::NOMBRE_CORTO,
        "Validation.Cliente.ContactoEtiquetaMaxLength",
    );

    // The column is `NOT NULL` and the unique index is on `(cliente_id, email)`, so an empty
    // address is not "no contact information", it is a row that cannot be stored.
    v.required_text(
        &campo("email"),
        &contacto.email,
        "Validation.Cliente.ContactoEmailRequired",
    );
    if let Some(email) = presente(Some(&contacto.email)) {
        v.require(
            es_email(email),
            FieldError::new(campo("email"), "Validation.Cliente.ContactoEmailInvalid"),
        );
        v.max_length(
            &campo("email"),
            email,
            limites::EMAIL,
            "Validation.Cliente.ContactoEmailMaxLength",
        );
    }

    v.max_length_opt(
        &campo("nombre"),
        contacto.nombre.as_deref(),
        limites::NOMBRE_LARGO,
        "Validation.Cliente.ContactoNombreMaxLength",
    );
    v.max_length_opt(
        &campo("telefono"),
        contacto.telefono.as_deref(),
        limites::TELEFONO,
        "Validation.Cliente.ContactoTelefonoMaxLength",
    );
}

/// An optional text that is blank means absence, not a value to validate.
fn presente(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppError;

    fn contacto() -> ClienteContactoInput {
        ClienteContactoInput {
            id: None,
            etiqueta: "Oficina".into(),
            email: "proyecto@example.com".into(),
            nombre: None,
            telefono: None,
            es_principal: false,
        }
    }

    fn input() -> ClienteInput {
        ClienteInput {
            nombre: "Tecnocasa".into(),
            cuit: None,
            direccion: None,
            telefono: None,
            email: None,
            condicion_iva: None,
            contactos: vec![],
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
    fn un_cliente_minimo_es_valido() {
        assert!(validate(&input()).is_ok());
    }

    #[test]
    fn el_nombre_es_obligatorio() {
        let dto = ClienteInput {
            nombre: "   ".into(),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.Cliente.NombreRequired"]
        );
    }

    #[test]
    fn el_cuit_exige_los_guiones() {
        for valido in ["20-12345678-9", "30-71234567-0"] {
            let dto = ClienteInput {
                cuit: Some(valido.into()),
                ..input()
            };
            assert!(validate(&dto).is_ok(), "{valido}");
        }
        for invalido in [
            "20123456789",
            "20-1234567-9",
            "AB-12345678-9",
            "20-12345678",
        ] {
            let dto = ClienteInput {
                cuit: Some(invalido.into()),
                ..input()
            };
            assert_eq!(
                keys(validate(&dto).unwrap_err()),
                ["Validation.Cliente.CuitInvalid"],
                "{invalido}"
            );
        }
    }

    #[test]
    fn un_cuit_vacio_es_ausencia_y_no_un_error() {
        let dto = ClienteInput {
            cuit: Some("  ".into()),
            ..input()
        };
        assert!(validate(&dto).is_ok());
    }

    #[test]
    fn el_email_tiene_que_parecer_un_email() {
        for invalido in ["sin-arroba", "a@b", "@example.com", "a@ example.com"] {
            let dto = ClienteInput {
                email: Some(invalido.into()),
                ..input()
            };
            assert!(
                keys(validate(&dto).unwrap_err())
                    .contains(&"Validation.Cliente.EmailInvalid".to_owned()),
                "{invalido}"
            );
        }
    }

    #[test]
    fn no_puede_haber_dos_contactos_principales() {
        let dto = ClienteInput {
            contactos: vec![
                ClienteContactoInput {
                    es_principal: true,
                    ..contacto()
                },
                ClienteContactoInput {
                    email: "otro@example.com".into(),
                    es_principal: true,
                    ..contacto()
                },
            ],
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.Cliente.ContactoPrincipalUnico"]
        );
    }

    #[test]
    fn un_solo_principal_es_correcto() {
        let dto = ClienteInput {
            contactos: vec![
                ClienteContactoInput {
                    es_principal: true,
                    ..contacto()
                },
                ClienteContactoInput {
                    email: "otro@example.com".into(),
                    ..contacto()
                },
            ],
            ..input()
        };
        assert!(validate(&dto).is_ok());
    }

    #[test]
    fn dos_contactos_con_el_mismo_email_se_rechazan() {
        let dto = ClienteInput {
            contactos: vec![
                contacto(),
                ClienteContactoInput {
                    email: "PROYECTO@example.com".into(),
                    ..contacto()
                },
            ],
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.Cliente.ContactoEmailDuplicado"]
        );
    }

    #[test]
    fn los_errores_de_un_contacto_llevan_su_indice() {
        let dto = ClienteInput {
            contactos: vec![
                contacto(),
                ClienteContactoInput {
                    etiqueta: String::new(),
                    email: "roto".into(),
                    ..contacto()
                },
            ],
            ..input()
        };
        let error = validate(&dto).unwrap_err();
        let campos: Vec<_> = error.fields().iter().map(|f| f.field.clone()).collect();
        assert!(campos.contains(&"contactos[1].etiqueta".to_owned()));
        assert!(campos.contains(&"contactos[1].email".to_owned()));
        assert!(!campos.iter().any(|c| c.starts_with("contactos[0]")));
    }

    #[test]
    fn se_informan_todos_los_problemas_de_una_vez() {
        let dto = ClienteInput {
            nombre: String::new(),
            cuit: Some("nope".into()),
            email: Some("tampoco".into()),
            ..input()
        };
        assert_eq!(keys(validate(&dto).unwrap_err()).len(), 3);
    }
}
