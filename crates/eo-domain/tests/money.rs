//! The mandatory tests of `docs/17-testing.md` §2.1.
//!
//! Every figure in the system passes through these two types, so they are tested exhaustively.

use eo_domain::{Decimal4, DomainError, Money};
use pretty_assertions::assert_eq;
use proptest::prelude::*;

fn money(s: &str) -> Money {
    Money::parse(s).expect("test literal parses")
}

fn dec(s: &str) -> Decimal4 {
    Decimal4::parse(s).expect("test literal parses")
}

#[test]
fn money_parse_y_display_son_inversos() {
    for literal in [
        "0.0000",
        "1.0000",
        "-1.5000",
        "12345.6700",
        "40000.0000",
        "-240.7500",
        "0.0001",
    ] {
        assert_eq!(money(literal).to_decimal_string(), literal);
    }
}

#[test]
fn money_display_siempre_cuatro_decimales() {
    assert_eq!(Money::ZERO.to_decimal_string(), "0.0000");
    assert_eq!(money("-1.5").to_decimal_string(), "-1.5000");
    assert_eq!(money("12345.67").to_decimal_string(), "12345.6700");
    assert_eq!(money("1234").to_decimal_string(), "1234.0000");
    assert_eq!(money(".5").to_decimal_string(), "0.5000");
}

#[test]
fn money_suma_es_exacta() {
    // The case that fails with f64: 0.1 + 0.2 != 0.3.
    let sum = money("0.1").checked_add(money("0.2")).expect("no overflow");
    assert_eq!(sum, money("0.3"));
    assert_eq!(sum.to_decimal_string(), "0.3000");
}

#[test]
fn money_suma_detecta_overflow() {
    let max = Money::from_raw(i64::MAX);
    assert_eq!(
        max.checked_add(Money::from_raw(1)),
        Err(DomainError::MoneyOverflow)
    );

    let min = Money::from_raw(i64::MIN);
    assert_eq!(
        min.checked_sub(Money::from_raw(1)),
        Err(DomainError::MoneyOverflow)
    );
}

#[test]
fn money_multiplicacion_redondea_half_away_from_zero() {
    // 2.5 units times one, rounded to the unit, is 3 and not the 2 banker's rounding would give.
    assert_eq!(money("2.5").round_to(0), money("3"));
    assert_eq!(money("-2.5").round_to(0), money("-3"));
    assert_eq!(money("2.345").round_to(2), money("2.35"));
    assert_eq!(money("-2.345").round_to(2), money("-2.35"));
    assert_eq!(money("2.355").round_to(2), money("2.36"));
    assert_eq!(money("2.344").round_to(2), money("2.34"));

    // The rounding that matters most is the one inside the multiplication, where the intermediate
    // product carries eight decimals. `1.0000 x 1.00005` cannot be expressed, so the equivalent
    // check is that a product landing exactly on a half rounds away from zero, not to even.
    assert_eq!(
        money("0.0001").checked_mul(dec("0.5")).expect("ok"),
        money("0.0001")
    );
    assert_eq!(
        money("-0.0001").checked_mul(dec("0.5")).expect("ok"),
        money("-0.0001")
    );
    assert_eq!(
        money("0.0002").checked_mul(dec("1.5")).expect("ok"),
        money("0.0003")
    );
}

#[test]
fn money_multiplicacion_por_cero_es_cero() {
    let result = money("-240.75")
        .checked_mul(Decimal4::ZERO)
        .expect("no overflow");
    assert_eq!(result, Money::ZERO);
    // Not a negative zero: the raw representation must be exactly 0.
    assert_eq!(result.raw(), 0);
    assert_eq!(result.to_decimal_string(), "0.0000");
}

#[test]
fn money_division_por_cero_es_error() {
    assert_eq!(
        money("100").checked_div(Decimal4::ZERO),
        Err(DomainError::DivisionByZero)
    );
}

#[test]
fn money_negativo_conserva_signo_en_display() {
    assert_eq!(money("-0.0001").to_decimal_string(), "-0.0001");
    assert_eq!(money("-1234.5").to_decimal_string(), "-1234.5000");
}

