//! Optimistic concurrency token. See `docs/04-dinero-fechas-y-tipos.md` §5.
//!
//! Eight bytes read as a big-endian `u64`, starting at 1. The legacy system declared the same
//! column but left the increment to EF Core and in practice it almost never moved; here bumping it
//! is the repository's job and it is not optional.

use crate::error::DomainError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RowVersion([u8; 8]);

impl RowVersion {
    pub const INITIAL: RowVersion = RowVersion([0, 0, 0, 0, 0, 0, 0, 1]);

    #[must_use]
    pub const fn from_bytes(bytes: [u8; 8]) -> Self {
        RowVersion(bytes)
    }

    #[must_use]
    pub const fn as_bytes(self) -> [u8; 8] {
        self.0
    }

    #[must_use]
    pub const fn as_u64(self) -> u64 {
        u64::from_be_bytes(self.0)
    }

    /// Wrapping is intentional: after 18 quintillion updates to one row, starting over is a better
    /// outcome than an overflow.
    #[must_use]
    pub fn next(self) -> RowVersion {
        RowVersion(u64::from_be_bytes(self.0).wrapping_add(1).to_be_bytes())
    }

    /// The 16-character lowercase hex string the DTOs carry.
    #[must_use]
    pub fn to_hex(self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }

    pub fn parse_hex(s: &str) -> Result<Self, DomainError> {
        let s = s.trim();
        if s.len() != 16 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(DomainError::InvalidNumberFormat);
        }
        let mut bytes = [0u8; 8];
        for (i, byte) in bytes.iter_mut().enumerate() {
            let pair = s
                .get(i * 2..i * 2 + 2)
                .ok_or(DomainError::InvalidNumberFormat)?;
            *byte = u8::from_str_radix(pair, 16).map_err(|_| DomainError::InvalidNumberFormat)?;
        }
        Ok(RowVersion(bytes))
    }

    /// From what the `BLOB` column holds. Any length other than 8 is a data error.
    pub fn from_slice(slice: &[u8]) -> Result<Self, DomainError> {
        <[u8; 8]>::try_from(slice)
            .map(RowVersion)
            .map_err(|_| DomainError::InvariantViolated("row_version must be exactly 8 bytes"))
    }
}

impl Default for RowVersion {
    fn default() -> Self {
        RowVersion::INITIAL
    }
}

impl fmt::Display for RowVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl Serialize for RowVersion {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for RowVersion {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        RowVersion::parse_hex(&s).map_err(serde::de::Error::custom)
    }
}
