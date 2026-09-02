//! Use cases of `certificados`. See `docs/06-casos-de-uso-y-formulas.md` §5.5 and §5.6.
//!
//! An issued certificate is a frozen copy: quantity, unit price and percentages are written into
//! `certificado_items` rather than read back from the order. The legacy system kept none of this —
//! it overwrote the percentages on the item and the printed PDF was the only record — which is the
//! whole reason the entity exists (RC-10).

use std::collections::HashMap;
use std::sync::Arc;

use chrono::NaiveDate;
use certaro_domain::entities::{Audit, Certificado, CertificadoItem, OrdenTrabajo};
use certaro_domain::{Decimal4, Money};
use tracing::info;
use uuid::Uuid;

use crate::dtos::certificados::{
    CertificadoBorrador, CertificadoBorradorItem, CertificadoDetalle, CertificadoFiltroDto,
    CertificadoInput, CertificadoListItem,
};
use crate::dtos::common::ListQuery;
use crate::error::{AppError, FieldError};
use crate::paging::PagedResult;
use crate::ports::repositories::{CertificadoRepository, Transaction, UnitOfWork};
use crate::ports::{ClockPort, IdGeneratorPort, SettingsStore};
use crate::result::AppResult;
use crate::use_cases::shared::{
    checked_sort, finish_read, finish_write, normalise, parse_row_version,
};
use crate::validation;
use crate::validation::movimientos::ContextoFecha;

const ENTITY: &str = "Certificado";

const SORTABLE: [&str; 4] = ["numero", "fecha", "totalNeto", "createdAt"];

pub struct CertificadosService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    ids: Arc<dyn IdGeneratorPort>,
    settings: Arc<dyn SettingsStore>,
}

