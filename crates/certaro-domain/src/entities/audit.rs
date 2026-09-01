use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::row_version::RowVersion;

/// The audit block every business entity embeds. Composition, not inheritance: the entity owns an
/// `audit` field and flattens it when serialising. See `docs/05-dominio-entidades.md` §1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Audit {
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub row_version: RowVersion,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Audit {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            created_at: now,
            updated_at: None,
            row_version: RowVersion::INITIAL,
            is_deleted: false,
            deleted_at: None,
        }
    }

    /// Marks the row as modified and moves the version forward, which is what makes the next
    /// concurrent write with a stale version fail instead of overwriting.
    pub fn touch(&mut self, now: DateTime<Utc>) {
        self.updated_at = Some(now);
        self.row_version = self.row_version.next();
    }

    pub fn soft_delete(&mut self, now: DateTime<Utc>) {
        self.is_deleted = true;
        self.deleted_at = Some(now);
        self.touch(now);
    }

    pub fn restore(&mut self, now: DateTime<Utc>) {
        self.is_deleted = false;
        self.deleted_at = None;
        self.touch(now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn instant(minute: u32) -> DateTime<Utc> {
        DateTime::from_timestamp(1_767_225_600 + i64::from(minute) * 60, 0).unwrap()
    }

    #[test]
    fn a_new_row_starts_at_version_one_and_has_never_been_updated() {
        let audit = Audit::new(instant(0));
        assert_eq!(audit.row_version, RowVersion::INITIAL);
        assert_eq!(audit.updated_at, None);
        assert!(!audit.is_deleted);
    }

    #[test]
    fn touching_moves_the_version_forward() {
        let mut audit = Audit::new(instant(0));
        audit.touch(instant(1));
        assert_eq!(audit.row_version.as_u64(), 2);
        assert_eq!(audit.updated_at, Some(instant(1)));
    }

    #[test]
    fn a_soft_delete_also_counts_as_a_modification() {
        let mut audit = Audit::new(instant(0));
        audit.soft_delete(instant(2));
        assert!(audit.is_deleted);
        assert_eq!(audit.deleted_at, Some(instant(2)));
        assert_eq!(audit.row_version.as_u64(), 2);
    }

    #[test]
    fn restoring_clears_the_deletion_and_keeps_moving_the_version() {
        let mut audit = Audit::new(instant(0));
        audit.soft_delete(instant(2));
        audit.restore(instant(3));
        assert!(!audit.is_deleted);
        assert_eq!(audit.deleted_at, None);
        assert_eq!(audit.row_version.as_u64(), 3);
    }
}
