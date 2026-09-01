//! V-01 of `docs/07-validaciones.md`.
//!
//! The document also lists `unidad` and `observaciones` rules for this DTO, but `movimientos` has
//! neither column in `docs/03-modelo-de-datos.md` §3.15, so there is nothing to validate. When the
//! columns are added the two rules come back here.

use chrono::NaiveDate;
use certaro_domain::constants::{limites, tipos_movimiento};
use certaro_domain::Money;
use uuid::Uuid;

use crate::dtos::movimientos::MovimientoInput;
use crate::error::FieldError;
use crate::result::AppResult;
use crate::validation::Validator;

/// Fallback lower bound when the configured one cannot be parsed. Catches the `01/01/0001` that a
/// mistyped year produces, which in the legacy system went straight into the database and broke
/// every report that grouped by month.
const FECHA_MINIMA: (i32, u32, u32) = (2000, 1, 1);

/// The date bounds, resolved from configuration once per call instead of being hardcoded here.
pub struct ContextoFecha {
    pub hoy: NaiveDate,
    /// From `Validation.FechaMinima`, default `2000-01-01`.
    pub minima: NaiveDate,
    /// From `Validation.FechaFuturaMaxDias`, default 365.
    pub max_dias_futuro: i64,
}

impl ContextoFecha {
    pub fn from_config(config: &crate::config::ValidationConfig, hoy: NaiveDate) -> Self {
        let minima = config
            .fecha_minima
            .parse::<NaiveDate>()
            .ok()
            .or_else(|| NaiveDate::from_ymd_opt(FECHA_MINIMA.0, FECHA_MINIMA.1, FECHA_MINIMA.2))
            .unwrap_or_default();
        Self {
            hoy,
            minima,
            max_dias_futuro: i64::from(config.fecha_futura_max_dias),
        }
    }
}

pub fn validate(input: &MovimientoInput, fechas: &ContextoFecha) -> AppResult<()> {
    let mut v = Validator::new();

    v.required_text(
        "concepto",
        &input.concepto,
        "Validation.Movimiento.ConceptoRequired",
    );
    v.max_length(
        "concepto",
        &input.concepto,
        limites::DESCRIPCION,
        "Validation.Movimiento.ConceptoMaxLength",
    );

    v.require(
        input.monto.is_positive(),
        FieldError::new("monto", "Validation.Movimiento.MontoRequired"),
    );
    // INV-02: a quantity of zero would make the total zero and the movement pointless; a negative
    // one would flip the sign behind the type's back.
    v.require(
        input.cantidad.is_positive(),
        FieldError::new("cantidad", "Validation.Movimiento.CantidadRequired"),
    );

    v.require(
        input.tipo_movimiento_id != Uuid::nil(),
        FieldError::new("tipoMovimientoId", "Validation.Movimiento.TipoRequired"),
    );
    // INV-03: the column is nullable for historical reasons, but a movement without a category
    // cannot be reported on, so the validator requires it.
    v.require(
        input.categoria_id.is_some_and(|id| id != Uuid::nil()),
        FieldError::new("categoriaId", "Validation.Movimiento.CategoriaRequired"),
    );

    validar_cotizacion(&mut v, input);
    validar_fecha(&mut v, input, fechas);

    // RC-05: an advance with no employee can never be discounted from a payroll, and the legacy
    // system produced exactly those orphans.
    if input.tipo_movimiento_id == tipos_movimiento::ADELANTO {
        v.require(
            input.empleado_id.is_some(),
            FieldError::new(
                "empleadoId",
                "Validation.Movimiento.EmpleadoRequeridoAdelanto",
            ),
        );
    }

    v.finish()
}

fn validar_cotizacion(v: &mut Validator, input: &MovimientoInput) {
    if input.moneda.requiere_cotizacion() {
        v.require(
            input.cotizacion_aplicada.is_some_and(|c| c.is_positive()),
            FieldError::new(
                "cotizacionAplicada",
                "Validation.Movimiento.CotizacionRequired",
            ),
        );
    } else {
        // A rate on a peso movement is meaningless and would be applied by any later conversion.
        v.require(
            input.cotizacion_aplicada.map_or(true, |c| c == Money::ZERO),
            FieldError::new(
                "cotizacionAplicada",
                "Validation.Movimiento.CotizacionNotApplicable",
            ),
        );
    }
}

