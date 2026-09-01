//! Shared fixed-point arithmetic for [`Money`](crate::Money) and [`Decimal4`](crate::Decimal4).
//!
//! Both newtypes are an `i64` holding the value multiplied by [`SCALE`]. The arithmetic lives here
//! once so the two types cannot drift apart, while staying distinct types so the compiler refuses
//! to add pesos to percentages. See `docs/04-dinero-fechas-y-tipos.md` §1.
//!
//! Not a single operation in this module uses floating point.

use crate::error::DomainError;

/// Fixed scale of the base representation: 4 decimal places.
pub const SCALE: i64 = 10_000;

/// Number of decimal places implied by [`SCALE`].
pub const DECIMALS: u32 = 4;

/// Rounds `raw` so that everything below `to_scale` is discarded, half-away-from-zero.
///
/// The result stays in the original scale: rounding `23_456` (`2.3456`) to 2 decimals yields
/// `23_500` (`2.3500`), not `235`. Never banker's rounding, which is what `f64::round` and most
/// decimal libraries do by default.
/// `to_scale` must divide `from_scale`; every caller passes a power of ten derived from
/// [`DECIMALS`], so the condition holds by construction.
pub const fn round_raw(raw: i64, from_scale: i64, to_scale: i64) -> i64 {
    let factor = from_scale / to_scale;
    if factor == 1 {
        return raw;
    }
    let half = factor / 2;
    let q = raw / factor;
    let r = raw % factor;
    // `r` carries the sign of `raw`, so comparing its magnitude is enough.
    let r_abs = if r < 0 { -r } else { r };
    if r_abs >= half {
        if raw >= 0 {
            (q + 1) * factor
        } else {
            (q - 1) * factor
        }
    } else {
        q * factor
    }
}

/// Divides a 128-bit product by `SCALE`, rounding half-away-from-zero, and narrows back to `i64`.
///
/// This is the single place where precision is lost in a multiplication, and the reason the
/// intermediate value is `i128`: two values scaled by 10 000 multiply to a value scaled by
/// 100 000 000, which overflows `i64` well before the amounts stop being realistic.
fn narrow_scaled(product: i128) -> Result<i64, DomainError> {
    let scale = SCALE as i128;
    let half = scale / 2;
    let q = product / scale;
    let r = product % scale;
    let adjusted = if r.abs() >= half {
        if product >= 0 {
            q + 1
        } else {
            q - 1
        }
    } else {
        q
    };
    i64::try_from(adjusted).map_err(|_| DomainError::MoneyOverflow)
}

pub fn checked_add(lhs: i64, rhs: i64) -> Result<i64, DomainError> {
    lhs.checked_add(rhs).ok_or(DomainError::MoneyOverflow)
}

pub fn checked_sub(lhs: i64, rhs: i64) -> Result<i64, DomainError> {
    lhs.checked_sub(rhs).ok_or(DomainError::MoneyOverflow)
}

/// Multiplies a scaled value by a scaled factor, keeping the scale of the result.
pub fn checked_mul(lhs: i64, factor: i64) -> Result<i64, DomainError> {
    narrow_scaled((lhs as i128) * (factor as i128))
}

/// Divides a scaled value by a scaled divisor, keeping the scale of the result.
pub fn checked_div(lhs: i64, divisor: i64) -> Result<i64, DomainError> {
    if divisor == 0 {
        return Err(DomainError::DivisionByZero);
    }
    let numerator = (lhs as i128) * (SCALE as i128);
    let divisor = divisor as i128;
    let q = numerator / divisor;
    let r = numerator % divisor;
    // Half-away-from-zero on the quotient: compare twice the remainder against the divisor to
    // avoid a division that would itself need rounding.
    let adjusted = if (r.abs() * 2) >= divisor.abs() {
        if (numerator >= 0) == (divisor >= 0) {
            q + 1
        } else {
            q - 1
        }
    } else {
        q
    };
    i64::try_from(adjusted).map_err(|_| DomainError::MoneyOverflow)
}

/// Builds a scaled value from whole units: `from_units(40_000)` is `40000.0000`.
pub fn from_units(units: i64) -> Result<i64, DomainError> {
    units.checked_mul(SCALE).ok_or(DomainError::MoneyOverflow)
}

