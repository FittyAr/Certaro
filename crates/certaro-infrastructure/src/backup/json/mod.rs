//! The JSON dump of the database. See `docs/13-servicios-externos-y-archivos.md` §5.
//!
//! A verbatim dump, not a report: amounts stay scaled integers and dates stay the text SQLite holds.
//! Fidelity is the whole point — this has to be able to rebuild the database exactly.

mod common;
mod export;
mod import;

#[cfg(test)]
mod tests;

pub use common::{columnas_de, Documento, Tabla, FORMAT_VERSION, TABLAS};
pub use export::exportar;
pub use import::importar;
