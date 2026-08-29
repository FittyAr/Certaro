//! Domain errors. See `docs/02-arquitectura.md` §6.1.
//!
//! These carry no translated text: the frontend owns translation. The application layer wraps
//! them into `AppError::Domain`, which maps to the `Error.Domain` i18n key.

/// Something that violates the rules of the domain itself, independently of any database or user
/// interface.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DomainError {
    #[error("money overflow")]
    MoneyOverflow,

    #[error("invalid scale: more than 4 decimal places")]
    InvalidScale,

    #[error("invalid number format")]
    InvalidNumberFormat,

    #[error("division by zero")]
    DivisionByZero,

    #[error("invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("unknown enum value {value} for {enum_name}")]
    UnknownEnumValue { enum_name: &'static str, value: i32 },

    #[error("invalid date")]
    InvalidDate,

    #[error("invariant violated: {0}")]
    InvariantViolated(&'static str),
}
