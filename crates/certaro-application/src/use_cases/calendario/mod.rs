use std::sync::Arc;

use crate::ports::clock::ClockPort;
use crate::ports::id_generator::IdGeneratorPort;
use crate::ports::repositories::UnitOfWork;

mod eventos;
mod recursos;

pub struct CalendarioService {
    pub(crate) uow: Arc<dyn UnitOfWork>,
    pub(crate) clock: Arc<dyn ClockPort>,
    pub(crate) id_gen: Arc<dyn IdGeneratorPort>,
}

impl CalendarioService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        id_gen: Arc<dyn IdGeneratorPort>,
    ) -> Self {
        Self { uow, clock, id_gen }
    }
}
