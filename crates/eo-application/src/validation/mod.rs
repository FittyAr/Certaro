//! Validation primitives. See `docs/07-validaciones.md`.
//!
//! Every rule produces a `FieldError` with an i18n key; no rule produces a sentence.

pub mod categorias;
pub mod movimientos;
pub mod tipos_movimiento;

use crate::error::{AppError, FieldError};

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
