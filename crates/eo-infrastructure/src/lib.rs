//! Infrastructure layer: everything that touches the outside world.
//!
//! Implements the ports declared in `eo-application`. Knows nothing about Tauri or the frontend.
//! See `docs/02-arquitectura.md` §2.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod config;
pub mod paths;
pub mod persistence;
pub mod telemetry;
