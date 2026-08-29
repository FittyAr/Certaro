//! Identifier generation. See `docs/04-dinero-fechas-y-tipos.md` §6.
//!
//! New rows get UUID v7, which is time-ordered, so insertion order matches primary-index order and
//! SQLite keeps its locality. Legacy v4 identifiers are preserved by the importer and accepted
//! here without complaint: there is no version check anywhere, on purpose.

use uuid::Uuid;

pub trait IdGenerator: Send + Sync {
    fn new_id(&self) -> Uuid;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct UuidV7Generator;

impl IdGenerator for UuidV7Generator {
    fn new_id(&self) -> Uuid {
        Uuid::now_v7()
    }
}

/// Hands out a fixed sequence, then keeps repeating the last one. For tests that assert on exact
/// identifiers.
#[derive(Debug)]
pub struct SequenceIdGenerator {
    ids: Vec<Uuid>,
    next: std::sync::atomic::AtomicUsize,
}

impl SequenceIdGenerator {
    #[must_use]
    pub fn new(ids: Vec<Uuid>) -> Self {
        Self {
            ids,
            next: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl IdGenerator for SequenceIdGenerator {
    fn new_id(&self) -> Uuid {
        let i = self.next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.ids
            .get(i)
            .copied()
            .or_else(|| self.ids.last().copied())
            .unwrap_or(Uuid::nil())
    }
}
