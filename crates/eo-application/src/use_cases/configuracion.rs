//! Use cases of configuration. See `docs/11-contratos-tauri.md` §5.13 and `docs/14` §2.
//!
//! Changes arrive as dotted keys with textual values, which is what a settings form produces. Each
//! one is looked up in the current configuration: **a key that does not exist is a validation
//! error**, never a silent no-op. The legacy `GetValue("Application:Settlement:Multiplier…")`
//! returned the default when the key was misspelled, so a typo looked like it worked.

use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::Value;
use tracing::info;

use crate::config::AppConfig;
use crate::error::{AppError, FieldError};
use crate::ports::SettingsStore;
use crate::result::AppResult;

pub type Cambios = BTreeMap<String, String>;

pub struct ConfiguracionService {
    settings: Arc<dyn SettingsStore>,
}

impl ConfiguracionService {
    pub fn new(settings: Arc<dyn SettingsStore>) -> Self {
        Self { settings }
    }

    #[must_use]
    pub fn get_all(&self) -> AppConfig {
        self.settings.snapshot()
    }

    /// Applies only the keys that changed and returns the configuration as it ended up.
    pub async fn set(&self, cambios: Cambios) -> AppResult<AppConfig> {
        if cambios.is_empty() {
            return Ok(self.settings.snapshot());
        }
        let actualizada = aplicar(&self.settings.snapshot(), &cambios)?;
        self.settings.save(actualizada.clone()).await?;
        info!(claves = ?cambios.keys().collect::<Vec<_>>(), "configuración actualizada");
        Ok(actualizada)
    }

    /// Puts the listed keys back to their compiled default, leaving the rest alone.
    pub async fn reset(&self, claves: Vec<String>) -> AppResult<AppConfig> {
        if claves.is_empty() {
            return Ok(self.settings.snapshot());
        }
        let actual = self.settings.snapshot();
        let defaults = to_value(&AppConfig::default())?;

        let mut cambios = Cambios::new();
        for clave in &claves {
            let valor = leer(&defaults, clave)
                .ok_or_else(|| desconocida(clave))?
                .clone();
            cambios.insert(clave.clone(), texto_de(&valor));
        }

        let actualizada = aplicar(&actual, &cambios)?;
        self.settings.save(actualizada.clone()).await?;
        info!(claves = ?claves, "configuración restablecida");
        Ok(actualizada)
    }
}

/// The configuration with `cambios` applied, validated as a whole.
///
/// Pure so that every rule below is testable without a store or a disk.
pub fn aplicar(base: &AppConfig, cambios: &Cambios) -> AppResult<AppConfig> {
    let mut documento = to_value(base)?;
    let mut errores = Vec::new();

    for (clave, texto) in cambios {
        let Some(actual) = leer(&documento, clave) else {
            errores.push(FieldError::new(clave.clone(), "Validation.Config.ClaveDesconocida"));
            continue;
        };
        match convertir(actual, texto) {
            // The type of the existing value is the type of the key: a numeric setting stays
            // numeric, so `"abc"` is refused here instead of at deserialisation with a message
            // nobody can act on.
            Some(valor) => escribir(&mut documento, clave, valor),
            None => errores.push(
                FieldError::new(clave.clone(), "Validation.Config.ValorInvalido")
                    .with_param("valor", texto),
            ),
        }
    }

    if !errores.is_empty() {
        return Err(AppError::Validation(errores));
    }

    let actualizada: AppConfig = serde_json::from_value(documento).map_err(|e| {
        AppError::Validation(vec![FieldError::new(
            "config",
            "Validation.Config.ValorInvalido",
        )
        .with_param("detalle", e.to_string())])
    })?;
    actualizada.validate()?;
    Ok(actualizada)
}

fn to_value(config: &AppConfig) -> AppResult<Value> {
    serde_json::to_value(config)
        .map_err(|e| AppError::unexpected(anyhow::anyhow!("config serialize: {e}")))
}

/// Walks a dotted path. Only objects are traversed: an index into an array is not a settings key.
fn leer<'v>(documento: &'v Value, clave: &str) -> Option<&'v Value> {
    clave
        .split('.')
        .try_fold(documento, |actual, segmento| actual.get(segmento))
}

fn escribir(documento: &mut Value, clave: &str, valor: Value) {
    let mut actual = documento;
    let segmentos: Vec<&str> = clave.split('.').collect();
    let Some((ultimo, previos)) = segmentos.split_last() else {
        return;
    };
    for segmento in previos {
        actual = match actual.get_mut(*segmento) {
            Some(siguiente) => siguiente,
            // Unreachable: `leer` already proved the path exists.
            None => return,
        };
    }
    if let Some(destino) = actual.get_mut(*ultimo) {
        *destino = valor;
    }
}

