use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::decimal4::Decimal4;
use crate::entities::audit::Audit;
use crate::enums::TipoJornada;

/// One day of one employee. See `docs/05-dominio-entidades.md` §2.2.
///
/// The identity is `(empleado_id, fecha)`, not `id`: the grid writes by natural key and the unique
/// index enforces it, so a cell clicked twice updates rather than duplicating (INV-07).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsistenciaEmpleado {
    pub id: Uuid,
    pub empleado_id: Uuid,
    /// Civil date, stored as UTC midnight.
    pub fecha: NaiveDate,
    pub tipo_jornada: TipoJornada,
    pub trabajo_id: Option<Uuid>,
    pub observaciones: Option<String>,
    #[serde(flatten)]
    pub audit: Audit,
}

impl AsistenciaEmpleado {
    pub fn factor_jornada(&self) -> Decimal4 {
        self.tipo_jornada.factor()
    }
}

/// What the attendance grid shows next to each employee. See `docs/06-casos-de-uso-y-formulas.md`
/// §8.3, and RC-06.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumenAsistencia {
    pub completas: u32,
    pub medias: u32,
    pub faltas: u32,
    pub faltas_justificadas: u32,
    pub feriados: u32,
    /// Sum of the day factors: what the settlement would count.
    pub jornadas_equivalentes: Decimal4,
}

impl ResumenAsistencia {
    pub fn de_tipos<I: IntoIterator<Item = TipoJornada>>(tipos: I) -> Self {
        let mut resumen = Self::default();
        for tipo in tipos {
            match tipo {
                TipoJornada::Completa => resumen.completas += 1,
                TipoJornada::Media => resumen.medias += 1,
                TipoJornada::Falta => resumen.faltas += 1,
                TipoJornada::FaltaJustificada => resumen.faltas_justificadas += 1,
                TipoJornada::Feriado => resumen.feriados += 1,
            }
            resumen.jornadas_equivalentes = resumen
                .jornadas_equivalentes
                .checked_add(tipo.factor())
                .unwrap_or(resumen.jornadas_equivalentes);
        }
        resumen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_resumen_cuenta_cada_tipo_y_suma_los_factores() {
        // 2 completas + 1 media + 1 feriado = 2 + 0,5 + 1 = 3,5 jornadas
        let resumen = ResumenAsistencia::de_tipos([
            TipoJornada::Completa,
            TipoJornada::Completa,
            TipoJornada::Media,
            TipoJornada::Falta,
            TipoJornada::FaltaJustificada,
            TipoJornada::Feriado,
        ]);
        assert_eq!(resumen.completas, 2);
        assert_eq!(resumen.medias, 1);
        assert_eq!(resumen.faltas, 1);
        assert_eq!(resumen.faltas_justificadas, 1);
        assert_eq!(resumen.feriados, 1);
        assert_eq!(
            resumen.jornadas_equivalentes,
            Decimal4::parse("3.5").unwrap()
        );
    }

    #[test]
    fn un_periodo_sin_registros_no_suma_jornadas() {
        let resumen = ResumenAsistencia::de_tipos([]);
        assert_eq!(resumen.jornadas_equivalentes, Decimal4::ZERO);
    }
}
