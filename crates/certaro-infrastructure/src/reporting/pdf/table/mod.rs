//! The table engine of the reports. See `docs/12-reportes-y-exportaciones.md` §1.1.
//!
//! What the layouts need and no PDF crate provides: relative and fixed column widths, cells merged
//! across columns and across rows, per-cell style, a header repeated on every page, and totals
//! pinned to the end.
//!
//! Everything is measured in points from the top of the page, like [`super::canvas`].

mod cell;
mod render;

#[cfg(test)]
mod tests;

pub use cell::*;
pub use render::*;
