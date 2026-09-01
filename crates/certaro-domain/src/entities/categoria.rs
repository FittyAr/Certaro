use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::audit::Audit;

/// A spending or income category, optionally hanging from a parent one (RC-04).
/// See `docs/05-dominio-entidades.md` §2.4.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Categoria {
    pub id: Uuid,
    pub nombre: String,
    pub descripcion: Option<String>,
    /// `#RRGGBB` in upper case, or nothing. The interface picks a neutral token when absent.
    pub color_hex: Option<String>,
    pub icono: Option<String>,
    pub categoria_padre_id: Option<Uuid>,
    #[serde(flatten)]
    pub audit: Audit,
}

impl Categoria {
    pub fn es_raiz(&self) -> bool {
        self.categoria_padre_id.is_none()
    }

    /// A category cannot be its own parent. Longer cycles (A → B → A) need the database and are
    /// checked as a business rule, not here.
    pub fn padre_es_valido(&self) -> bool {
        self.categoria_padre_id != Some(self.id)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;

    fn categoria(id: Uuid, padre: Option<Uuid>) -> Categoria {
        Categoria {
            id,
            nombre: "Materiales".into(),
            descripcion: None,
            color_hex: None,
            icono: None,
            categoria_padre_id: padre,
            audit: Audit::new(DateTime::<Utc>::from_timestamp(0, 0).unwrap()),
        }
    }

    #[test]
    fn sin_padre_es_una_categoria_raiz() {
        assert!(categoria(Uuid::from_u128(1), None).es_raiz());
        assert!(!categoria(Uuid::from_u128(1), Some(Uuid::from_u128(2))).es_raiz());
    }

    #[test]
    fn una_categoria_no_puede_ser_su_propio_padre() {
        let id = Uuid::from_u128(1);
        assert!(!categoria(id, Some(id)).padre_es_valido());
        assert!(categoria(id, Some(Uuid::from_u128(2))).padre_es_valido());
    }
}