/// Renders the canonical wire format: optional sign, integer part with no thousands separator,
/// a dot, and **exactly four** decimals. `"0.0000"`, `"40000.0000"`, `"-240.7500"`.
pub fn to_decimal_string(raw: i64) -> String {
    // i128 so that i64::MIN does not overflow when taking the absolute value.
    let magnitude = (raw as i128).abs();
    let scale = SCALE as i128;
    let integer = magnitude / scale;
    let fraction = magnitude % scale;
    let sign = if raw < 0 { "-" } else { "" };
    format!(
        "{sign}{integer}.{fraction:0width$}",
        width = DECIMALS as usize
    )
}

/// Parses the wire format and the shapes a user can type.
///
/// Accepts `"1234.56"`, `"-240.75"`, `"1234"`, `".5"`, `"+3"`. Rejects the empty string, more than
/// four decimals, scientific notation, thousands separators and anything else, because a silently
/// truncated amount is worse than a rejected one.
pub fn parse(s: &str) -> Result<i64, DomainError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(DomainError::InvalidNumberFormat);
    }

    let (negative, rest) = match s.as_bytes()[0] {
        b'-' => (true, &s[1..]),
        b'+' => (false, &s[1..]),
        _ => (false, s),
    };
    if rest.is_empty() {
        return Err(DomainError::InvalidNumberFormat);
    }

    let (int_part, frac_part) = match rest.split_once('.') {
        Some((i, f)) => (i, f),
        None => (rest, ""),
    };

    // `".5"` is allowed, `"5."` is not: a trailing dot is a typo, not a number.
    if frac_part.is_empty() && rest.contains('.') {
        return Err(DomainError::InvalidNumberFormat);
    }
    if int_part.is_empty() && frac_part.is_empty() {
        return Err(DomainError::InvalidNumberFormat);
    }
    if !int_part.bytes().all(|b| b.is_ascii_digit()) {
        return Err(DomainError::InvalidNumberFormat);
    }
    if !frac_part.bytes().all(|b| b.is_ascii_digit()) {
        return Err(DomainError::InvalidNumberFormat);
    }
    if frac_part.len() > DECIMALS as usize {
        return Err(DomainError::InvalidScale);
    }

    let integer: i64 = if int_part.is_empty() {
        0
    } else {
        int_part.parse().map_err(|_| DomainError::MoneyOverflow)?
    };

    // Right-pad the fraction to exactly four digits: "5" is five tenths, so 5000.
    let mut fraction: i64 = 0;
    for i in 0..DECIMALS as usize {
        let digit = frac_part
            .as_bytes()
            .get(i)
            .map_or(0, |b| i64::from(b - b'0'));
        fraction = fraction * 10 + digit;
    }

    let magnitude = integer
        .checked_mul(SCALE)
        .and_then(|v| v.checked_add(fraction))
        .ok_or(DomainError::MoneyOverflow)?;

    Ok(if negative { -magnitude } else { magnitude })
}

/// Serde helpers shared by both newtypes. Values cross the wire as **strings** so that JavaScript
/// never sees a number it would silently degrade to `f64`.
pub mod serde_impl {
    use super::{parse, to_decimal_string};
    use serde::de::{Error as DeError, Unexpected, Visitor};
    use serde::{Deserializer, Serializer};
    use std::fmt;

    pub fn serialize<S: Serializer>(raw: i64, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&to_decimal_string(raw))
    }

    struct RawVisitor;

    impl Visitor<'_> for RawVisitor {
        type Value = i64;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a decimal string with at most 4 decimal places")
        }

        fn visit_str<E: DeError>(self, v: &str) -> Result<Self::Value, E> {
            parse(v).map_err(|e| E::custom(e.to_string()))
        }

        // Numbers are refused on purpose: accepting them would let a JSON producer round-trip an
        // amount through an f64 and lose centavos before it ever reaches this code.
        fn visit_f64<E: DeError>(self, v: f64) -> Result<Self::Value, E> {
            Err(E::invalid_type(Unexpected::Float(v), &self))
        }

        fn visit_i64<E: DeError>(self, v: i64) -> Result<Self::Value, E> {
            Err(E::invalid_type(Unexpected::Signed(v), &self))
        }

        fn visit_u64<E: DeError>(self, v: u64) -> Result<Self::Value, E> {
            Err(E::invalid_type(Unexpected::Unsigned(v), &self))
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
        d.deserialize_any(RawVisitor)
    }
}
