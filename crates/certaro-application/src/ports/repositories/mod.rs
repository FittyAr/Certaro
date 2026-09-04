//! Repository ports. See `docs/02-arquitectura.md` §5.
//!
//! Use cases only ever see these traits, so the domain can be exercised without a database and a
//! change of ORM stays inside the infrastructure crate.

pub mod auth;
pub mod comercial;
pub mod common;
pub mod dashboard;
pub mod movimientos;
pub mod operaciones;
pub mod personal;
pub mod proyectos;
pub mod sistema;
pub mod uow;

pub use auth::*;
pub use comercial::*;
pub use common::*;
pub use dashboard::*;
pub use movimientos::*;
pub use operaciones::*;
pub use personal::*;
pub use proyectos::*;
pub use sistema::*;
pub use uow::*;
