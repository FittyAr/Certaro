//! V-14 of `docs/07-validaciones.md`.

use certaro_domain::constants::limites;
use uuid::Uuid;

use crate::dtos::asistencias::{AsistenciaRangoInput, AsistenciaUpsertInput};
use crate::error::FieldError;
use crate::result::AppResult;
use crate::validation::movimientos::ContextoFecha;
use crate::validation::Validator;

/// Uniqueness by `(empleado_id, fecha)` is not checked here: the unique index enforces it and the
/// upsert resolves it, so a shape check would only duplicate the rule badly.
pub fn validate(input: &AsistenciaUpsertInput, fechas: &ContextoFecha) -> AppResult<()> {
    let mut v = Validator::new();

    v.require(
        input.empleado_id != Uuid::nil(),
        FieldError::new("empleadoId", "Validation.Asistencia.EmpleadoRequired"),
    );

    validar_fecha(&mut v, "fecha", input.fecha, fechas);

    v.max_length_opt(
        "observaciones",
        input.observaciones.as_deref(),
        limites::OBSERVACIONES,
        "Validation.Asistencia.ObservacionesMaxLength",
    );

    v.finish()
}

pub fn validate_rango(input: &AsistenciaRangoInput, fechas: &ContextoFecha) -> AppResult<()> {
    let mut v = Validator::new();

    v.require(
        input.empleado_id != Uuid::nil(),
        FieldError::new("empleadoId", "Validation.Asistencia.EmpleadoRequired"),
    );
    v.require(
        input.desde <= input.hasta,
        FieldError::new("hasta", "Validation.Asistencia.RangoInvalid"),
    );
    validar_fecha(&mut v, "desde", input.desde, fechas);
    validar_fecha(&mut v, "hasta", input.hasta, fechas);

    v.finish()
}

fn validar_fecha(v: &mut Validator, campo: &str, fecha: chrono::NaiveDate, fechas: &ContextoFecha) {
    let maxima = fechas.hoy + chrono::Duration::days(fechas.max_dias_futuro);
    v.require(
        fecha >= fechas.minima && fecha <= maxima,
        FieldError::new(campo, "Validation.Common.FechaOutOfRange")
            .with_param("minima", fechas.minima)
            .with_param("maxima", maxima),
    );
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use certaro_domain::TipoJornada;

    use super::*;
    use crate::AppError;

    fn contexto() -> ContextoFecha {
        ContextoFecha {
            minima: NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
            hoy: NaiveDate::from_ymd_opt(2026, 8, 29).unwrap(),
            max_dias_futuro: 365,
        }
    }

    fn input() -> AsistenciaUpsertInput {
        AsistenciaUpsertInput {
            empleado_id: Uuid::from_u128(1),
            fecha: NaiveDate::from_ymd_opt(2026, 8, 3).unwrap(),
            tipo_jornada: Some(TipoJornada::Completa),
            trabajo_id: None,
            observaciones: None,
        }
    }

    fn keys(error: AppError) -> Vec<String> {
        error
            .fields()
            .iter()
            .map(|f| f.message_key.clone())
            .collect()
    }

    #[test]
    fn una_celda_minima_es_valida() {
        assert!(validate(&input(), &contexto()).is_ok());
    }

    #[test]
    fn borrar_una_celda_es_valido() {
        let dto = AsistenciaUpsertInput {
            tipo_jornada: None,
            ..input()
        };
        assert!(validate(&dto, &contexto()).is_ok());
    }

    #[test]
    fn el_empleado_es_obligatorio() {
        let dto = AsistenciaUpsertInput {
            empleado_id: Uuid::nil(),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto, &contexto()).unwrap_err()),
            ["Validation.Asistencia.EmpleadoRequired"]
        );
    }

    #[test]
    fn una_fecha_fuera_de_rango_se_rechaza() {
        let dto = AsistenciaUpsertInput {
            fecha: NaiveDate::from_ymd_opt(1999, 12, 31).unwrap(),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto, &contexto()).unwrap_err()),
            ["Validation.Common.FechaOutOfRange"]
        );
    }

    #[test]
    fn un_rango_invertido_se_rechaza() {
        let dto = AsistenciaRangoInput {
            empleado_id: Uuid::from_u128(1),
            desde: NaiveDate::from_ymd_opt(2026, 8, 10).unwrap(),
            hasta: NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
            tipo_jornada: TipoJornada::Completa,
            solo_dias_habiles: true,
            trabajo_id: None,
        };
        assert_eq!(
            keys(validate_rango(&dto, &contexto()).unwrap_err()),
            ["Validation.Asistencia.RangoInvalid"]
        );
    }
}
