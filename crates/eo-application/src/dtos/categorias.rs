//! Contract of the `categorias` module. See `docs/11-contratos-tauri.md` §5.12.

use eo_domain::entities::Categoria;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dtos::common::AuditDto;
use crate::ports::repositories::{CategoriaConUso, CategoriaFiltro};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoriaFiltroDto {
    pub texto: Option<String>,
    pub categoria_padre_id: Option<Uuid>,
    /// Asks for the root categories only. Distinct from not filtering by parent at all, which is
    /// why a plain `Option<Uuid>` cannot express it.
    #[serde(default)]
    pub solo_raiz: bool,
}

impl From<CategoriaFiltroDto> for CategoriaFiltro {
    fn from(dto: CategoriaFiltroDto) -> Self {
        let padre = if dto.solo_raiz {
            Some(None)
        } else {
            dto.categoria_padre_id.map(Some)
        };
        Self {
            texto: dto.texto.filter(|t| !t.trim().is_empty()),
            categoria_padre_id: padre,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoriaInput {
    pub nombre: String,
    pub descripcion: Option<String>,
    pub color_hex: Option<String>,
    pub icono: Option<String>,
    pub categoria_padre_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoriaListItem {
    pub id: Uuid,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub color_hex: Option<String>,
    pub icono: Option<String>,
    pub categoria_padre_id: Option<Uuid>,
    pub categoria_padre_nombre: Option<String>,
    pub movimientos_count: u64,
    pub hijas_count: u64,
    /// Precomputed so the interface can disable the action instead of letting the user discover
    /// the refusal by trying.
    pub puede_eliminarse: bool,
    pub row_version: String,
}

impl From<CategoriaConUso> for CategoriaListItem {
    fn from(row: CategoriaConUso) -> Self {
        let libre = row.movimientos_count == 0 && row.hijas_count == 0;
        Self {
            id: row.categoria.id,
            nombre: row.categoria.nombre,
            descripcion: row.categoria.descripcion,
            color_hex: row.categoria.color_hex,
            icono: row.categoria.icono,
            categoria_padre_id: row.categoria.categoria_padre_id,
            categoria_padre_nombre: row.padre_nombre,
            movimientos_count: row.movimientos_count,
            hijas_count: row.hijas_count,
            puede_eliminarse: libre,
            row_version: row.categoria.audit.row_version.to_hex(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoriaDetalle {
    pub id: Uuid,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub color_hex: Option<String>,
    pub icono: Option<String>,
    pub categoria_padre_id: Option<Uuid>,
    pub movimientos_count: u64,
    pub hijas_count: u64,
    pub puede_eliminarse: bool,
    pub audit: AuditDto,
}

impl CategoriaDetalle {
    pub fn build(categoria: &Categoria, movimientos_count: u64, hijas_count: u64) -> Self {
        Self {
            id: categoria.id,
            nombre: categoria.nombre.clone(),
            descripcion: categoria.descripcion.clone(),
            color_hex: categoria.color_hex.clone(),
            icono: categoria.icono.clone(),
            categoria_padre_id: categoria.categoria_padre_id,
            movimientos_count,
            hijas_count,
            puede_eliminarse: movimientos_count == 0 && hijas_count == 0,
            audit: AuditDto::from(&categoria.audit),
        }
    }
}
