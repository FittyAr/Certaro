//! Formatting of amounts, dates and numbers for the reports. See `docs/12` §1.2 rules 3 and 4.
//!
//! Every visible value of every report goes through this module. The legacy system formatted with
//! the operating system's culture, so the same export produced different files on two machines,
//! and the PDF, the spreadsheet and the Word document disagreed on how many decimals an amount
//! had. Here the locale is explicit and there is exactly one implementation.

use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc};
use eo_application::config::LocaleConfig;
use eo_domain::{Decimal4, Money};

/// An amount with the configured symbol, separators and number of visible decimals.
///
/// The sign goes before the symbol, which is how a negative amount is written in Spanish and how
/// the interface shows it; `$ -240,75` reads as a currency called `$ -`.
#[must_use]
pub fn format_money(value: Money, locale: &LocaleConfig) -> String {
    let magnitude = format_money_plain(value.abs(), locale);
    let signo = if value.round_to(u32::from(locale.decimales_moneda)).is_negative() {
        "-"
    } else {
        ""
    };
    format!("{signo}{} {magnitude}", locale.simbolo_moneda)
}

/// The same number without the symbol, for columns that carry it in the heading.
#[must_use]
pub fn format_money_plain(value: Money, locale: &LocaleConfig) -> String {
    format_scaled(
        value.round_to(u32::from(locale.decimales_moneda)).raw(),
        locale.decimales_moneda,
        locale,
        true,
    )
}

/// A percentage with the configured number of decimals and its sign.
#[must_use]
pub fn format_percent(value: Decimal4, locale: &LocaleConfig, decimals: u8) -> String {
    format!(
        "{} %",
        format_scaled(
            value.round_to(u32::from(decimals)).raw(),
            decimals,
            locale,
            false
        )
    )
}

/// A plain number: quantities, multipliers, days.
///
/// Trailing zeros are dropped down to `min_decimals`, so a quantity of `2.5000` prints as `2,5`
/// and one of `2.0000` prints as `2`. Doc 12 §4.4 asks for exactly that: the legacy `N0` format
/// printed `2` for a quantity of `2,5` and hid half a unit.
#[must_use]
pub fn format_number(value: Decimal4, locale: &LocaleConfig, min_decimals: u8) -> String {
    let text = format_scaled(value.raw(), 4, locale, false);
    trim_decimals(&text, &locale.separador_decimal, min_decimals)
}

/// A civil date with the configured pattern.
#[must_use]
pub fn format_date(value: NaiveDate, locale: &LocaleConfig) -> String {
    apply_pattern(
        &locale.formato_fecha,
        value.year(),
        value.month(),
        value.day(),
        0,
        0,
        0,
    )
}

/// An instant, converted to the local offset the way the interface shows it.
#[must_use]
pub fn format_datetime(value: DateTime<Utc>, locale: &LocaleConfig) -> String {
    let local = value.with_timezone(&chrono::Local);
    apply_pattern(
        &locale.formato_fecha_hora,
        local.year(),
        local.month(),
        local.day(),
        local.hour(),
        local.minute(),
        local.second(),
    )
}

/// ISO-8601 for the CSV, which Excel must not reinterpret as month/day (doc 12 §2.4).
#[must_use]
pub fn format_date_iso(value: NaiveDate) -> String {
    value.format("%Y-%m-%d").to_string()
}

/// A scaled integer as text, grouping thousands and using the configured decimal mark.
fn format_scaled(raw: i64, decimals: u8, locale: &LocaleConfig, group: bool) -> String {
    let negative = raw < 0;
    // `unsigned_abs` rather than `abs`: `i64::MIN` has no positive counterpart.
    let magnitude = raw.unsigned_abs();
    let scale = 10_u64.pow(4);
    let whole = magnitude / scale;
    let fraction = magnitude % scale;

    let mut text = if group {
        group_thousands(whole, &locale.separador_miles)
    } else {
        whole.to_string()
    };

    if decimals > 0 {
        let shown = format!("{fraction:04}");
        text.push_str(&locale.separador_decimal);
        text.push_str(&shown[..usize::from(decimals.min(4))]);
    }

    if negative {
        format!("-{text}")
    } else {
        text
    }
}

fn group_thousands(value: u64, separator: &str) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push_str(separator);
        }
        out.push(digit);
    }
    out
}

/// Drops trailing zeros of the fraction, and the separator with them when nothing is left.
fn trim_decimals(text: &str, separator: &str, min_decimals: u8) -> String {
    let Some((whole, fraction)) = text.split_once(separator) else {
        return text.to_owned();
    };
    let kept = fraction.trim_end_matches('0');
    let width = usize::from(min_decimals).max(kept.len());
    if width == 0 {
        return whole.to_owned();
    }
    let padded = format!("{kept:0<width$}");
    format!("{whole}{separator}{padded}")
}

