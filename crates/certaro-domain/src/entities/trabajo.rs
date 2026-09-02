use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::audit::Audit;
use crate::enums::EstadoTrabajo;
use crate::money::Money;

/// A contracted task inside a site. See `docs/05-dominio-entidades.md` §2.20.
///
/// There is no `cliente_id` here on purpose: the customer is reached through the site, and the
/// legacy denormalised column is exactly what made the "jobs by customer" filter wrong.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trabajo {
    pub id: Uuid,
    pub proyecto_id: Uuid,
    pub descripcion: String,
    /// Civil dates: what matters is the day, not the instant.
    pub fecha_inicio: NaiveDate,
    pub fecha_fin: Option<NaiveDate>,
    pub presupuesto: Money,
    pub estado: EstadoTrabajo,
    #[serde(flatten)]
    pub audit: Audit,
}

impl Trabajo {
    pub fn esta_abierto(&self) -> bool {
        self.estado.esta_abierto()
    }
}
