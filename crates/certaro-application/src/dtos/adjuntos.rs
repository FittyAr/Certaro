//! DTOs of attachments. See `docs/11-contratos-tauri.md` §5.12.

use chrono::{DateTime, Utc};
use certaro_domain::entities::{Adjunto, EntidadAdjunto};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdjuntoItem {
    pub id: Uuid,
    pub entidad_tipo: EntidadAdjunto,
    pub entidad_id: Uuid,
    pub nombre_archivo: String,
    pub mime: String,
    /// Bytes. The interface turns this into KB or MB; the backend does not format it.
    pub tamano: u64,
    pub adjuntado_en: DateTime<Utc>,
}

impl From<Adjunto> for AdjuntoItem {
    fn from(entity: Adjunto) -> Self {
        Self {
            id: entity.id,
            entidad_tipo: entity.entidad_tipo,
            entidad_id: entity.entidad_id,
            adjuntado_en: entity.adjuntado_en(),
            nombre_archivo: entity.nombre_archivo,
            mime: entity.mime,
            tamano: entity.tamano,
        }
    }
}

/// What the frontend sends to attach a file. `ruta_origen` is an absolute path the user picked in
/// the system dialog, so the backend treats it as untrusted input.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdjuntoInput {
    pub entidad_tipo: EntidadAdjunto,
    pub entidad_id: Uuid,
    pub ruta_origen: String,
}
