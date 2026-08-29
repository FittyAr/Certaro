//! Application layer: use cases, DTOs, validation and ports.
//!
//! Knows the domain and nothing else. No SeaORM, no reqwest, no filesystem, no Tauri.
//! See `docs/02-arquitectura.md` §2.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod config;
pub mod error;
pub mod paging;
pub mod ports;
pub mod result;
pub mod validation;

pub use error::{AppError, FieldError};
pub use paging::{PageRequest, PagedResult};
pub use result::AppResult;
