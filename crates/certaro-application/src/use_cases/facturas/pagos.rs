use chrono::NaiveDate;
use certaro_domain::entities::{Audit, Factura, PagoFactura};
use certaro_domain::{recalcular_estado_factura, Money, StateMachine};
use tracing::info;
use uuid::Uuid;

use crate::dtos::facturas::{FacturaDetalle, PagoFacturaInput, PagoFacturaItem};
use crate::error::AppError;
use crate::paging::PageRequest;
use crate::ports::repositories::{FacturaRepository, MovimientoFiltro, SortDir};
use crate::result::AppResult;
use crate::use_cases::shared::{finish_read, finish_write, parse_row_version};
use crate::validation;

use super::factura_crud::load_detalle;
use super::{FacturasService, ENTITY, ENTITY_PAGO};

impl FacturasService {
    pub async fn pagos_de(&self, factura_id: Uuid) -> AppResult<Vec<PagoFacturaItem>> {
        let tx = self.uow.begin().await?;
        let result = tx.facturas().pagos_de(factura_id).await;
        let pagos = finish_read(tx, result).await?;
        Ok(pagos.iter().map(PagoFacturaItem::from).collect())
    }

    pub async fn crear_pago(&self, input: PagoFacturaInput) -> AppResult<FacturaDetalle> {
        let hoy = self.hoy();
        validation::facturas::validate_pago(&input, &self.contexto_fecha(hoy))?;

        let now = self.clock.now_utc();
        let dias = self.dias_vencimiento();
        let tolerancia = self
            .settings
            .snapshot()
            .business
            .tolerancia_sobrepago_factura;

        let pago = PagoFactura {
            id: self.ids.new_id(),
            factura_id: input.factura_id,
            fecha: input.fecha,
            monto: input.monto,
            medio_pago: input.medio_pago.trim().to_owned(),
            audit: Audit::new(now),
        };

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.facturas();
            let factura = cargar_para_pago(repo, input.factura_id).await?;
            ensure_no_excede_saldo(&factura, pago.monto, None, tolerancia)?;

            repo.insert_pago(&pago).await?;
            recalcular(repo, input.factura_id, hoy, dias).await?;
            load_detalle(&*tx, input.factura_id, hoy, dias).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(
            factura = %detalle.id,
            monto = %pago.monto.to_decimal_string(),
            saldo = %detalle.saldo.to_decimal_string(),
            "pago registrado"
        );
        Ok(detalle)
    }

    pub async fn actualizar_pago(
        &self,
        id: Uuid,
        input: PagoFacturaInput,
        row_version: &str,
    ) -> AppResult<FacturaDetalle> {
        let hoy = self.hoy();
        validation::facturas::validate_pago(&input, &self.contexto_fecha(hoy))?;
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();
        let dias = self.dias_vencimiento();
        let tolerancia = self
            .settings
            .snapshot()
            .business
            .tolerancia_sobrepago_factura;

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.facturas();
            let mut pago = repo
                .find_pago(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY_PAGO, id))?;

            let factura = cargar_para_pago(repo, pago.factura_id).await?;
            ensure_no_excede_saldo(&factura, input.monto, Some(pago.id), tolerancia)?;

            pago.fecha = input.fecha;
            pago.monto = input.monto;
            pago.medio_pago = input.medio_pago.trim().to_owned();
            pago.audit.touch(now);
            repo.update_pago(&pago, esperado).await?;

            recalcular(repo, pago.factura_id, hoy, dias).await?;
            load_detalle(&*tx, pago.factura_id, hoy, dias).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(factura = %detalle.id, pago = %id, "pago actualizado");
        Ok(detalle)
    }

    pub async fn borrar_pago(&self, id: Uuid, row_version: &str) -> AppResult<FacturaDetalle> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();
        let hoy = self.hoy();
        let dias = self.dias_vencimiento();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.facturas();
            let pago = repo
                .find_pago(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY_PAGO, id))?;

            repo.soft_delete_pago(id, esperado, now).await?;
            recalcular(repo, pago.factura_id, hoy, dias).await?;

            let movs_repo = tx.movimientos();
            if let Ok(res) = movs_repo
                .search(
                    &MovimientoFiltro {
                        factura_id: Some(pago.factura_id),
                        monto_min: Some(pago.monto),
                        monto_max: Some(pago.monto),
                        ..Default::default()
                    },
                    PageRequest::new(1, 20),
                    None,
                    SortDir::Desc,
                )
                .await
            {
                let pago_id_str = pago.id.to_string();
                let ref_corta = &pago_id_str[..8.min(pago_id_str.len())];
                let candidato = res
                    .items
                    .iter()
                    .find(|m| {
                        m.movimiento.concepto.contains(ref_corta)
                            || m.movimiento.concepto.contains(&pago.medio_pago)
                    })
                    .or_else(|| res.items.first());

                if let Some(mov) = candidato {
                    let _ = movs_repo
                        .soft_delete(
                            mov.movimiento.id,
                            mov.movimiento.audit.row_version,
                            now,
                        )
                        .await;
                }
            }

            load_detalle(&*tx, pago.factura_id, hoy, dias).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(factura = %detalle.id, pago = %id, "pago eliminado");
        Ok(detalle)
    }
}

async fn cargar_para_pago(repo: &dyn FacturaRepository, id: Uuid) -> AppResult<Factura> {
    let factura = repo
        .find_con_pagos(id)
        .await?
        .ok_or_else(|| AppError::not_found(ENTITY, id))?;

    if !factura.estado.admite_pagos() {
        return Err(AppError::Conflict {
            code: "FACTURA_NO_ADMITE_PAGOS",
            message_key: "State.Factura.NoAdmitePagos",
            params: [("estado".to_owned(), factura.estado.as_key().to_owned())].into(),
        });
    }
    Ok(factura)
}

fn ensure_no_excede_saldo(
    factura: &Factura,
    monto: Money,
    excluir: Option<Uuid>,
    tolerancia: Money,
) -> AppResult<()> {
    let ya_pagado = Money::try_sum(
        factura
            .pagos
            .iter()
            .filter(|p| !p.audit.is_deleted && Some(p.id) != excluir)
            .map(|p| p.monto),
    )?;
    let saldo = factura.total.checked_sub(ya_pagado)?;
    let limite = saldo.checked_add(tolerancia)?;

    if monto > limite {
        return Err(AppError::Conflict {
            code: "PAGO_EXCEDE_SALDO",
            message_key: "Validation.PagoFactura.ExcedeSaldo",
            params: [("saldo".to_owned(), saldo.to_decimal_string())].into(),
        });
    }
    Ok(())
}

async fn recalcular(
    repo: &dyn FacturaRepository,
    factura_id: Uuid,
    hoy: NaiveDate,
    dias: u32,
) -> AppResult<()> {
    let mut factura = repo
        .find_con_pagos(factura_id)
        .await?
        .ok_or_else(|| AppError::not_found(ENTITY, factura_id))?;
    let anterior = factura.estado;
    recalcular_estado_factura(&mut factura, hoy, dias)?;
    if factura.estado != anterior {
        repo.update_estado(factura_id, factura.estado).await?;
    }
    Ok(())
}
