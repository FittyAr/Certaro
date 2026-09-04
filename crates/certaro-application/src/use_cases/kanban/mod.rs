use std::sync::Arc;

pub mod checklists_etiquetas;
pub mod columnas;
pub mod sync;
pub mod tableros;
pub mod tarjetas;

use crate::error::{AppError, FieldError};
use crate::ports::clock::ClockPort;
use crate::ports::id_generator::IdGeneratorPort;
use crate::ports::repositories::UnitOfWork;

pub(crate) fn validation_err(field: &'static str, message_key: &'static str) -> AppError {
    AppError::Validation(vec![FieldError::new(field, message_key)])
}

pub struct KanbanService {
    pub(crate) uow: Arc<dyn UnitOfWork>,
    pub(crate) clock: Arc<dyn ClockPort>,
    pub(crate) id_gen: Arc<dyn IdGeneratorPort>,
}

impl KanbanService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        id_gen: Arc<dyn IdGeneratorPort>,
    ) -> Self {
        Self {
            uow,
            clock,
            id_gen,
        }
    }
}
