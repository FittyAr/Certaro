//! Date conversion from the legacy format. See `docs/15-migracion-de-datos.md` §3.5.

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;

/// Parses the three text formats the legacy system wrote, because different EF Core versions
/// produced different precisions.
pub fn parse_legacy_text(raw: &str) -> Result<NaiveDateTime> {
    let trimmed = raw.trim();
    // Try with fractional seconds first (7 digits, EF Core default).
    if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S%.f") {
        return Ok(dt);
    }
    // ISO format with Z suffix.
    if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed.trim_end_matches('Z'), "%Y-%m-%dT%H:%M:%S%.f") {
        return Ok(dt);
    }
    // Without fractional seconds.
    if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt);
    }
    // ISO without fractional.
    if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
        return Ok(dt);
    }
    anyhow::bail!("cannot parse legacy date: {raw:?}")
}

/// Audit dates: the text already represents UTC (`DateTime.UtcNow` in the old system).
pub fn audit(raw: &str) -> Result<DateTime<Utc>> {
    let naive = parse_legacy_text(raw)
        .with_context(|| format!("audit date: {raw:?}"))?;
    Ok(Utc.from_utc_datetime(&naive))
}

/// Business instant: the text represents local time in the configured timezone.
/// `Movimientos.Fecha` is the only column that keeps the time component.
pub fn business_instant(raw: &str, tz: Tz) -> Result<DateTime<Utc>> {
    let naive = parse_legacy_text(raw)
        .with_context(|| format!("business instant: {raw:?}"))?;
    match tz.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => Ok(dt.with_timezone(&Utc)),
        chrono::LocalResult::Ambiguous(a, _) => Ok(a.with_timezone(&Utc)),
        chrono::LocalResult::None => {
            // DST gap: shift forward one hour.
            let shifted = naive + chrono::Duration::hours(1);
            Ok(tz.from_utc_datetime(&shifted).with_timezone(&Utc))
        }
    }
}

/// Civil date: only the day matters. The time is discarded and replaced with midnight UTC.
/// This is what prevents an asistencia at 22:30 local from becoming the next day.
pub fn business_civil(raw: &str) -> Result<DateTime<Utc>> {
    let naive = parse_legacy_text(raw)
        .with_context(|| format!("civil date: {raw:?}"))?;
    Ok(civil_to_utc(naive.date()))
}

/// Midnight UTC of the given civil date.
pub fn civil_to_utc(date: NaiveDate) -> DateTime<Utc> {
    Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_acepta_formato_sin_fraccion() {
        let dt = parse_legacy_text("2026-03-15 22:30:00").unwrap();
        assert_eq!(dt.format("%Y-%m-%d %H:%M:%S").to_string(), "2026-03-15 22:30:00");
    }

    #[test]
    fn parse_acepta_formato_con_fraccion() {
        let dt = parse_legacy_text("2026-03-15 22:30:00.1234567").unwrap();
        // The fractional part is preserved.
        assert!(dt.format("%Y-%m-%d %H:%M:%S%.f").to_string().contains("22:30:00.1234567"));
    }

    #[test]
    fn parse_acepta_iso_con_z() {
        let dt = parse_legacy_text("2026-03-15T22:30:00.123Z").unwrap();
        assert!(dt.format("%Y-%m-%d %H:%M:%S%.f").to_string().contains("22:30:00.123"));
    }

    #[test]
    fn fecha_civil_no_cambia_de_dia() {
        let tz: Tz = "America/Argentina/Buenos_Aires".parse().unwrap();
        // 22:30 local is 01:30 UTC next day, but civil date keeps the original day.
        let utc = business_civil("2026-03-15 22:30:00").unwrap();
        assert_eq!(utc.format("%Y-%m-%d").to_string(), "2026-03-15");
        assert_eq!(utc.format("%H:%M:%S").to_string(), "00:00:00");
    }

    #[test]
    fn fecha_negocio_con_hora_se_convierte() {
        let tz: Tz = "America/Argentina/Buenos_Aires".parse().unwrap();
        // UTC-3: 22:00 local = 01:00 UTC next day.
        let utc = business_instant("2026-03-15 22:00:00", tz).unwrap();
        assert_eq!(utc.format("%Y-%m-%d %H:%M").to_string(), "2026-03-16 01:00");
    }

    #[test]
    fn auditoria_no_se_desplaza() {
        let utc = audit("2026-03-15 22:30:00").unwrap();
        assert_eq!(utc.format("%Y-%m-%d %H:%M:%S").to_string(), "2026-03-15 22:30:00");
    }
}
