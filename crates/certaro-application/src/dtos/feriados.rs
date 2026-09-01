//! Contract of the `feriados` module. See `docs/11-contratos-tauri.md` §5.13.

use chrono::NaiveDate;
use certaro_domain::entities::Feriado;
use certaro_domain::OrigenFeriado;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeriadoDto {
    pub fecha: NaiveDate,
    pub nombre: String,
    pub tipo: Option<String>,
    pub origen: OrigenFeriado,
}

impl From<Feriado> for FeriadoDto {
    fn from(f: Feriado) -> Self {
        Self {
            fecha: f.fecha,
            nombre: f.nombre,
            tipo: f.tipo,
            origen: f.origen,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeriadoInput {
    pub fecha: NaiveDate,
    pub nombre: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeriadoSyncResult {
    pub agregados: u64,
    pub total: u64,
    /// Years the provider could not be reached for. The sync is not an error: the calendar simply
    /// stays as it was.
    pub anios_con_error: u32,
}
