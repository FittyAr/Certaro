use certaro_domain::entities::{Audit, Certificado, CertificadoItem};
use certaro_domain::{Decimal4, Money};
use tracing::info;
use uuid::Uuid;

use crate::dtos::certificados::{
    CertificadoBorrador, CertificadoBorradorItem, CertificadoDetalle, CertificadoInput,
};
use crate::error::AppError;
use crate::result::AppResult;
use crate::use_cases::shared::{finish_read, finish_write, normalise};
use crate::validation;

use super::calculation::{acumulados_de, avances_pedidos, verificar_acumulados};
use super::{load_detalle, CertificadosService};

impl CertificadosService {
    /// Prefills the issuing form: what each line has certified so far and what is left.
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
            let anteriores = tx.certificados().de_orden(orden_trabajo_id).await?;
            let ya_descontado = Money::try_sum(anteriores.iter().map(|c| c.otros_descuentos))?;
            let otros_descuentos_restante = row
                .orden
                .otros_descuentos
                .checked_sub(ya_descontado)
                .unwrap_or(Money::ZERO);

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
                otros_descuentos: otros_descuentos_restante,
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

            let anteriores = certificados.de_orden(orden.id).await?;
            let ya_descontado = Money::try_sum(anteriores.iter().map(|c| c.otros_descuentos))?;
            let descuento_disponible = orden
                .otros_descuentos
                .checked_sub(ya_descontado)
                .unwrap_or(Money::ZERO);

            let bases = orden
                .items
                .iter()
                .map(|i| i.base())
                .collect::<Result<Vec<_>, _>>()?;
            let total_orden = Money::try_sum(bases)?;
            let descuento_a_aplicar = if total_orden.raw() > 0 && orden.otros_descuentos.raw() > 0 {
                let pct = Decimal4::from_raw(total_certificado.raw())
                    .checked_div(Decimal4::from_raw(total_orden.raw()))
                    .and_then(|q| q.checked_mul(Decimal4::HUNDRED))
                    .unwrap_or(Decimal4::ZERO);
                let prop = orden.otros_descuentos.percent(pct)?;
                prop.min(descuento_disponible)
            } else {
                descuento_disponible
            };

            let neto_previo = total_certificado.checked_add(ajuste_uocra)?;
            let otros_descuentos = descuento_a_aplicar.min(neto_previo);
            let total_neto = neto_previo.checked_sub(otros_descuentos)?;

            let certificado = Certificado {
                id,
                orden_trabajo_id: orden.id,
                numero,
                fecha: input.fecha,
                observaciones: normalise(input.observaciones.clone()),
                total_certificado,
                ajuste_uocra,
                otros_descuentos,
                total_neto,
                items: Vec::new(),
                audit: Audit::new(now),
            };
            certificados.insert(&certificado).await?;
            for linea in &lineas {
                certificados.insert_item(linea).await?;
            }

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
}
