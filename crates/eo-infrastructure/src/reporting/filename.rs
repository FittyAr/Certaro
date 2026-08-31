//! Proposed file names and the sanitising of the parts that come from user data. See `docs/12` §1.3.

use chrono::{DateTime, NaiveDate, Utc};

/// Longest a single user-provided component may be, in characters.
const MAX_PART: usize = 60;

/// The formats a report can be written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatoExport {
    Pdf,
    Xlsx,
    Docx,
    Csv,
    Json,
}

impl FormatoExport {
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            FormatoExport::Pdf => "pdf",
            FormatoExport::Xlsx => "xlsx",
            FormatoExport::Docx => "docx",
            FormatoExport::Csv => "csv",
            FormatoExport::Json => "json",
        }
    }
}

/// Replaces everything that cannot be in a file name, collapses the separators it leaves behind and
/// caps the length.
///
/// A customer called `Metalúrgica S.A. / Planta 2` is a real name and would otherwise produce a
/// path with a directory separator in the middle of it.
#[must_use]
pub fn sanitize(part: &str, fallback: &str) -> String {
    let replaced: String = part
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '-' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect();

    let mut collapsed = String::with_capacity(replaced.len());
    for c in replaced.chars() {
        if c == '_' && collapsed.ends_with('_') {
            continue;
        }
        collapsed.push(c);
    }

    let trimmed: String = collapsed
        .trim_matches(|c| c == '_' || c == '.')
        .chars()
        .take(MAX_PART)
        .collect();

    if trimmed.is_empty() {
        fallback.to_owned()
    } else {
        trimmed
    }
}

#[must_use]
pub fn stamp(now: DateTime<Utc>) -> String {
    now.with_timezone(&chrono::Local)
        .format("%Y%m%d_%H%M%S")
        .to_string()
}

#[must_use]
pub fn day(date: NaiveDate) -> String {
    date.format("%Y%m%d").to_string()
}

#[must_use]
pub fn movimientos(now: DateTime<Utc>, formato: FormatoExport) -> String {
    format!("Movimientos_{}.{}", stamp(now), formato.extension())
}

#[must_use]
pub fn liquidacion(empleado: &str, hasta: NaiveDate) -> String {
    format!(
        "Liquidacion_{}_{}.pdf",
        sanitize(empleado, "empleado"),
        day(hasta)
    )
}

#[must_use]
pub fn certificado(obra: &str, numero: i32, fecha: NaiveDate) -> String {
    format!(
        "Certificado_{}_{}_{}.pdf",
        sanitize(obra, "obra"),
        numero,
        day(fecha)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn instante() -> DateTime<Utc> {
        // Fixed so the name is asserted exactly; the local offset is applied by `stamp`.
        chrono::TimeZone::with_ymd_and_hms(&chrono::Local, 2026, 8, 29, 14, 30, 12)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn el_nombre_de_movimientos_lleva_la_marca_de_tiempo_y_la_extension() {
        assert_eq!(
            movimientos(instante(), FormatoExport::Xlsx),
            "Movimientos_20260829_143012.xlsx"
        );
    }

    #[test]
    fn una_barra_en_el_nombre_del_cliente_no_llega_al_path() {
        let nombre = liquidacion(
            "Juan / Pérez",
            NaiveDate::from_ymd_opt(2026, 8, 31).unwrap(),
        );
        assert_eq!(nombre, "Liquidacion_Juan_Pérez_20260831.pdf");
        assert!(!nombre.contains('/'));
    }

    #[test]
    fn los_separadores_repetidos_se_colapsan() {
        assert_eq!(sanitize("Obra   ***   Norte", "x"), "Obra_Norte");
    }

    #[test]
    fn un_nombre_que_queda_vacio_usa_el_respaldo() {
        assert_eq!(sanitize("***", "obra"), "obra");
        assert_eq!(sanitize("", "obra"), "obra");
    }

    #[test]
    fn el_componente_se_recorta_a_sesenta_caracteres() {
        let largo = "a".repeat(200);
        assert_eq!(sanitize(&largo, "x").chars().count(), 60);
    }

    #[test]
    fn el_certificado_lleva_obra_numero_y_fecha() {
        assert_eq!(
            certificado(
                "Edificio Sur",
                7,
                NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()
            ),
            "Certificado_Edificio_Sur_7_20260301.pdf"
        );
    }

    #[test]
    fn un_punto_al_final_no_sobrevive_y_no_deja_doble_extension() {
        assert_eq!(sanitize("Obra Norte.", "x"), "Obra_Norte");
    }
}
