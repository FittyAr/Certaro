use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::audit::Audit;
use crate::enums::EstadoProyecto;

/// A physical site, identified by a number the customer also uses (RC-07).
/// See `docs/05-dominio-entidades.md` §2.14.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proyecto {
    pub id: Uuid,
    /// Unique across every site ever created, deleted ones included (INV-06).
    pub numero: i32,
    pub nombre: String,
    pub direccion: Option<String>,
    pub localidad: Option<String>,
    pub cliente_id: Uuid,
    pub estado: EstadoProyecto,
    #[serde(flatten)]
    pub audit: Audit,
}

impl Proyecto {
    /// A job can only be started, resumed or reopened while its site is running.
    pub fn esta_activa(&self) -> bool {
        self.estado == EstadoProyecto::Activa
    }
}
