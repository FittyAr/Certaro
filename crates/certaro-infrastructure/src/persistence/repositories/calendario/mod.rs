//! SeaORM repositories for the Calendar module.

mod eventos;
mod recursos;

pub use eventos::SeaOrmCalendarioEventoRepository;
pub use recursos::{SeaOrmCalendarioGrupoRecursoRepository, SeaOrmCalendarioRecursoRepository};
