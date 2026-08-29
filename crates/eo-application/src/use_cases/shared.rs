//! Pieces every use case repeats: closing a transaction, parsing a row version, normalising an
//! optional text and validating a sort field.
//!
//! They live here rather than being copied per module so that a fix to, say, the rollback of a
//! failed read applies everywhere at once.

use eo_domain::RowVersion;

use crate::error::{AppError, FieldError};
use crate::ports::Transaction;
use crate::result::AppResult;

/// Closes a read-only transaction. Reads are wrapped too, so a listing that spans several queries
/// sees one consistent snapshot.
pub async fn finish_read<T>(tx: Box<dyn Transaction>, outcome: AppResult<T>) -> AppResult<T> {
    let rolled_back = tx.rollback().await;
    match outcome {
        Ok(value) => rolled_back.map(|()| value),
        // The original failure is what the user needs to see; a rollback error on top of it would
        // only hide the cause.
        Err(e) => Err(e),
    }
}

pub async fn finish_write<T>(tx: Box<dyn Transaction>, outcome: AppResult<T>) -> AppResult<T> {
    match outcome {
        Ok(value) => tx.commit().await.map(|()| value),
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

pub fn parse_row_version(raw: &str) -> AppResult<RowVersion> {
    RowVersion::parse_hex(raw).map_err(AppError::from)
}

/// An empty optional text is `None`, not `Some("")`: two ways of saying nothing make every later
/// comparison ambiguous.
pub fn normalise(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

/// Rejects any sort field outside the module's closed list. The value comes from the frontend and
/// ends up in an `ORDER BY`, so it is matched against a list instead of being escaped.
pub fn checked_sort<'a>(sort_by: Option<&'a str>, allowed: &[&str]) -> AppResult<Option<&'a str>> {
    match sort_by {
        None => Ok(None),
        Some(field) if allowed.contains(&field) => Ok(Some(field)),
        Some(_) => Err(AppError::Validation(vec![FieldError::new(
            "sortBy",
            "Validation.Common.SortByNotAllowed",
        )])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_campo_de_orden_desconocido_se_rechaza() {
        assert!(checked_sort(Some("fecha"), &["fecha"]).is_ok());
        assert!(checked_sort(None, &["fecha"]).unwrap().is_none());
        // Anything else would be interpolated into SQL.
        assert!(checked_sort(Some("fecha; DROP TABLE movimientos"), &["fecha"]).is_err());
    }

    #[test]
    fn un_texto_opcional_en_blanco_es_ausencia() {
        assert_eq!(normalise(Some("  ".into())), None);
        assert_eq!(normalise(Some(" hola ".into())), Some("hola".into()));
        assert_eq!(normalise(None), None);
    }
}
