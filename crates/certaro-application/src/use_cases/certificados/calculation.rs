use std::collections::HashMap;
use uuid::Uuid;

use certaro_domain::entities::OrdenTrabajo;
use certaro_domain::Decimal4;

use crate::dtos::certificados::CertificadoInput;
use crate::error::{AppError, FieldError};
use crate::ports::repositories::CertificadoRepository;
use crate::result::AppResult;

/// The order's items that this certificate touches, paired with the progress asked for, in the
/// sheet's own order. Zero and missing lines are dropped: they certify nothing.
pub fn avances_pedidos<'a>(
    input: &CertificadoInput,
    orden: &'a OrdenTrabajo,
) -> AppResult<Vec<(&'a certaro_domain::OrdenTrabajoItem, Decimal4)>> {
    let mut avances = Vec::new();
    for (i, pedido) in input.items.iter().enumerate() {
        let Some(item) = orden
            .items
            .iter()
            .find(|it| it.id == pedido.orden_trabajo_item_id)
        else {
            return Err(AppError::Validation(vec![FieldError::new(
                format!("items[{i}].ordenTrabajoItemId"),
                "Validation.Common.EntityNotFound",
            )]));
        };
        if pedido.porcentaje_actual.is_positive() {
            avances.push((item, pedido.porcentaje_actual));
        }
    }
    avances.sort_by_key(|(item, _)| item.orden);
    Ok(avances)
}

/// Doc 07 §5.3: the sum against the certified history, on scaled `i64`.
pub fn verificar_acumulados(
    avances: &[(&certaro_domain::OrdenTrabajoItem, Decimal4)],
    acumulados: &HashMap<Uuid, Decimal4>,
) -> AppResult<()> {
    let errores: Vec<_> = avances
        .iter()
        .enumerate()
        .filter_map(|(i, (item, porcentaje))| {
            let anterior = acumulados.get(&item.id).copied().unwrap_or(Decimal4::ZERO);
            (anterior.raw() + porcentaje.raw() > Decimal4::HUNDRED.raw()).then(|| {
                FieldError::new(
                    format!("items[{i}].porcentajeActual"),
                    "Validation.Certificado.AcumuladoExcedido",
                )
                .with_param("item", item.descripcion.clone())
                .with_param("acumulado", anterior.to_decimal_string())
            })
        })
        .collect();

    if errores.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(errores))
    }
}

pub async fn acumulados_de(
    repo: &dyn CertificadoRepository,
    orden_trabajo_id: Uuid,
) -> AppResult<HashMap<Uuid, Decimal4>> {
    Ok(repo
        .acumulado_por_item(orden_trabajo_id)
        .await?
        .into_iter()
        .collect())
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, NaiveDate, Utc};
    use certaro_domain::entities::{Audit, OrdenTrabajoItem};
    use certaro_domain::Money;

    use super::*;
    use crate::dtos::certificados::CertificadoInputItem;

    fn ahora() -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(0, 0).unwrap()
    }

    fn item(n: u128, orden: i32, actual: &str) -> OrdenTrabajoItem {
        OrdenTrabajoItem {
            id: Uuid::from_u128(n),
            orden_trabajo_id: Uuid::from_u128(100),
            descripcion: format!("item {n}"),
            unidad: "u".into(),
            cantidad: Decimal4::parse("10").unwrap(),
            precio_unitario: Money::parse("100").unwrap(),
            porcentaje_anterior: Decimal4::ZERO,
            porcentaje_actual: Decimal4::parse(actual).unwrap(),
            ejecutado: false,
            nota: None,
            orden,
            audit: Audit::new(ahora()),
        }
    }

    fn orden(items: Vec<OrdenTrabajoItem>) -> OrdenTrabajo {
        OrdenTrabajo {
            id: Uuid::from_u128(100),
            trabajo_id: Uuid::from_u128(101),
            titulo: "Planilla".into(),
            numero_certificado: None,
            fecha: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            observaciones: None,
            ajuste_uocra_porcentaje: Decimal4::ZERO,
            otros_descuentos: Money::ZERO,
            items,
            audit: Audit::new(ahora()),
        }
    }

    fn input(items: &[(u128, &str)]) -> CertificadoInput {
        CertificadoInput {
            orden_trabajo_id: Uuid::from_u128(100),
            fecha: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            observaciones: None,
            items: items
                .iter()
                .map(|(n, pct)| CertificadoInputItem {
                    orden_trabajo_item_id: Uuid::from_u128(*n),
                    porcentaje_actual: Decimal4::parse(pct).unwrap(),
                })
                .collect(),
        }
    }

    #[test]
    fn un_avance_en_cero_no_se_certifica() {
        let o = orden(vec![item(1, 0, "0"), item(2, 1, "50")]);
        let avances = avances_pedidos(&input(&[(1, "0"), (2, "50")]), &o).unwrap();
        assert_eq!(avances.len(), 1);
        assert_eq!(avances[0].0.id, Uuid::from_u128(2));
    }

    #[test]
    fn los_avances_salen_en_el_orden_de_la_planilla() {
        let o = orden(vec![item(1, 5, "10"), item(2, 1, "20")]);
        let avances = avances_pedidos(&input(&[(1, "10"), (2, "20")]), &o).unwrap();
        assert_eq!(
            avances.iter().map(|(i, _)| i.orden).collect::<Vec<_>>(),
            [1, 5]
        );
    }

    #[test]
    fn un_item_ajeno_a_la_orden_es_un_formulario_viejo() {
        let o = orden(vec![item(1, 0, "10")]);
        let error = avances_pedidos(&input(&[(9, "10")]), &o).unwrap_err();
        assert_eq!(error.fields()[0].field, "items[0].ordenTrabajoItemId");
    }

    #[test]
    fn el_acumulado_historico_acota_el_avance() {
        let it = item(1, 0, "45");
        let avances = vec![(&it, Decimal4::parse("45").unwrap())];
        let mut acumulados = HashMap::new();
        acumulados.insert(it.id, Decimal4::parse("60").unwrap());

        let error = verificar_acumulados(&avances, &acumulados).unwrap_err();
        assert_eq!(
            error.fields()[0].message_key,
            "Validation.Certificado.AcumuladoExcedido"
        );
        assert_eq!(error.fields()[0].params["acumulado"], "60.0000");
    }

    #[test]
    fn cerrar_en_cien_exacto_se_permite() {
        let it = item(1, 0, "40");
        let avances = vec![(&it, Decimal4::parse("40").unwrap())];
        let mut acumulados = HashMap::new();
        acumulados.insert(it.id, Decimal4::parse("60").unwrap());
        assert!(verificar_acumulados(&avances, &acumulados).is_ok());
    }

    #[test]
    fn sin_historia_el_avance_llega_hasta_cien() {
        let it = item(1, 0, "100");
        let avances = vec![(&it, Decimal4::HUNDRED)];
        assert!(verificar_acumulados(&avances, &HashMap::new()).is_ok());
    }
}
