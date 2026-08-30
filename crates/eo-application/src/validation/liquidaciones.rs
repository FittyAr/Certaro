//! V-15 and V-16 of `docs/07-validaciones.md`.

use eo_domain::constants::limites;
use uuid::Uuid;

use crate::dtos::liquidaciones::{LiquidacionBatchInput, LiquidacionInput, LiquidacionUpdateInput};
use crate::error::FieldError;
use crate::result::AppResult;
use crate::validation::Validator;

/// Note what is **not** here: `total_adelantos <= total_bruto`. An employee who took more in
/// advances than they earned is a real case; the interface paints the net red and records it.
pub fn validate(input: &LiquidacionInput) -> AppResult<()> {
    let mut v = Validator::new();
    validar_uno(&mut v, "", input);
    v.finish()
}

pub fn validate_batch(input: &LiquidacionBatchInput) -> AppResult<()> {
    let mut v = Validator::new();

    v.require(
        !input.dtos.is_empty(),
        FieldError::new("dtos", "Validation.Liquidacion.BatchEmpty"),
    );

    for (i, dto) in input.dtos.iter().enumerate() {
        validar_uno(&mut v, &format!("dtos[{i}]."), dto);
    }

    v.finish()
}

pub fn validate_update(input: &LiquidacionUpdateInput) -> AppResult<()> {
    let mut v = Validator::new();

    v.require(
        input.dias_trabajados.is_positive(),
        FieldError::new(
            "diasTrabajados",
            "Validation.Liquidacion.DiasTrabajadosRequired",
        ),
    );
    v.require(
        input.tarifa_aplicada.is_positive(),
        FieldError::new("tarifaAplicada", "Validation.Liquidacion.TarifaRequired"),
    );
    v.require(
        !input.total_bruto.is_negative(),
        FieldError::new("totalBruto", "Validation.Liquidacion.BrutoNegative"),
    );
    v.require(
        !input.total_adelantos.is_negative(),
        FieldError::new("totalAdelantos", "Validation.Liquidacion.AdelantosNegative"),
    );
    v.max_length_opt(
        "observaciones",
        input.observaciones.as_deref(),
        limites::OBSERVACIONES,
        "Validation.Liquidacion.ObservacionesMaxLength",
    );

    v.finish()
}

fn validar_uno(v: &mut Validator, prefijo: &str, input: &LiquidacionInput) {
    let campo = |nombre: &str| format!("{prefijo}{nombre}");

    v.require(
        input.empleado_id != Uuid::nil(),
        FieldError::new(
            campo("empleadoId"),
            "Validation.Liquidacion.EmpleadoRequired",
        ),
    );

    // INV-17.
    v.require(
        input.fecha_inicio <= input.fecha_fin,
        FieldError::new(
            campo("fechaInicio"),
            "Validation.Liquidacion.FechaInicioInvalid",
        ),
    );

    v.require(
        input.dias_trabajados.is_positive(),
        FieldError::new(
            campo("diasTrabajados"),
            "Validation.Liquidacion.DiasTrabajadosRequired",
        ),
    );
    v.require(
        input.tarifa_aplicada.is_positive(),
        FieldError::new(
            campo("tarifaAplicada"),
            "Validation.Liquidacion.TarifaRequired",
        ),
    );
    v.require(
        !input.total_bruto.is_negative(),
        FieldError::new(campo("totalBruto"), "Validation.Liquidacion.BrutoNegative"),
    );
    v.require(
        !input.total_adelantos.is_negative(),
        FieldError::new(
            campo("totalAdelantos"),
            "Validation.Liquidacion.AdelantosNegative",
        ),
    );

    for (i, adelanto) in input.adelantos.iter().enumerate() {
        v.require(
            adelanto.movimiento_id != Uuid::nil(),
            FieldError::new(
                campo(&format!("adelantos[{i}].movimientoId")),
                "Validation.Liquidacion.AdelantoMovimientoRequired",
            ),
        );
        v.max_length(
            &campo(&format!("adelantos[{i}].concepto")),
            &adelanto.concepto,
            limites::DESCRIPCION,
            "Validation.Liquidacion.AdelantoConceptoMaxLength",
        );
    }

    v.max_length_opt(
        &campo("observaciones"),
        input.observaciones.as_deref(),
        limites::OBSERVACIONES,
        "Validation.Liquidacion.ObservacionesMaxLength",
    );
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use eo_domain::{Decimal4, Money};

    use super::*;
    use crate::AppError;

    fn dia(d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, d).unwrap()
    }

    fn input() -> LiquidacionInput {
        LiquidacionInput {
            empleado_id: Uuid::from_u128(1),
            fecha_inicio: dia(1),
            fecha_fin: dia(15),
            dias_trabajados: Decimal4::from_units(10).unwrap(),
            tarifa_aplicada: Money::from_units(40_000).unwrap(),
            incluir_sabados: false,
            incluir_domingos: false,
            incluir_feriados: false,
            multiplicador_sabado: Decimal4::ONE,
            multiplicador_domingo: Decimal4::ONE,
            multiplicador_feriado: Decimal4::ONE,
            total_bruto: Money::from_units(400_000).unwrap(),
            total_adelantos: Money::from_units(260_000).unwrap(),
            observaciones: None,
            adelantos: Vec::new(),
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
    fn una_liquidacion_minima_es_valida() {
        assert!(validate(&input()).is_ok());
    }

    #[test]
    fn el_fin_no_puede_preceder_al_inicio() {
        let dto = LiquidacionInput {
            fecha_fin: dia(1) - chrono::Duration::days(1),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.Liquidacion.FechaInicioInvalid"]
        );
    }

    #[test]
    fn un_neto_negativo_es_valido() {
        // Taking more in advances than earned happens, and refusing to record it would leave the
        // debt untracked.
        let dto = LiquidacionInput {
            total_bruto: Money::from_units(100_000).unwrap(),
            total_adelantos: Money::from_units(150_000).unwrap(),
            ..input()
        };
        assert!(validate(&dto).is_ok());
    }

    #[test]
    fn los_dias_deben_ser_positivos() {
        let dto = LiquidacionInput {
            dias_trabajados: Decimal4::ZERO,
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.Liquidacion.DiasTrabajadosRequired"]
        );
    }

    #[test]
    fn un_lote_vacio_se_rechaza() {
        let dto = LiquidacionBatchInput { dtos: Vec::new() };
        assert_eq!(
            keys(validate_batch(&dto).unwrap_err()),
            ["Validation.Liquidacion.BatchEmpty"]
        );
    }

    #[test]
    fn el_lote_indica_que_elemento_falla() {
        let mala = LiquidacionInput {
            tarifa_aplicada: Money::ZERO,
            ..input()
        };
        let dto = LiquidacionBatchInput {
            dtos: vec![input(), mala],
        };
        let error = validate_batch(&dto).unwrap_err();
        assert_eq!(error.fields()[0].field, "dtos[1].tarifaAplicada");
    }
}