fn validar_fecha(v: &mut Validator, input: &MovimientoInput, fechas: &ContextoFecha) {
    let minima = fechas.minima;
    let fecha = input.fecha.date_naive();
    let maxima = fechas.hoy + chrono::Duration::days(fechas.max_dias_futuro);

    v.require(
        fecha >= minima && fecha <= maxima,
        FieldError::new("fecha", "Validation.Common.FechaOutOfRange")
            .with_param("min", minima.to_string())
            .with_param("max", maxima.to_string()),
    );
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};
    use certaro_domain::{Decimal4, Moneda};

    use super::*;
    use crate::AppError;

    fn contexto() -> ContextoFecha {
        ContextoFecha::from_config(
            &crate::config::ValidationConfig::default(),
            NaiveDate::from_ymd_opt(2026, 8, 29).unwrap(),
        )
    }

    fn instante(year: i32, month: u32, day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap()
    }

    fn input() -> MovimientoInput {
        MovimientoInput {
            fecha: instante(2026, 8, 29),
            concepto: "Cable 2.5".into(),
            monto: Money::parse("1500.0000").unwrap(),
            cantidad: Decimal4::ONE,
            tipo_movimiento_id: tipos_movimiento::GASTO,
            moneda: Moneda::Ars,
            cotizacion_aplicada: None,
            tipo_concepto_pago_id: None,
            categoria_id: Some(Uuid::from_u128(7)),
            cliente_id: None,
            trabajo_id: None,
            empleado_id: None,
            factura_id: None,
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
    fn un_movimiento_completo_pasa() {
        assert!(validate(&input(), &contexto()).is_ok());
    }

    #[test]
    fn el_concepto_admite_quinientos_caracteres_y_no_quinientos_uno() {
        // The legacy validator said 200 while the column said 500, so a valid concept was
        // rejected by the form and accepted by an import.
        let dto = MovimientoInput {
            concepto: "a".repeat(500),
            ..input()
        };
        assert!(validate(&dto, &contexto()).is_ok());

        let dto = MovimientoInput {
            concepto: "a".repeat(501),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto, &contexto()).unwrap_err()),
            ["Validation.Movimiento.ConceptoMaxLength"]
        );
    }

    #[test]
    fn el_monto_y_la_cantidad_tienen_que_ser_positivos() {
        let dto = MovimientoInput {
            monto: Money::ZERO,
            ..input()
        };
        assert_eq!(
            keys(validate(&dto, &contexto()).unwrap_err()),
            ["Validation.Movimiento.MontoRequired"]
        );

        let dto = MovimientoInput {
            cantidad: Decimal4::ZERO,
            ..input()
        };
        assert_eq!(
            keys(validate(&dto, &contexto()).unwrap_err()),
            ["Validation.Movimiento.CantidadRequired"]
        );
    }

    #[test]
    fn la_categoria_es_obligatoria_aunque_la_columna_admita_nulo() {
        let dto = MovimientoInput {
            categoria_id: None,
            ..input()
        };
        assert_eq!(
            keys(validate(&dto, &contexto()).unwrap_err()),
            ["Validation.Movimiento.CategoriaRequired"]
        );
    }

    #[test]
    fn en_dolares_la_cotizacion_es_obligatoria() {
        let dto = MovimientoInput {
            moneda: Moneda::Usd,
            ..input()
        };
        assert_eq!(
            keys(validate(&dto, &contexto()).unwrap_err()),
            ["Validation.Movimiento.CotizacionRequired"]
        );

        let dto = MovimientoInput {
            moneda: Moneda::Usd,
            cotizacion_aplicada: Some(Money::parse("1350.0000").unwrap()),
            ..input()
        };
        assert!(validate(&dto, &contexto()).is_ok());
    }

    #[test]
    fn en_pesos_una_cotizacion_no_tiene_sentido() {
        let dto = MovimientoInput {
            cotizacion_aplicada: Some(Money::parse("1350.0000").unwrap()),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto, &contexto()).unwrap_err()),
            ["Validation.Movimiento.CotizacionNotApplicable"]
        );
    }

    #[test]
    fn una_fecha_del_ano_uno_se_rechaza() {
        let dto = MovimientoInput {
            fecha: instante(1, 1, 1),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto, &contexto()).unwrap_err()),
            ["Validation.Common.FechaOutOfRange"]
        );
    }

    #[test]
    fn el_limite_futuro_es_un_ano_por_defecto() {
        let dto = MovimientoInput {
            fecha: instante(2027, 8, 29),
            ..input()
        };
        assert!(validate(&dto, &contexto()).is_ok());

        let dto = MovimientoInput {
            fecha: instante(2027, 8, 31),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto, &contexto()).unwrap_err()),
            ["Validation.Common.FechaOutOfRange"]
        );
    }

    #[test]
    fn un_adelanto_sin_empleado_no_se_puede_liquidar() {
        let dto = MovimientoInput {
            tipo_movimiento_id: tipos_movimiento::ADELANTO,
            ..input()
        };
        assert_eq!(
            keys(validate(&dto, &contexto()).unwrap_err()),
            ["Validation.Movimiento.EmpleadoRequeridoAdelanto"]
        );

        let dto = MovimientoInput {
            tipo_movimiento_id: tipos_movimiento::ADELANTO,
            empleado_id: Some(Uuid::from_u128(3)),
            ..input()
        };
        assert!(validate(&dto, &contexto()).is_ok());
    }
}
