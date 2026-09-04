use std::path::PathBuf;
use certaro_domain::{Decimal4, Money};
use serde::{Deserialize, Serialize};
use super::super::types::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ApplicationConfig {
    pub name: String,
    pub environment: Environment,
    pub seed_enabled: bool,
    pub last_page_size: u32,
    pub theme: ThemePreference,
    pub last_route: String,
    pub sidebar_expanded: bool,
    /// Only set for tests and portable installs; otherwise resolved from the operating system.
    pub data_dir: Option<PathBuf>,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            name: "Certaro".to_owned(),
            environment: Environment::Production,
            seed_enabled: false,
            last_page_size: 30,
            theme: ThemePreference::System,
            last_route: "dashboard".to_owned(),
            sidebar_expanded: true,
            data_dir: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LocaleConfig {
    pub language: String,
    pub formato_fecha: String,
    pub formato_fecha_hora: String,
    pub primer_dia_semana: u8,
    pub simbolo_moneda: String,
    pub separador_miles: String,
    pub separador_decimal: String,
    /// Decimals **shown**; storage is always four (doc 04).
    pub decimales_moneda: u8,
    pub decimales_porcentaje: u8,
    pub moneda_por_defecto: String,
    /// IANA name. Only affects presentation and the reading of civil dates (doc 04 §3.4).
    pub zona_horaria: String,
}

impl Default for LocaleConfig {
    fn default() -> Self {
        Self {
            language: "es".to_owned(),
            formato_fecha: "dd/MM/yyyy".to_owned(),
            formato_fecha_hora: "dd/MM/yyyy HH:mm".to_owned(),
            primer_dia_semana: 1,
            simbolo_moneda: "$".to_owned(),
            separador_miles: ".".to_owned(),
            separador_decimal: ",".to_owned(),
            decimales_moneda: 2,
            decimales_porcentaje: 2,
            moneda_por_defecto: "ars".to_owned(),
            zona_horaria: "America/Argentina/Buenos_Aires".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DiasPorFrecuencia {
    pub diario: Decimal4,
    /// Monday to Saturday.
    pub semanal: Decimal4,
    pub quincenal: Decimal4,
    pub mensual: Decimal4,
}

impl Default for DiasPorFrecuencia {
    fn default() -> Self {
        Self {
            diario: Decimal4::ONE,
            semanal: Decimal4::from_raw(60_000),
            quincenal: Decimal4::from_raw(150_000),
            mensual: Decimal4::from_raw(300_000),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BusinessConfig {
    /// The "GENERCON" of the certificate. Empty falls back to `Application.Name` in reports.
    pub nombre_comercial: String,
    /// The "ENERGIA CONTROLADA" / "Cuentas Claras".
    pub lema: String,
    /// The "PABLO BAEZ".
    pub contratista: String,
    pub cuit: String,
    pub direccion: String,
    pub telefono: String,
    pub email: String,
    pub logo_path: Option<PathBuf>,
    /// Only a suggestion; VAT is typed by hand (doc 06 §4.1).
    pub iva_sugerido: Decimal4,
    pub factura_dias_vencimiento_default: u32,
    /// Upper bound of each ageing bucket, in days, inclusive. Anything past the last one falls
    /// into the open-ended bucket.
    pub buckets_antiguedad: Vec<u32>,
    /// How much a payment may exceed the outstanding balance before being refused (INV-09). Zero
    /// by default; a small tolerance exists for the cent that rounding leaves behind.
    pub tolerancia_sobrepago_factura: Money,
    pub categoria_profundidad_maxima: u8,
    pub dias_por_frecuencia: DiasPorFrecuencia,
}

impl Default for BusinessConfig {
    fn default() -> Self {
        Self {
            nombre_comercial: String::new(),
            lema: String::new(),
            contratista: String::new(),
            cuit: String::new(),
            direccion: String::new(),
            telefono: String::new(),
            email: String::new(),
            logo_path: None,
            iva_sugerido: Decimal4::from_raw(210_000),
            factura_dias_vencimiento_default: 30,
            buckets_antiguedad: vec![30, 60, 90],
            tolerancia_sobrepago_factura: Money::ZERO,
            categoria_profundidad_maxima: 3,
            dias_por_frecuencia: DiasPorFrecuencia::default(),
        }
    }
}
