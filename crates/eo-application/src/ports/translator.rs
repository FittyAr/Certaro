//! Resolution of i18n keys outside the interface. See `docs/12-reportes-y-exportaciones.md` §1.2.
//!
//! The reports need translated labels, and the reports run in the backend. Without this port the
//! generators would carry Spanish literals, which is exactly what the legacy system did: every
//! label of every report was hardcoded, so the English build printed Spanish paperwork.

use std::collections::HashMap;

/// Resolves a key into text in the configured language.
///
/// A missing key returns the key itself rather than an empty string or a panic: a label that reads
/// `Report.Col.Fecha` on a PDF is an obvious defect, while a blank column heading is not.
pub trait Translator: Send + Sync {
    /// The language this instance resolves, as its two-letter code.
    fn language(&self) -> &str;

    fn text(&self, key: &str) -> String;

    /// Same, substituting `{name}` placeholders. Only named parameters exist (doc 14 §4).
    fn format(&self, key: &str, params: &[(&str, &str)]) -> String {
        let mut text = self.text(key);
        for (name, value) in params {
            text = text.replace(&format!("{{{name}}}"), value);
        }
        text
    }
}

/// A translator backed by a map, for tests and for callers that only need a handful of keys.
#[derive(Debug, Clone, Default)]
pub struct MapTranslator {
    language: String,
    entries: HashMap<String, String>,
}

impl MapTranslator {
    #[must_use]
    pub fn new(language: impl Into<String>, entries: HashMap<String, String>) -> Self {
        Self {
            language: language.into(),
            entries,
        }
    }
}

impl Translator for MapTranslator {
    fn language(&self) -> &str {
        &self.language
    }

    fn text(&self, key: &str) -> String {
        self.entries
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn translator() -> MapTranslator {
        MapTranslator::new(
            "es",
            HashMap::from([
                ("Report.Col.Fecha".to_owned(), "Fecha".to_owned()),
                (
                    "Report.Footer.Page".to_owned(),
                    "Página {actual} de {total}".to_owned(),
                ),
            ]),
        )
    }

    #[test]
    fn una_clave_conocida_se_resuelve() {
        assert_eq!(translator().text("Report.Col.Fecha"), "Fecha");
    }

    #[test]
    fn una_clave_faltante_devuelve_la_clave_y_no_una_cadena_vacia() {
        assert_eq!(translator().text("Report.Col.Nada"), "Report.Col.Nada");
    }

    #[test]
    fn los_parametros_con_nombre_se_sustituyen() {
        let texto = translator().format("Report.Footer.Page", &[("actual", "2"), ("total", "7")]);
        assert_eq!(texto, "Página 2 de 7");
    }

    #[test]
    fn un_parametro_que_no_esta_en_el_texto_no_molesta() {
        let texto = translator().format("Report.Col.Fecha", &[("actual", "2")]);
        assert_eq!(texto, "Fecha");
    }
}
