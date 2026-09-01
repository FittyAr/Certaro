use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::audit::Audit;

/// A customer. See `docs/05-dominio-entidades.md` §2.7.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cliente {
    pub id: Uuid,
    pub nombre: String,
    /// `XX-XXXXXXXX-X`. Deliberately not unique: the legacy data has customers with no CUIT at all
    /// and a few that share one.
    pub cuit: Option<String>,
    pub direccion: Option<String>,
    pub telefono: Option<String>,
    /// Kept for compatibility as the "main" address; the source of truth is `contactos` (RC-13).
    pub email: Option<String>,
    pub condicion_iva: Option<String>,
    /// Loaded by the repository. Empty on a customer read from a list query.
    pub contactos: Vec<ClienteContacto>,
    #[serde(flatten)]
    pub audit: Audit,
}

impl Cliente {
    /// The contact the interface preselects when offering to write an email.
    pub fn contacto_principal(&self) -> Option<&ClienteContacto> {
        self.contactos
            .iter()
            .find(|c| c.es_principal && !c.audit.is_deleted)
    }

    /// V-04: at most one contact may be flagged as the main one.
    pub fn tiene_un_solo_principal(&self) -> bool {
        self.contactos
            .iter()
            .filter(|c| c.es_principal && !c.audit.is_deleted)
            .count()
            <= 1
    }
}

/// One of the N contacts of a customer (RC-13). See `docs/05-dominio-entidades.md` §2.8.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClienteContacto {
    pub id: Uuid,
    pub cliente_id: Uuid,
    /// "Personal", "Oficina", "Compras"… Defaults to "General".
    pub etiqueta: String,
    pub email: String,
    pub nombre: Option<String>,
    pub telefono: Option<String>,
    pub es_principal: bool,
    #[serde(flatten)]
    pub audit: Audit,
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;

    fn ahora() -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(0, 0).unwrap()
    }

    fn contacto(n: u128, es_principal: bool) -> ClienteContacto {
        ClienteContacto {
            id: Uuid::from_u128(n),
            cliente_id: Uuid::from_u128(1),
            etiqueta: "General".into(),
            email: format!("c{n}@example.com"),
            nombre: None,
            telefono: None,
            es_principal,
            audit: Audit::new(ahora()),
        }
    }

    fn cliente(contactos: Vec<ClienteContacto>) -> Cliente {
        Cliente {
            id: Uuid::from_u128(1),
            nombre: "Tecnocasa".into(),
            cuit: None,
            direccion: None,
            telefono: None,
            email: None,
            condicion_iva: None,
            contactos,
            audit: Audit::new(ahora()),
        }
    }

    #[test]
    fn el_principal_es_el_marcado() {
        let c = cliente(vec![contacto(2, false), contacto(3, true)]);
        assert_eq!(
            c.contacto_principal().map(|c| c.id),
            Some(Uuid::from_u128(3))
        );
    }

    #[test]
    fn sin_marca_no_hay_principal() {
        let c = cliente(vec![contacto(2, false)]);
        assert!(c.contacto_principal().is_none());
    }

    #[test]
    fn dos_principales_rompen_la_regla() {
        assert!(!cliente(vec![contacto(2, true), contacto(3, true)]).tiene_un_solo_principal());
        assert!(cliente(vec![contacto(2, true), contacto(3, false)]).tiene_un_solo_principal());
    }

    #[test]
    fn un_principal_borrado_no_cuenta() {
        // Otherwise deleting the main contact and marking a new one would look like a violation.
        let mut borrado = contacto(2, true);
        borrado.audit.soft_delete(ahora());
        let c = cliente(vec![borrado, contacto(3, true)]);
        assert!(c.tiene_un_solo_principal());
        assert_eq!(
            c.contacto_principal().map(|c| c.id),
            Some(Uuid::from_u128(3))
        );
    }
}