#[test]
fn money_serializa_como_string() {
    let json = serde_json::to_string(&money("12345.67")).expect("serialises");
    assert_eq!(json, r#""12345.6700""#);

    let back: Money = serde_json::from_str(&json).expect("deserialises");
    assert_eq!(back, money("12345.67"));
}

#[test]
fn money_deserializa_rechaza_un_numero_json() {
    // Accepting a number would let a producer round-trip the amount through an f64 first.
    assert!(serde_json::from_str::<Money>("12345.67").is_err());
    assert!(serde_json::from_str::<Money>("12345").is_err());
}

#[test]
fn money_deserializa_rechaza_mas_de_cuatro_decimales() {
    assert_eq!(Money::parse("1.00005"), Err(DomainError::InvalidScale));
    assert!(serde_json::from_str::<Money>(r#""1.00005""#).is_err());
}

#[test]
fn money_parse_rechaza_lo_que_no_es_un_numero() {
    for bad in ["", " ", "abc", "1e4", "1.2.3", "1,5", "5.", "--1", "1 000"] {
        assert!(Money::parse(bad).is_err(), "{bad:?} should be rejected");
    }
}

#[test]
fn money_from_units_y_from_raw_coinciden() {
    assert_eq!(
        Money::from_units(40_000).expect("no overflow"),
        Money::from_raw(400_000_000)
    );
    assert_eq!(
        Money::from_units(40_000).expect("ok").to_decimal_string(),
        "40000.0000"
    );
}

#[test]
fn money_from_raw_ida_y_vuelta_en_los_bordes() {
    for raw in [0, 1, -1, i64::MAX, i64::MIN, i64::MIN + 1] {
        assert_eq!(Money::from_raw(raw).raw(), raw);
    }
}

#[test]
fn money_try_sum_acumula_y_detecta_overflow() {
    let items = [money("10.25"), money("0.75"), money("-1.00")];
    assert_eq!(Money::try_sum(items).expect("no overflow"), money("10"));

    let overflowing = [Money::from_raw(i64::MAX), Money::from_raw(1)];
    assert_eq!(Money::try_sum(overflowing), Err(DomainError::MoneyOverflow));

    assert_eq!(Money::try_sum([]).expect("empty is zero"), Money::ZERO);
}

#[test]
fn decimal4_porcentaje_de_money_es_exacto() {
    // 1000.0000 * 33.3333% = 333.3330
    let result = money("1000").percent(dec("33.3333")).expect("no overflow");
    assert_eq!(result.to_decimal_string(), "333.3330");

    // The UOCRA adjustment of doc 06 §5: 8% of an amount, as a percentage and not a fixed sum.
    assert_eq!(
        money("125000")
            .percent(dec("8"))
            .expect("ok")
            .to_decimal_string(),
        "10000.0000"
    );
}

#[test]
fn decimal4_as_fraction_divide_por_cien() {
    assert_eq!(dec("60").as_fraction(), dec("0.6"));
    assert_eq!(Decimal4::HUNDRED.as_fraction(), Decimal4::ONE);
    assert_eq!(Decimal4::ZERO.as_fraction(), Decimal4::ZERO);
}

#[test]
fn decimal4_constantes_tienen_la_escala_correcta() {
    assert_eq!(Decimal4::ONE.to_decimal_string(), "1.0000");
    assert_eq!(Decimal4::HALF.to_decimal_string(), "0.5000");
    assert_eq!(Decimal4::HUNDRED.to_decimal_string(), "100.0000");
    assert_eq!(Decimal4::ZERO.to_decimal_string(), "0.0000");
}

#[test]
fn decimal4_valida_el_rango_de_porcentaje() {
    assert!(dec("0").is_valid_percentage());
    assert!(dec("100").is_valid_percentage());
    assert!(dec("33.3333").is_valid_percentage());
    assert!(!dec("100.0001").is_valid_percentage());
    assert!(!dec("-0.0001").is_valid_percentage());
}

#[test]
fn money_division_redondea_half_away_from_zero() {
    // 10 / 3 = 3.3333…, truncated at the fourth decimal and rounded up.
    assert_eq!(
        money("10")
            .checked_div(dec("3"))
            .expect("ok")
            .to_decimal_string(),
        "3.3333"
    );
    assert_eq!(
        money("-10")
            .checked_div(dec("3"))
            .expect("ok")
            .to_decimal_string(),
        "-3.3333"
    );
    assert_eq!(
        money("1")
            .checked_div(dec("8"))
            .expect("ok")
            .to_decimal_string(),
        "0.1250"
    );
}

proptest! {
    /// Addition is associative. With f64 it is not.
    #[test]
    fn money_suma_asociativa(a in -1_000_000_000_000i64..1_000_000_000_000,
                             b in -1_000_000_000_000i64..1_000_000_000_000,
                             c in -1_000_000_000_000i64..1_000_000_000_000) {
        let (a, b, c) = (Money::from_raw(a), Money::from_raw(b), Money::from_raw(c));
        let left = a.checked_add(b).and_then(|ab| ab.checked_add(c));
        let right = b.checked_add(c).and_then(|bc| a.checked_add(bc));
        prop_assert_eq!(left, right);
    }

    /// Display and parse are inverses for every representable value.
    #[test]
    fn money_roundtrip(raw: i64) {
        let m = Money::from_raw(raw);
        prop_assert_eq!(Money::parse(&m.to_decimal_string()), Ok(m));
    }

    /// Multiplying by 100% returns the same value.
    ///
    /// This looks trivial and is not: a naive `a * b / 10_000` without correct rounding loses one
    /// unit on about half the inputs.
    #[test]
    fn money_por_cien_por_ciento_es_identidad(raw in -100_000_000_000_000i64..100_000_000_000_000) {
        let m = Money::from_raw(raw);
        prop_assert_eq!(m.percent(Decimal4::HUNDRED), Ok(m));
        prop_assert_eq!(m.checked_mul(Decimal4::ONE), Ok(m));
    }
}
