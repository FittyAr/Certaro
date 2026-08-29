//! State machines. See `docs/08-maquinas-de-estado.md`.
//!
//! The legacy system let any state be written over any other: an annulled invoice still took
//! payments and a cancelled job could go back to being quoted. Here the legal moves live in one
//! place, the transition is the only way to change a state, and the frontend asks for the legal
//! targets instead of listing the enum.

mod factura;
mod obra;
mod trabajo;

pub use factura::recalcular_estado_factura;

use crate::error::DomainError;

/// A closed enumeration whose values can only change along documented edges.
pub trait StateMachine: Copy + Eq + Sized + 'static {
    /// Name used in the error and in the i18n key. Not translated here.
    const ENTITY: &'static str;

    /// Every state reachable from this one by a user action. Automatic states are absent on
    /// purpose: they are written by the recalculation, never chosen from a dropdown.
    fn allowed_targets(self) -> &'static [Self];

    /// The variant name, which doubles as the last segment of its i18n key.
    fn as_key(self) -> &'static str;

    fn is_terminal(self) -> bool {
        self.allowed_targets().is_empty()
    }

    /// Staying put is always legal, so re-sending the current state is a no-op rather than an
    /// error. A retried request must not fail just because the first one succeeded.
    fn can_transition_to(self, to: Self) -> bool {
        self == to || self.allowed_targets().contains(&to)
    }

    fn transition_to(self, to: Self) -> Result<Self, DomainError> {
        if self.can_transition_to(to) {
            Ok(to)
        } else {
            Err(DomainError::InvalidStateTransition {
                entity: Self::ENTITY,
                from: self.as_key(),
                to: to.as_key(),
            })
        }
    }
}
