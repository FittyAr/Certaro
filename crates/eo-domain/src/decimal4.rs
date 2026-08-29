//! Non-monetary decimals: percentages, multipliers, quantities, worked days.
//!
//! Same scale as [`Money`](crate::Money), different type. The legacy system scaled these through
//! the same reflection-based converter and therefore could not tell them apart; here the compiler
//! does. See `docs/04-dinero-fechas-y-tipos.md` §1.2.

use crate::error::DomainError;
use crate::scaled::{self, SCALE};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Decimal4(i64);

impl Decimal4 {
    pub const ZERO: Decimal4 = Decimal4(0);
    pub const ONE: Decimal4 = Decimal4(10_000);
    pub const HUNDRED: Decimal4 = Decimal4(1_000_000);
    pub const HALF: Decimal4 = Decimal4(5_000);
    pub const SCALE: i64 = SCALE;

    #[must_use]
    pub const fn from_raw(raw: i64) -> Self {
        Decimal4(raw)
    }

    #[must_use]
    pub const fn raw(self) -> i64 {
        self.0
    }

    pub fn from_units(units: i64) -> Result<Self, DomainError> {
        scaled::from_units(units).map(Decimal4)
    }

    pub fn parse(s: &str) -> Result<Self, DomainError> {
        scaled::parse(s).map(Decimal4)
    }

    #[must_use]
    pub fn to_decimal_string(self) -> String {
        scaled::to_decimal_string(self.0)
    }

    pub fn checked_add(self, rhs: Decimal4) -> Result<Decimal4, DomainError> {
        scaled::checked_add(self.0, rhs.0).map(Decimal4)
    }

    pub fn checked_sub(self, rhs: Decimal4) -> Result<Decimal4, DomainError> {
        scaled::checked_sub(self.0, rhs.0).map(Decimal4)
    }

    pub fn checked_mul(self, factor: Decimal4) -> Result<Decimal4, DomainError> {
        scaled::checked_mul(self.0, factor.0).map(Decimal4)
    }

    pub fn checked_div(self, divisor: Decimal4) -> Result<Decimal4, DomainError> {
        scaled::checked_div(self.0, divisor.0).map(Decimal4)
    }

    #[must_use]
    pub fn round_to(self, decimals: u32) -> Decimal4 {
        Decimal4(scaled::round_raw(self.0, SCALE, pow10(decimals)))
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

    #[must_use]
    pub const fn abs(self) -> Decimal4 {
        Decimal4(self.0.saturating_abs())
    }

    #[must_use]
    pub const fn neg(self) -> Decimal4 {
        Decimal4(self.0.saturating_neg())
    }

    pub fn try_sum<I: IntoIterator<Item = Decimal4>>(iter: I) -> Result<Decimal4, DomainError> {
        iter.into_iter()
            .try_fold(Decimal4::ZERO, |acc, item| acc.checked_add(item))
    }

    /// Reads `self` as a percentage and returns the fraction: `60` becomes `0.6`.
    ///
    /// Exact only for percentages with at most two decimals, which is every percentage the system
    /// accepts (`Locale.DecimalesPorcentaje` defaults to 2). To apply a percentage to an amount use
    /// [`Money::percent`](crate::Money::percent), which multiplies before dividing and keeps the
    /// digits this conversion would drop.
    #[must_use]
    pub fn as_fraction(self) -> Decimal4 {
        Decimal4(scaled::checked_div(self.0, Decimal4::HUNDRED.0).unwrap_or(0))
    }

    /// True when the value lies in `[0, 100]`, the range every percentage field must respect.
    #[must_use]
    pub const fn is_valid_percentage(self) -> bool {
        self.0 >= 0 && self.0 <= Decimal4::HUNDRED.0
    }
}

const fn pow10(decimals: u32) -> i64 {
    match decimals {
        0 => 1,
        1 => 10,
        2 => 100,
        3 => 1_000,
        _ => 10_000,
    }
}

impl fmt::Display for Decimal4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_decimal_string())
    }
}

impl Serialize for Decimal4 {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        scaled::serde_impl::serialize(self.0, s)
    }
}

impl<'de> Deserialize<'de> for Decimal4 {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        scaled::serde_impl::deserialize(d).map(Decimal4)
    }
}
