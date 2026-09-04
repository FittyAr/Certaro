//! Backups, restore and the JSON dump, against a real database file on disk.
//!
//! An in-memory database would not exercise the part that matters: closing the connection before
//! the file is replaced, and the `-wal` sidecar that the legacy restore left behind.

#[path = "backup/common.rs"]
mod common;
#[path = "backup/lifecycle.rs"]
mod lifecycle;
#[path = "backup/json.rs"]
mod json;
