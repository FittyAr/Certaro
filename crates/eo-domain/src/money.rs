//! Monetary amounts. See `docs/04-dinero-fechas-y-tipos.md` §1.

use crate::decimal4::Decimal4;
use crate::error::DomainError;
use crate::scaled;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

pub use crate::scaled::SCALE;

/// A monetary amount, stored as an `i64` scaled by [`SCALE`].
///
/// Never use `f64` for money: `0.1 + 0.2 != 0.3` is a difference the user finds immediately when
/// comparing the app against a receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Money(i64);

impl Money {
    pub const ZERO: Money = Money(0);
    pub const SCALE: i64 = SCALE;

    /// From the base representation, i.e. what the `INTEGER` column holds.
    #[must_use]
    pub const fn from_raw(raw: i64) -> Self {
        Money(raw)
    }

    /// To the base representation, i.e. what goes into the `INTEGER` column.
    #[must_use]
    pub const fn raw(self) -> i64 {
        self.0
    }

    /// From whole units: `from_units(40_000)` is `40000.0000`.
    pub fn from_units(units: i64) -> Result<Self, DomainError> {
        scaled::from_units(units).map(Money)
    }

    /// Parses `"1234.56"`, `"-240.75"`, `"1234"`, `".5"`. Rejects more than four decimals.
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        scaled::parse(s).map(Money)
    }

    /// The canonical wire format: exactly four decimals.
    #[must_use]
    pub fn to_decimal_string(self) -> String {
        scaled::to_decimal_string(self.0)
    }

    pub fn checked_add(self, rhs: Money) -> Result<Money, DomainError> {
        scaled::checked_add(self.0, rhs.0).map(Money)
    }

    pub fn checked_sub(self, rhs: Money) -> Result<Money, DomainError> {
        scaled::checked_sub(self.0, rhs.0).map(Money)
    }

    /// Multiplies by a decimal factor: a quantity, a percentage fraction, a multiplier.
    ///
    /// There is deliberately no `Money × Money`: money times money is not money.
    pub fn checked_mul(self, factor: Decimal4) -> Result<Money, DomainError> {
        scaled::checked_mul(self.0, factor.raw()).map(Money)
    }

    pub fn checked_div(self, divisor: Decimal4) -> Result<Money, DomainError> {
        scaled::checked_div(self.0, divisor.raw()).map(Money)
    }

    /// Rounds to `decimals` (0..=4) places, half-away-from-zero. Values above 4 are clamped to 4,
    /// which is a no-op, because the type cannot hold more precision than that anyway.
    #[must_use]
    pub fn round_to(self, decimals: u32) -> Money {
        Money(scaled::round_raw(self.0, SCALE, pow10(decimals)))
    }

    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub const fn is_negative(self) -> bool {
        self.0 < 0
    }

    #[must_use]
    pub const fn is_positive(self) -> bool {
        self.0 > 0
    }

    /// Saturating at `i64::MIN + 1`. That bound is ~922 billion million units of currency, so it
    /// is not reachable with real data; saturating keeps the signature total.
    #[must_use]
    pub const fn abs(self) -> Money {
        Money(self.0.saturating_abs())
    }

    #[must_use]
    pub const fn neg(self) -> Money {
        Money(self.0.saturating_neg())
    }

    /// Sums a sequence, failing on overflow instead of wrapping.
    pub fn try_sum<I: IntoIterator<Item = Money>>(iter: I) -> Result<Money, DomainError> {
        iter.into_iter()
            .try_fold(Money::ZERO, |acc, item| acc.checked_add(item))
    }

    /// Applies a percentage: `Money::parse("200")?.percent(Decimal4::parse("21")?)` is `42.0000`.
    ///
    /// Equivalent to multiplying by the fraction, in one step so the caller cannot forget to
    /// divide by a hundred.
    pub fn percent(self, percentage: Decimal4) -> Result<Money, DomainError> {
        self.checked_mul(percentage.as_fraction())
    }
}

/// `10^decimals`, clamped to the four decimals the type can represent.
const fn pow10(decimals: u32) -> i64 {
    match decimals {
        0 => 1,
        1 => 10,
        2 => 100,
        3 => 1_000,
        _ => 10_000,
    }
}

impl fmt::Display for Money {
    /// The wire format, not a localised one. Presentation formatting belongs to the frontend and
    /// the report generator, which read the separators from configuration.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_decimal_string())
    }
}

impl Serialize for Money {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        scaled::serde_impl::serialize(self.0, s)
    }
}

impl<'de> Deserialize<'de> for Money {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        scaled::serde_impl::deserialize(d).map(Money)
    }
}
