//! Use cases of `facturas` and its payments. See `docs/09-modulos-funcionales.md` §3.8.

use std::sync::Arc;

use chrono::NaiveDate;
use crate::ports::clock::ClockPort;
use crate::ports::id_generator::IdGeneratorPort;
use crate::ports::repositories::UnitOfWork;
use crate::ports::settings::SettingsStore;
use crate::validation::movimientos::ContextoFecha;

mod factura_crud;
mod pagos;

pub(crate) const ENTITY: &str = "Factura";
pub(crate) const ENTITY_PAGO: &str = "PagoFactura";

pub struct FacturasService {
    pub(crate) uow: Arc<dyn UnitOfWork>,
    pub(crate) clock: Arc<dyn ClockPort>,
    pub(crate) ids: Arc<dyn IdGeneratorPort>,
    pub(crate) settings: Arc<dyn SettingsStore>,
}

impl FacturasService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        ids: Arc<dyn IdGeneratorPort>,
        settings: Arc<dyn SettingsStore>,
    ) -> Self {
        Self {
            uow,
            clock,
            ids,
            settings,
        }
    }

    /// Today as a civil date. Everything overdue-related is measured in days, so the instant is
    /// noise; what matters is which day it is.
    pub(crate) fn hoy(&self) -> NaiveDate {
        self.clock.now_utc().date_naive()
    }

    pub(crate) fn dias_vencimiento(&self) -> u32 {
        self.settings
            .snapshot()
            .business
            .factura_dias_vencimiento_default
    }

    pub(crate) fn contexto_fecha(&self, hoy: NaiveDate) -> ContextoFecha {
        ContextoFecha::from_config(&self.settings.snapshot().validation, hoy)
    }
}
