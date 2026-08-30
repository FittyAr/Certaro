//! Backup, restore and JSON dump. See `docs/13-servicios-externos-y-archivos.md` §4 and §5.

pub mod json;
pub mod service;

pub use service::SqliteBackupService;

/// File name of a backup: the prefix, a UTC timestamp, and the database extension.
///
/// UTC rather than local time so the names sort chronologically even across a daylight-saving
/// change, which is when a locally-named pair would sort backwards.
#[must_use]
pub fn nombre_backup(now: chrono::DateTime<chrono::Utc>) -> String {
    format!("electroobra_{}.db", now.format("%Y%m%d_%H%M%S"))
}

/// The instant encoded in a backup name, or `None` when the name is not one of ours.
#[must_use]
pub fn instante_de(nombre: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let resto = nombre.strip_prefix("electroobra_")?.strip_suffix(".db")?;
    chrono::NaiveDateTime::parse_from_str(resto, "%Y%m%d_%H%M%S")
        .ok()
        .map(|naive| naive.and_utc())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn instante(hora: u32) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc.with_ymd_and_hms(2026, 8, 29, hora, 30, 12).unwrap()
    }

    #[test]
    fn el_nombre_va_y_vuelve() {
        let nombre = nombre_backup(instante(14));
        assert_eq!(nombre, "electroobra_20260829_143012.db");
        assert_eq!(instante_de(&nombre), Some(instante(14)));
    }

    #[test]
    fn un_archivo_ajeno_no_se_interpreta_como_backup() {
        for nombre in [
            "electroobra.db",
            "otra_cosa_20260829_143012.db",
            "electroobra_20260829_143012.sqlite",
            "electroobra_ayer.db",
        ] {
            assert_eq!(instante_de(nombre), None, "{nombre}");
        }
    }

    #[test]
    fn los_nombres_ordenan_cronologicamente_como_texto() {
        let mut nombres = vec![nombre_backup(instante(20)), nombre_backup(instante(8))];
        nombres.sort();
        assert_eq!(nombres[0], nombre_backup(instante(8)));
    }
}