/// Parses `texto` into the same JSON type the current value has.
fn convertir(actual: &Value, texto: &str) -> Option<Value> {
    match actual {
        Value::String(_) => Some(Value::String(texto.to_owned())),
        Value::Bool(_) => match texto {
            "true" => Some(Value::Bool(true)),
            "false" => Some(Value::Bool(false)),
            _ => None,
        },
        Value::Number(numero) if numero.is_f64() => {
            texto.parse::<f64>().ok().and_then(serde_json::Number::from_f64).map(Value::Number)
        }
        Value::Number(_) => texto.parse::<i64>().ok().map(Value::from),
        // A list arrives as JSON, which is unambiguous about quoting and about an empty list.
        Value::Array(_) => serde_json::from_str::<Vec<Value>>(texto).ok().map(Value::Array),
        Value::Null => Some(Value::String(texto.to_owned())),
        Value::Object(_) => None,
    }
}

/// The textual form of a value, as `set` expects to receive it.
fn texto_de(valor: &Value) -> String {
    match valor {
        Value::String(s) => s.clone(),
        otro => otro.to_string(),
    }
}

fn desconocida(clave: &str) -> AppError {
    AppError::Validation(vec![FieldError::new(
        clave.to_owned(),
        "Validation.Config.ClaveDesconocida",
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cambios(pares: &[(&str, &str)]) -> Cambios {
        pares
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    #[test]
    fn una_clave_de_texto_se_aplica() {
        let config = aplicar(
            &AppConfig::default(),
            &cambios(&[("locale.language", "en")]),
        )
        .unwrap();
        assert_eq!(config.locale.language, "en");
    }

    #[test]
    fn una_clave_numerica_se_convierte() {
        let config = aplicar(
            &AppConfig::default(),
            &cambios(&[("attachments.maxSizeMb", "50")]),
        )
        .unwrap();
        assert_eq!(config.attachments.max_size_mb, 50);
    }

    #[test]
    fn una_clave_booleana_solo_acepta_true_o_false() {
        assert!(aplicar(&AppConfig::default(), &cambios(&[("backup.enabled", "false")])).is_ok());
        let error =
            aplicar(&AppConfig::default(), &cambios(&[("backup.enabled", "no")])).unwrap_err();
        assert!(matches!(error, AppError::Validation(_)));
    }

    #[test]
    fn una_clave_desconocida_es_un_error_y_no_se_ignora() {
        // This is the legacy behaviour being reversed: a misspelled key used to return the default.
        let error = aplicar(
            &AppConfig::default(),
            &cambios(&[("settlement.multiplierSaturday", "1.5")]),
        )
        .unwrap_err();
        match error {
            AppError::Validation(errores) => {
                assert_eq!(errores[0].message_key, "Validation.Config.ClaveDesconocida");
                assert_eq!(errores[0].field, "settlement.multiplierSaturday");
            }
            otro => panic!("se esperaba validación, vino {otro:?}"),
        }
    }

    #[test]
    fn un_valor_del_tipo_equivocado_se_rechaza() {
        let error = aplicar(
            &AppConfig::default(),
            &cambios(&[("attachments.maxSizeMb", "grande")]),
        )
        .unwrap_err();
        assert!(matches!(error, AppError::Validation(_)));
    }

    #[test]
    fn los_errores_de_varias_claves_vienen_juntos() {
        let error = aplicar(
            &AppConfig::default(),
            &cambios(&[("no.existe", "1"), ("attachments.maxSizeMb", "x")]),
        )
        .unwrap_err();
        match error {
            AppError::Validation(errores) => assert_eq!(errores.len(), 2),
            otro => panic!("{otro:?}"),
        }
    }

    #[test]
    fn una_lista_se_manda_como_json() {
        let config = aplicar(
            &AppConfig::default(),
            &cambios(&[("dashboard.casasDolar", "[\"oficial\"]")]),
        )
        .unwrap();
        assert_eq!(config.dashboard.casas_dolar, vec!["oficial".to_owned()]);
    }

    #[test]
    fn un_cambio_que_rompe_una_regla_de_negocio_no_pasa() {
        // `maxSizeMb` above `maxTotalMb` is refused by `AppConfig::validate`, and that check runs
        // on the merged result rather than key by key.
        let error = aplicar(
            &AppConfig::default(),
            &cambios(&[("attachments.maxSizeMb", "5000")]),
        )
        .unwrap_err();
        assert!(matches!(error, AppError::Validation(_)));
    }

    #[test]
    fn una_seccion_entera_no_es_una_clave() {
        let error = aplicar(&AppConfig::default(), &cambios(&[("locale", "en")])).unwrap_err();
        assert!(matches!(error, AppError::Validation(_)));
    }

    #[test]
    fn el_texto_de_un_valor_no_agrega_comillas_a_las_cadenas() {
        assert_eq!(texto_de(&Value::String("es".into())), "es");
        assert_eq!(texto_de(&Value::from(30)), "30");
        assert_eq!(texto_de(&Value::Bool(true)), "true");
    }
}
