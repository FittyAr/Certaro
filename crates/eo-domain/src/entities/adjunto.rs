use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::Audit;
use crate::error::DomainError;

/// A file attached to a record. See `docs/05-dominio-entidades.md` §2.1.
///
/// The relation is polymorphic and therefore has no foreign key: `entidad_tipo` plus `entidad_id`
/// says what it hangs from. That is what makes the closed enum below load-bearing — a misspelled
/// type in the legacy free-text column left the attachment orphaned and invisible.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Adjunto {
    pub id: Uuid,
    pub entidad_tipo: EntidadAdjunto,
    pub entidad_id: Uuid,
    /// Original name, already sanitised. What the user reads in the list.
    pub nombre_archivo: String,
    /// Relative to the attachments root, always with `/` as separator so the value is portable
    /// between platforms. See `docs/13-servicios-externos-y-archivos.md` §1.2.
    pub ruta_relativa: String,
    pub mime: String,
    /// Size in bytes. Not scaled: this is a count, not an amount.
    pub tamano: u64,
    pub audit: Audit,
}

impl Adjunto {
    /// The relative path of a new attachment: `{tipo}/{id}/{uuid}_{nombre}`.
    ///
    /// Built here rather than in the adapter so the convention is part of the domain and a test can
    /// pin it without touching the disk.
    #[must_use]
    pub fn ruta_para(
        entidad_tipo: EntidadAdjunto,
        entidad_id: Uuid,
        id: Uuid,
        nombre_saneado: &str,
    ) -> String {
        format!(
            "{}/{}/{}_{}",
            entidad_tipo.as_str(),
            entidad_id,
            id,
            nombre_saneado
        )
    }

    /// Instant the file was attached. The list is ordered by this, newest first.
    #[must_use]
    pub const fn adjuntado_en(&self) -> DateTime<Utc> {
        self.audit.created_at
    }
}

/// What an attachment can hang from. See `docs/05-dominio-entidades.md` §3.7.
///
/// Persisted as its **name**, not as a number, so the rows the legacy system already wrote keep
/// meaning what they meant.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EntidadAdjunto {
    Obra,
    Trabajo,
    Factura,
    #[default]
    Movimiento,
    Empleado,
}

impl EntidadAdjunto {
    pub const ALL: [Self; 5] = [
        Self::Obra,
        Self::Trabajo,
        Self::Factura,
        Self::Movimiento,
        Self::Empleado,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Obra => "Obra",
            Self::Trabajo => "Trabajo",
            Self::Factura => "Factura",
            Self::Movimiento => "Movimiento",
            Self::Empleado => "Empleado",
        }
    }

    pub fn parse(value: &str) -> Result<Self, DomainError> {
        Self::ALL
            .into_iter()
            .find(|variant| variant.as_str() == value)
            .ok_or(DomainError::UnknownEnumValue {
                enum_name: "EntidadAdjunto",
                value: -1,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_tipo_va_y_vuelve_por_su_nombre() {
        for tipo in EntidadAdjunto::ALL {
            assert_eq!(EntidadAdjunto::parse(tipo.as_str()).unwrap(), tipo);
        }
    }

    #[test]
    fn un_tipo_desconocido_no_se_adivina() {
        // The legacy column was free text, so this is exactly the value that used to slip through.
        assert!(EntidadAdjunto::parse("movimiento").is_err());
        assert!(EntidadAdjunto::parse("Certificado").is_err());
    }

    #[test]
    fn la_ruta_sigue_la_convencion_con_barras() {
        let ruta = Adjunto::ruta_para(
            EntidadAdjunto::Movimiento,
            Uuid::from_u128(1),
            Uuid::from_u128(2),
            "factura_luz.pdf",
        );
        assert_eq!(
            ruta,
            "Movimiento/00000000-0000-0000-0000-000000000001/00000000-0000-0000-0000-000000000002_factura_luz.pdf"
        );
        assert!(!ruta.contains('\\'));
    }
}
