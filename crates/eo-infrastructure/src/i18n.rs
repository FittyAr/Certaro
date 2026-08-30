//! The translator the reports use. See `docs/14-configuracion-e-i18n.md` §4.
//!
//! The locale files are the frontend's, embedded in the binary at compile time. One set of texts
//! for the whole application is the point: the legacy system had the screen labels in a resource
//! file and the report labels hardcoded in the generator, and the two drifted until the same
//! concept was called three different things in three places.
//!
//! `pnpm i18n:check` already verifies that both languages carry the same keys, so embedding them
//! also puts the report labels under that check.

use std::collections::HashMap;

use eo_application::ports::Translator;
use serde_json::Value;

const ES: &str = include_str!("../../../src/locales/es.json");
const EN: &str = include_str!("../../../src/locales/en.json");

/// Language of last resort. Spanish is the canonical file: it is the one that always has every key.
const FALLBACK: &str = "es";

pub struct JsonTranslator {
    language: String,
    entries: HashMap<String, String>,
    /// Spanish, consulted when the chosen language lacks a key. Empty when Spanish *is* the choice.
    fallback: HashMap<String, String>,
}

impl JsonTranslator {
    /// Builds a translator for a language code. An unknown code falls back to Spanish rather than
    /// failing: a misconfigured language must not stop a report from being generated.
    #[must_use]
    pub fn new(language: &str) -> Self {
        let code = match language.split(['-', '_']).next().unwrap_or(FALLBACK) {
            "en" => "en",
            _ => FALLBACK,
        };
        let entries = flatten_document(if code == "en" { EN } else { ES });
        let fallback = if code == FALLBACK {
            HashMap::new()
        } else {
            flatten_document(ES)
        };
        Self {
            language: code.to_owned(),
            entries,
            fallback,
        }
    }
}

impl Translator for JsonTranslator {
    fn language(&self) -> &str {
        &self.language
    }

    fn text(&self, key: &str) -> String {
        self.entries
            .get(key)
            .or_else(|| self.fallback.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_owned())
    }
}

/// Flattens the nested document into `{ "A.B.C": "text" }`. A malformed file yields no keys, and
/// every label then prints as its own key, which is visible without crashing the export.
fn flatten_document(raw: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    if let Ok(Value::Object(root)) = serde_json::from_str::<Value>(raw) {
        flatten_into(&Value::Object(root), "", &mut out);
    }
    out
}

fn flatten_into(node: &Value, prefix: &str, out: &mut HashMap<String, String>) {
    match node {
        Value::Object(map) => {
            for (key, value) in map {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten_into(value, &path, out);
            }
        }
        Value::String(text) => {
            out.insert(prefix.to_owned(), text.clone());
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn los_dos_idiomas_traen_las_claves_de_reportes() {
        for language in ["es", "en"] {
            let t = JsonTranslator::new(language);
            assert_ne!(
                t.text("Report.Col.Fecha"),
                "Report.Col.Fecha",
                "falta la clave en {language}"
            );
        }
    }

    #[test]
    fn cada_idioma_resuelve_en_su_propio_texto() {
        assert_eq!(JsonTranslator::new("es").text("General.Save"), "Guardar");
        assert_eq!(JsonTranslator::new("en").text("General.Save"), "Save");
    }

    #[test]
    fn un_idioma_desconocido_cae_en_castellano_sin_fallar() {
        let t = JsonTranslator::new("de");
        assert_eq!(t.language(), "es");
        assert_eq!(t.text("General.Save"), "Guardar");
    }

    #[test]
    fn una_variante_regional_se_reduce_al_idioma() {
        assert_eq!(JsonTranslator::new("en-US").language(), "en");
        assert_eq!(JsonTranslator::new("es_AR").language(), "es");
    }

    #[test]
    fn una_clave_inexistente_se_devuelve_tal_cual() {
        assert_eq!(JsonTranslator::new("es").text("No.Existe"), "No.Existe");
    }

    #[test]
    fn los_parametros_con_nombre_se_reemplazan() {
        let t = JsonTranslator::new("es");
        let texto = t.format("Report.Footer.Page", &[("actual", "3"), ("total", "9")]);
        assert!(texto.contains('3') && texto.contains('9'), "{texto}");
        assert!(!texto.contains('{'), "quedó un placeholder sin sustituir: {texto}");
    }
}