impl CertificadosService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        ids: Arc<dyn IdGeneratorPort>,
        settings: Arc<dyn SettingsStore>,
    ) -> Self {
        Self {
            uow,
            clock,
            ids,
            settings,
        }
    }

    pub async fn list(
        &self,
        query: ListQuery<CertificadoFiltroDto>,
    ) -> AppResult<PagedResult<CertificadoListItem>> {
        let sort_by = checked_sort(query.sort_by.as_deref(), &SORTABLE)?;
        let page = query.page_request();
        page.validate()?;
        let filtro = query.filtro.into();

        let tx = self.uow.begin().await?;
        let result = tx
            .certificados()
            .search(&filtro, page, sort_by, query.sort_dir)
            .await;
        let page = finish_read(tx, result).await?;
        Ok(page.map(CertificadoListItem::from))
    }

    pub async fn get(&self, id: Uuid) -> AppResult<CertificadoDetalle> {
        let tx = self.uow.begin().await?;
        let loaded = load_detalle(&*tx, id).await;
        finish_read(tx, loaded).await
    }

    /// Prefills the issuing form: what each line has certified so far and what is left.
    ///
    /// The accumulated figure comes from the certificates, not from `porcentaje_anterior`, so the
    /// form and the check that guards the write read the same source.
    pub async fn preparar(&self, orden_trabajo_id: Uuid) -> AppResult<CertificadoBorrador> {
        let tx = self.uow.begin().await?;
        let loaded = async {
            let row = tx
                .ordenes_trabajo()
                .find_detalle(orden_trabajo_id)
                .await?
                .ok_or_else(|| AppError::not_found("OrdenTrabajo", orden_trabajo_id))?;
            let acumulados = acumulados_de(tx.certificados(), orden_trabajo_id).await?;
            let numero = tx.certificados().ultimo_numero(orden_trabajo_id).await? + 1;

            let items = row
                .orden
                .items
                .iter()
                .map(|item| {
                    let anterior = acumulados.get(&item.id).copied().unwrap_or(Decimal4::ZERO);
                    let base = item.base()?;
                    Ok(CertificadoBorradorItem {
                        orden_trabajo_item_id: item.id,
                        descripcion: item.descripcion.clone(),
                        unidad: item.unidad.clone(),
                        cantidad: item.cantidad,
                        precio_unitario: item.precio_unitario,
                        porcentaje_acumulado_anterior: anterior,
                        porcentaje_disponible: Decimal4::HUNDRED.checked_sub(anterior)?,
                        porcentaje_actual: item.porcentaje_actual,
                        base,
                        subtotal_acumulado_anterior: base.percent(anterior)?,
                    })
                })
                .collect::<AppResult<Vec<_>>>()?;

            Ok(CertificadoBorrador {
                orden_trabajo_id,
                orden_titulo: row.orden.titulo.clone(),
                numero_sugerido: numero,
                trabajo_descripcion: row.trabajo_descripcion.clone(),
                proyecto_nombre: row.proyecto_nombre.clone(),
                cliente_nombre: row.cliente_nombre.clone(),
                ajuste_uocra_porcentaje: row.orden.ajuste_uocra_porcentaje,
                otros_descuentos: row.orden.otros_descuentos,
                items,
            })
        }
        .await;
        finish_read(tx, loaded).await
    }

    /// Issues a certificate. `docs/06-casos-de-uso-y-formulas.md` §5.5, step by step.
    pub async fn create(&self, input: CertificadoInput) -> AppResult<CertificadoDetalle> {
        let hoy = self.hoy();
        validation::certificados::validate(&input, &self.contexto_fecha(hoy))?;

        let now = self.clock.now_utc();
        let id = self.ids.new_id();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let ordenes = tx.ordenes_trabajo();
            let certificados = tx.certificados();

            let orden = ordenes
                .find_con_items(input.orden_trabajo_id)
                .await?
                .ok_or_else(|| AppError::not_found("OrdenTrabajo", input.orden_trabajo_id))?;

            let avances = avances_pedidos(&input, &orden)?;
            if avances.is_empty() {
                return Err(AppError::Conflict {
                    code: "CERTIFICADO_VACIO",
                    message_key: "Validation.Certificado.SinAvance",
                    params: Default::default(),
                });
            }

            let acumulados = acumulados_de(certificados, orden.id).await?;
            verificar_acumulados(&avances, &acumulados)?;

            // Never `count + 1`: a voided certificate keeps its number spent (INV-15).
            let numero = certificados.ultimo_numero(orden.id).await? + 1;

            let mut lineas = Vec::with_capacity(avances.len());
            for (item, porcentaje_actual) in &avances {
                let anterior = acumulados.get(&item.id).copied().unwrap_or(Decimal4::ZERO);
                let acumulado = anterior.checked_add(*porcentaje_actual)?;
                let base = item.base()?;
                lineas.push(CertificadoItem {
                    id: self.ids.new_id(),
                    certificado_id: id,
                    orden_trabajo_item_id: item.id,
                    cantidad: item.cantidad,
                    precio_unitario: item.precio_unitario,
                    porcentaje_anterior: anterior,
                    porcentaje_actual: *porcentaje_actual,
                    subtotal_actual: base.percent(*porcentaje_actual)?,
                    subtotal_acumulado: base.percent(acumulado)?,
                    audit: Audit::new(now),
                });
            }

            let total_certificado = Money::try_sum(lineas.iter().map(|l| l.subtotal_actual))?;
            let ajuste_uocra = total_certificado.percent(orden.ajuste_uocra_porcentaje)?;
            let total_neto = total_certificado
                .checked_sub(ajuste_uocra)?
                .checked_sub(orden.otros_descuentos)?;

            let certificado = Certificado {
                id,
                orden_trabajo_id: orden.id,
                numero,
                fecha: input.fecha,
                observaciones: normalise(input.observaciones.clone()),
                total_certificado,
                ajuste_uocra,
                otros_descuentos: orden.otros_descuentos,
                total_neto,
                items: Vec::new(),
                audit: Audit::new(now),
            };
            certificados.insert(&certificado).await?;
            for linea in &lineas {
                certificados.insert_item(linea).await?;
            }

            // Step 8: the progress moves into the history and the current column goes back to zero.
            for linea in &lineas {
                let anterior = linea
                    .porcentaje_anterior
                    .checked_add(linea.porcentaje_actual)?;
                ordenes
                    .update_avance_item(
                        linea.orden_trabajo_item_id,
                        anterior,
                        Decimal4::ZERO,
                        anterior.raw() >= Decimal4::HUNDRED.raw(),
                        now,
                    )
                    .await?;
            }

            // The order's own fields do not change, but the aggregate did: bumping its version
            // makes a form opened before the issue fail its next save instead of overwriting.
            ordenes.touch(orden.id, now).await?;

            load_detalle(&*tx, id).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(
            id = %detalle.id,
            numero = detalle.numero,
            total_neto = %detalle.total_neto,
            "certificado emitido"
        );
        Ok(detalle)
    }

    /// The notes are the only editable field of an issued certificate (doc 08 §5.1).
    pub async fn update_observaciones(
        &self,
        id: Uuid,
        observaciones: Option<String>,
        row_version: &str,
    ) -> AppResult<CertificadoDetalle> {
        validation::certificados::validate_observaciones(observaciones.as_deref())?;
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();
        let texto = normalise(observaciones);

        let tx = self.uow.begin().await?;
        let outcome = async {
            tx.certificados()
                .find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;
            tx.certificados()
                .update_observaciones(id, texto.as_deref(), esperado, now)
                .await?;
            load_detalle(&*tx, id).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(%id, "observaciones de certificado actualizadas");
        Ok(detalle)
    }

    /// Voids a certificate, reverting step 8. Only the last one of its order (doc 06 §5.6).
    pub async fn delete(&self, id: Uuid, row_version: &str) -> AppResult<()> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let certificados = tx.certificados();
            let ordenes = tx.ordenes_trabajo();

            let certificado = certificados
                .find_con_items(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            // Voiding an intermediate one would leave the later certificates resting on a history
            // that no longer explains them.
            let ultimo = certificados
                .ultimo_numero(certificado.orden_trabajo_id)
                .await?;
            if certificado.numero != ultimo {
                return Err(AppError::Conflict {
                    code: "CERTIFICADO_NO_ULTIMO",
                    message_key: "Validation.Certificado.NoEsUltimo",
                    params: [("ultimo".to_owned(), ultimo.to_string())].into(),
                });
            }

            let orden = ordenes
                .find_con_items(certificado.orden_trabajo_id)
                .await?
                .ok_or_else(|| AppError::not_found("OrdenTrabajo", certificado.orden_trabajo_id))?;

            for linea in &certificado.items {
                let Some(item) = orden
                    .items
                    .iter()
                    .find(|i| i.id == linea.orden_trabajo_item_id)
                else {
                    continue;
                };
                let anterior = item
                    .porcentaje_anterior
                    .checked_sub(linea.porcentaje_actual)?;
                ordenes
                    .update_avance_item(
                        item.id,
                        anterior,
                        linea.porcentaje_actual,
                        anterior.raw() >= Decimal4::HUNDRED.raw(),
                        now,
                    )
                    .await?;
            }

            certificados.soft_delete(id, esperado, now).await?;

            // The number stays spent, so the order points at the previous one, if any.
            let previo = ordenes.count_certificados(orden.id).await?;
            ordenes.touch(orden.id, now).await?;
            info!(orden = %orden.id, restantes = previo.saturating_sub(1), "certificado anulado");
            Ok(())
        }
        .await;
        finish_write(tx, outcome).await
    }

    fn hoy(&self) -> NaiveDate {
        self.clock.now_utc().date_naive()
    }

    fn contexto_fecha(&self, hoy: NaiveDate) -> ContextoFecha {
        ContextoFecha::from_config(&self.settings.snapshot().validation, hoy)
    }
}