/// Applies a `dd/MM/yyyy HH:mm`-style pattern. Longest tokens first so `yyyy` is not eaten as two
/// `yy`; the frontend's `useDateFormat` does the same, and both must agree.
fn apply_pattern(
    pattern: &str,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> String {
    let mut out = String::with_capacity(pattern.len() + 4);
    let bytes: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        let rest: String = bytes[i..].iter().collect();
        let replacement = if rest.starts_with("yyyy") {
            Some((4, format!("{year:04}")))
        } else if rest.starts_with("yy") {
            Some((2, format!("{:02}", year.rem_euclid(100))))
        } else if rest.starts_with("MM") {
            Some((2, format!("{month:02}")))
        } else if rest.starts_with("dd") {
            Some((2, format!("{day:02}")))
        } else if rest.starts_with("HH") {
            Some((2, format!("{hour:02}")))
        } else if rest.starts_with("mm") {
            Some((2, format!("{minute:02}")))
        } else if rest.starts_with("ss") {
            Some((2, format!("{second:02}")))
        } else {
            None
        };

        match replacement {
            Some((len, text)) => {
                out.push_str(&text);
                i += len;
            }
            None => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locale() -> LocaleConfig {
        LocaleConfig::default()
    }

    fn ingles() -> LocaleConfig {
        LocaleConfig {
            separador_miles: ",".to_owned(),
            separador_decimal: ".".to_owned(),
            formato_fecha: "MM/dd/yyyy".to_owned(),
            simbolo_moneda: "US$".to_owned(),
            ..LocaleConfig::default()
        }
    }

    #[test]
    fn formato_moneda_es_estable() {
        let l = locale();
        assert_eq!(format_money(Money::from_raw(12_345_678_900), &l), "$ 1.234.567,89");
        assert_eq!(format_money(Money::parse("1500.5").unwrap(), &l), "$ 1.500,50");
        assert_eq!(format_money(Money::ZERO, &l), "$ 0,00");
        assert_eq!(format_money(Money::parse("-240.75").unwrap(), &l), "-$ 240,75");
        assert_eq!(format_money(Money::parse("999.999").unwrap(), &l), "$ 1.000,00");
    }

    #[test]
    fn el_mismo_importe_en_otro_locale_da_otra_cadena() {
        assert_eq!(
            format_money(Money::parse("1234567.89").unwrap(), &ingles()),
            "US$ 1,234,567.89"
        );
    }

    #[test]
    fn los_decimales_visibles_salen_de_la_configuracion() {
        let cuatro = LocaleConfig {
            decimales_moneda: 4,
            ..locale()
        };
        assert_eq!(format_money(Money::parse("1.2345").unwrap(), &cuatro), "$ 1,2345");
        let cero = LocaleConfig {
            decimales_moneda: 0,
            ..locale()
        };
        assert_eq!(format_money(Money::parse("1500.6").unwrap(), &cero), "$ 1.501");
    }

    #[test]
    fn formato_fecha_es_estable() {
        let fecha = NaiveDate::from_ymd_opt(2026, 8, 14).unwrap();
        assert_eq!(format_date(fecha, &locale()), "14/08/2026");
        assert_eq!(format_date(fecha, &ingles()), "08/14/2026");
        assert_eq!(format_date_iso(fecha), "2026-08-14");
    }

    #[test]
    fn un_patron_de_dos_digitos_de_anio_no_se_come_el_de_cuatro() {
        let fecha = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
        let l = LocaleConfig {
            formato_fecha: "dd-MM-yy".to_owned(),
            ..locale()
        };
        assert_eq!(format_date(fecha, &l), "02-01-26");
    }

    #[test]
    fn las_cantidades_pierden_los_ceros_pero_no_los_decimales_reales() {
        let l = locale();
        assert_eq!(format_number(Decimal4::parse("2").unwrap(), &l, 0), "2");
        assert_eq!(format_number(Decimal4::parse("2.5").unwrap(), &l, 0), "2,5");
        assert_eq!(format_number(Decimal4::parse("2.5").unwrap(), &l, 2), "2,50");
        assert_eq!(format_number(Decimal4::parse("100.1234").unwrap(), &l, 0), "100,1234");
    }

    #[test]
    fn el_porcentaje_usa_los_decimales_que_se_le_piden() {
        let l = locale();
        assert_eq!(format_percent(Decimal4::parse("33.333").unwrap(), &l, 1), "33,3 %");
        assert_eq!(format_percent(Decimal4::parse("-12.5").unwrap(), &l, 1), "-12,5 %");
        assert_eq!(format_percent(Decimal4::parse("8").unwrap(), &l, 0), "8 %");
    }

    #[test]
    fn un_importe_negativo_chico_conserva_el_signo() {
        assert_eq!(format_money(Money::parse("-0.01").unwrap(), &locale()), "-$ 0,01");
    }
}
