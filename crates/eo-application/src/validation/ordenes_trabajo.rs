//! V-08 and V-09 of `docs/07-validaciones.md`.

use eo_domain::constants::limites;
use eo_domain::Decimal4;
use uuid::Uuid;

use crate::dtos::ordenes_trabajo::{OrdenTrabajoInput, OrdenTrabajoItemInput};
use crate::error::FieldError;
use crate::result::AppResult;
use crate::validation::Validator;

/// V-08. The items are validated in the same pass so one submit reports every bad line.
pub fn validate(input: &OrdenTrabajoInput) -> AppResult<()> {
    let mut v = Validator::new();

    v.required_text(
        "titulo",
        &input.titulo,
        "Validation.OrdenTrabajo.TituloRequired",
    );
    v.max_length(
        "titulo",
        &input.titulo,
        limites::NOMBRE_LARGO,
        "Validation.OrdenTrabajo.TituloMaxLength",
    );

    v.require(
        input.trabajo_id != Uuid::nil(),
        FieldError::new("trabajoId", "Validation.OrdenTrabajo.TrabajoRequired"),
    );

    // An order with no items certifies nothing, so it is not an order yet.
    v.require(
        !input.items.is_empty(),
        FieldError::new("items", "Validation.OrdenTrabajo.ItemsRequired"),
    );

    v.max_length_opt(
        "observaciones",
        input.observaciones.as_deref(),
        limites::OBSERVACIONES,
        "Validation.OrdenTrabajo.ObservacionesMaxLength",
    );

    // The UOCRA adjustment is a percentage of what was certified, so the same [0, 100] range as
    // any other percentage in the system.
    v.require(
        input.ajuste_uocra_porcentaje.is_valid_percentage(),
        FieldError::new(
            "ajusteUocraPorcentaje",
            "Validation.OrdenTrabajo.AjusteInvalid",
        ),
    );
    v.require(
        !input.otros_descuentos.is_negative(),
        FieldError::new(
            "otrosDescuentos",
            "Validation.OrdenTrabajo.DescuentoNegative",
        ),
    );

    for (i, item) in input.items.iter().enumerate() {
        validar_item(&mut v, i, item);
    }

    v.finish()
}

/// V-09. Errors are addressed as `items[i].campo` so the grid can highlight the offending cell.
fn validar_item(v: &mut Validator, index: usize, item: &OrdenTrabajoItemInput) {
    let campo = |name: &str| format!("items[{index}].{name}");

    v.required_text(
        &campo("descripcion"),
        &item.descripcion,
        "Validation.OrdenTrabajoItem.DescripcionRequired",
    );
    v.max_length(
        &campo("descripcion"),
        &item.descripcion,
        limites::DESCRIPCION,
        "Validation.OrdenTrabajoItem.DescripcionMaxLength",
    );
    v.max_length(
        &campo("unidad"),
        &item.unidad,
        limites::UNIDAD,
        "Validation.OrdenTrabajoItem.UnidadMaxLength",
    );

    v.require(
        item.cantidad.is_positive(),
        FieldError::new(
            campo("cantidad"),
            "Validation.OrdenTrabajoItem.CantidadRequired",
        ),
    );
    v.require(
        !item.precio_unitario.is_negative(),
        FieldError::new(
            campo("precioUnitario"),
            "Validation.OrdenTrabajoItem.PrecioNegative",
        ),
    );
    v.require(
        item.porcentaje_actual.is_valid_percentage(),
        FieldError::new(
            campo("porcentajeActual"),
            "Validation.OrdenTrabajoItem.PorcentajeInvalid",
        ),
    );
    v.max_length_opt(
        &campo("nota"),
        item.nota.as_deref(),
        limites::OBSERVACIONES,
        "Validation.OrdenTrabajoItem.NotaMaxLength",
    );
}

/// The accumulated ceiling of V-09, checked against the history the input does not carry.
///
/// `porcentaje_anterior` is not part of the input — it is written only by issuing or voiding a
/// certificate — so the use case supplies it and this runs after the shape check. The comparison is
/// on the scaled `i64`, never on `f64`: `0.1 + 0.2` must not decide whether a certification is
/// legal. The error lands on `porcentajeActual`, the field the user is editing, and not on the
/// read-only history.
pub fn validar_acumulado(
    index: usize,
    porcentaje_anterior: Decimal4,
    porcentaje_actual: Decimal4,
) -> Option<FieldError> {
    if porcentaje_anterior.raw() + porcentaje_actual.raw() > Decimal4::HUNDRED.raw() {
        Some(
            FieldError::new(
                format!("items[{index}].porcentajeActual"),
                "Validation.OrdenTrabajoItem.PorcentajeAcumuladoInvalid",
            )
            .with_param(
                "disponible",
                porcentaje_anterior_disponible(porcentaje_anterior),
            ),
        )
    } else {
        None
    }
}

