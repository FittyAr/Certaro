use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::constants;
use crate::entities::audit::Audit;

/// Primary classification of a movement. See `docs/05-dominio-entidades.md` §2.19.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TipoMovimiento {
    pub id: Uuid,
    pub nombre: String,
    pub descripcion: Option<String>,
    /// `true` adds to the balance, `false` subtracts from it.
    pub es_ingreso: bool,
    pub es_sistema: bool,
    #[serde(flatten)]
    pub audit: Audit,
}

impl TipoMovimiento {
    /// A protected row cannot be deleted and its sign cannot change, because the historical
    /// balance was computed with that sign.
    pub fn es_de_sistema_protegido(&self) -> bool {
        self.es_sistema || constants::tipos_movimiento::TODOS.contains(&self.id)
    }

    /// The sign with which a movement of this type enters the balance.
    pub fn signo(&self) -> i64 {
        if self.es_ingreso {
            1
        } else {
            -1
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;

    fn tipo(id: Uuid, es_sistema: bool, es_ingreso: bool) -> TipoMovimiento {
        TipoMovimiento {
            id,
            nombre: "Ingreso".into(),
            descripcion: None,
            es_ingreso,
            es_sistema,
            audit: Audit::new(DateTime::<Utc>::from_timestamp(0, 0).unwrap()),
        }
    }

    #[test]
    fn a_seeded_identifier_is_protected_even_if_the_flag_was_cleared() {
        let seeded = tipo(constants::tipos_movimiento::ADELANTO, false, false);
        assert!(seeded.es_de_sistema_protegido());
    }

    #[test]
    fn a_row_created_by_the_user_is_not_protected() {
        assert!(!tipo(Uuid::from_u128(42), false, true).es_de_sistema_protegido());
    }

    #[test]
    fn the_sign_follows_the_income_flag() {
        assert_eq!(tipo(Uuid::from_u128(1), false, true).signo(), 1);
        assert_eq!(tipo(Uuid::from_u128(1), false, false).signo(), -1);
    }
}