/// The order's items that this certificate touches, paired with the progress asked for, in the
/// sheet's own order. Zero and missing lines are dropped: they certify nothing.
fn avances_pedidos<'a>(
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
            // An id that is not in the order means a stale form, not a line to invent.
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
fn verificar_acumulados(
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

async fn acumulados_de(
    repo: &dyn CertificadoRepository,
    orden_trabajo_id: Uuid,
) -> AppResult<HashMap<Uuid, Decimal4>> {
    Ok(repo
        .acumulado_por_item(orden_trabajo_id)
        .await?
        .into_iter()
        .collect())
}

/// The descriptions the detail shows come from the order, since a frozen typo should still read
/// corrected; the amounts next to them never do.
pub(crate) async fn load_detalle(tx: &dyn Transaction, id: Uuid) -> AppResult<CertificadoDetalle> {
    let row = tx
        .certificados()
        .find_detalle(id)
        .await?
        .ok_or_else(|| AppError::not_found(ENTITY, id))?;

    let etiquetas = match tx
        .ordenes_trabajo()
        .find_con_items(row.orden_trabajo_id)
        .await?
    {
        Some(orden) => orden
            .items
            .into_iter()
            .map(|i| (i.id, (i.descripcion, i.unidad)))
            .collect(),
        None => HashMap::new(),
    };

    CertificadoDetalle::build(&row, &etiquetas)
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, NaiveDate, Utc};
    use certaro_domain::entities::OrdenTrabajoItem;

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
