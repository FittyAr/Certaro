//! Text sanitization. See `docs/15-migracion-de-datos.md` §3.8.

/// Trims leading and trailing whitespace. Returns `None` if the result is empty and the column
/// is nullable.
#[must_use]
pub fn trim_or_null(raw: &str, nullable: bool) -> Option<String> {
    let trimmed = raw.trim().to_owned();
    if trimmed.is_empty() && nullable {
        None
    } else {
        Some(trimmed)
    }
}

/// Trims and returns `Some` always (for non-nullable columns).
#[must_use]
pub fn trim(raw: &str) -> String {
    raw.trim().to_owned()
}

/// Normalizes a CUIT by removing dashes and dots.
#[must_use]
pub fn normalize_cuit(raw: &str) -> String {
    raw.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Validates a hex color against `^#[0-9A-Fa-f]{6}$`. Returns `None` if invalid.
#[must_use]
pub fn validate_color_hex(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.len() == 7
        && trimmed.starts_with('#')
        && trimmed[1..].chars().all(|c| c.is_ascii_hexdigit())
    {
        Some(trimmed.to_owned())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_or_null_devuelve_none_para_vacio_nullable() {
        assert_eq!(trim_or_null("  ", true), None);
        assert_eq!(trim_or_null("", true), None);
    }

    #[test]
    fn trim_or_null_conserva_texto_no_vacio() {
        assert_eq!(trim_or_null("  hola  ", true), Some("hola".to_owned()));
    }

    #[test]
    fn cuit_se_normaliza() {
        assert_eq!(normalize_cuit("20-12345678-9"), "20123456789");
        assert_eq!(normalize_cuit("20.123.456.78-9"), "20123456789");
    }

    #[test]
    fn color_hex_valido() {
        assert_eq!(validate_color_hex("#FF00AA"), Some("#FF00AA".to_owned()));
        assert_eq!(validate_color_hex("#ff00aa"), Some("#ff00aa".to_owned()));
    }

    #[test]
    fn color_hex_invalido() {
        assert_eq!(validate_color_hex("FF00AA"), None);
        assert_eq!(validate_color_hex("#GG0000"), None);
        assert_eq!(validate_color_hex("#FFF"), None);
    }
}