fn porcentaje_anterior_disponible(porcentaje_anterior: Decimal4) -> String {
    Decimal4::HUNDRED
        .checked_sub(porcentaje_anterior)
        .unwrap_or(Decimal4::ZERO)
        .to_decimal_string()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use eo_domain::Money;

    use super::*;
    use crate::AppError;

    fn item() -> OrdenTrabajoItemInput {
        OrdenTrabajoItemInput {
            id: None,
            descripcion: "cableado".into(),
            unidad: "m".into(),
            cantidad: Decimal4::parse("4200").unwrap(),
            precio_unitario: Money::parse("1000").unwrap(),
            porcentaje_actual: Decimal4::parse("60").unwrap(),
            ejecutado: false,
            nota: None,
        }
    }

    fn input() -> OrdenTrabajoInput {
        OrdenTrabajoInput {
            trabajo_id: Uuid::from_u128(1),
            titulo: "Planilla 1".into(),
            fecha: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            observaciones: None,
            ajuste_uocra_porcentaje: Decimal4::parse("8").unwrap(),
            otros_descuentos: Money::ZERO,
            items: vec![item()],
        }
    }

    fn keys(error: AppError) -> Vec<String> {
        error
            .fields()
            .iter()
            .map(|f| f.message_key.clone())
            .collect()
    }

    fn campos(error: AppError) -> Vec<String> {
        error.fields().iter().map(|f| f.field.clone()).collect()
    }

    #[test]
    fn una_orden_minima_es_valida() {
        assert!(validate(&input()).is_ok());
    }

    #[test]
    fn una_orden_sin_items_no_certifica_nada() {
        let dto = OrdenTrabajoInput {
            items: vec![],
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.OrdenTrabajo.ItemsRequired"]
        );
    }

    #[test]
    fn la_cantidad_debe_ser_positiva() {
        let dto = OrdenTrabajoInput {
            items: vec![OrdenTrabajoItemInput {
                cantidad: Decimal4::ZERO,
                ..item()
            }],
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.OrdenTrabajoItem.CantidadRequired"]
        );
    }

    #[test]
    fn el_error_del_item_apunta_a_su_fila() {
        let dto = OrdenTrabajoInput {
            items: vec![
                item(),
                OrdenTrabajoItemInput {
                    descripcion: "  ".into(),
                    ..item()
                },
            ],
            ..input()
        };
        assert_eq!(
            campos(validate(&dto).unwrap_err()),
            ["items[1].descripcion"]
        );
    }

    #[test]
    fn el_porcentaje_actual_no_pasa_de_cien() {
        let dto = OrdenTrabajoInput {
            items: vec![OrdenTrabajoItemInput {
                porcentaje_actual: Decimal4::parse("100.0001").unwrap(),
                ..item()
            }],
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.OrdenTrabajoItem.PorcentajeInvalid"]
        );
    }

    #[test]
    fn el_ajuste_uocra_es_un_porcentaje() {
        let dto = OrdenTrabajoInput {
            ajuste_uocra_porcentaje: Decimal4::parse("101").unwrap(),
            ..input()
        };
        assert_eq!(
            keys(validate(&dto).unwrap_err()),
            ["Validation.OrdenTrabajo.AjusteInvalid"]
        );
    }

    /// The gap the legacy system left open: it accepted certifying 200 % of a line.
    #[test]
    fn el_acumulado_no_pasa_de_cien() {
        let error = validar_acumulado(
            0,
            Decimal4::parse("60").unwrap(),
            Decimal4::parse("40.0001").unwrap(),
        )
        .expect("debe rechazarse");
        assert_eq!(error.field, "items[0].porcentajeActual");
        assert_eq!(
            error.message_key,
            "Validation.OrdenTrabajoItem.PorcentajeAcumuladoInvalid"
        );
    }

    #[test]
    fn cerrar_exactamente_en_cien_es_valido() {
        assert!(validar_acumulado(
            0,
            Decimal4::parse("60").unwrap(),
            Decimal4::parse("40").unwrap()
        )
        .is_none());
    }
}
