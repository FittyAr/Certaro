//! Validation primitives. See `docs/07-validaciones.md`.
//!
//! Every rule produces a `FieldError` with an i18n key; no rule produces a sentence.

pub mod categorias;
pub mod certificados;
pub mod clientes;
pub mod facturas;
pub mod movimientos;
pub mod obras;
pub mod ordenes_trabajo;
pub mod tipos_movimiento;
pub mod trabajos;

use crate::error::{AppError, FieldError};

/// A pragmatic shape check: one `@`, something on each side, and a dot in the domain.
///
/// Deliberately not RFC 5322. A stricter rule rejects addresses that work, and a looser one lets
/// through what is obviously a typo; this catches the typo and gets out of the way.
#[must_use]
pub fn es_email(value: &str) -> bool {
    let value = value.trim();
    let Some((local, dominio)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !dominio.starts_with('.')
        && !dominio.ends_with('.')
        && dominio.contains('.')
        && !dominio.contains('@')
        && !value.contains(char::is_whitespace)
}

/// `XX-XXXXXXXX-X`. The hyphens are required: the legacy data has them and a bare eleven-digit
/// string is almost always a CUIT pasted from somewhere that needs reformatting.
#[must_use]
pub fn es_cuit(value: &str) -> bool {
    let bytes = value.trim().as_bytes();
    bytes.len() == 13
        && bytes[2] == b'-'
        && bytes[11] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 2 || i == 11 || b.is_ascii_digit())
}

/// Accumulates field errors so one submit reports every problem at once instead of making the
/// user fix them one round-trip at a time.
#[derive(Debug, Default)]
pub struct Validator {
    errors: Vec<FieldError>,
}

impl Validator {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, error: FieldError) -> &mut Self {
        self.errors.push(error);
        self
    }

    /// Adds `error` when `condition` does **not** hold.
    pub fn require(&mut self, condition: bool, error: FieldError) -> &mut Self {
        if !condition {
            self.errors.push(error);
        }
        self
    }

    pub fn required_text(&mut self, field: &str, value: &str, key: &str) -> &mut Self {
        self.require(!value.trim().is_empty(), FieldError::new(field, key))
    }

    pub fn max_length(&mut self, field: &str, value: &str, max: usize, key: &str) -> &mut Self {
        // Counted in characters, not bytes: an accented name must not be rejected for being long.
        self.require(
            value.chars().count() <= max,
            FieldError::new(field, key).with_param("max", max),
        )
    }

    pub fn max_length_opt(
        &mut self,
        field: &str,
        value: Option<&str>,
        max: usize,
        key: &str,
    ) -> &mut Self {
        match value {
            Some(v) => self.max_length(field, v, max, key),
            None => self,
        }
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Consumes the validator: `Ok(())` when clean, `AppError::Validation` otherwise.
    pub fn finish(self) -> Result<(), AppError> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::Validation(self.errors))
        }
    }
}
