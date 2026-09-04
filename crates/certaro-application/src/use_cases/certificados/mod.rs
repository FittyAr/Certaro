//! Use cases of `certificados`. See `docs/06-casos-de-uso-y-formulas.md` §5.5 and §5.6.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::NaiveDate;
use certaro_domain::Decimal4;
use tracing::info;
use uuid::Uuid;

use crate::dtos::certificados::{CertificadoDetalle, CertificadoFiltroDto, CertificadoListItem};
use crate::dtos::common::ListQuery;
use crate::error::AppError;
use crate::paging::PagedResult;
use crate::ports::repositories::{Transaction, UnitOfWork};
use crate::ports::{ClockPort, IdGeneratorPort, SettingsStore};
use crate::result::AppResult;
use crate::use_cases::shared::{
    checked_sort, finish_read, finish_write, normalise, parse_row_version,
};
use crate::validation;
use crate::validation::movimientos::ContextoFecha;

mod calculation;
mod issuance;

const ENTITY: &str = "Certificado";
const SORTABLE: [&str; 4] = ["numero", "fecha", "totalNeto", "createdAt"];

pub struct CertificadosService {
    pub(crate) uow: Arc<dyn UnitOfWork>,
    pub(crate) clock: Arc<dyn ClockPort>,
    pub(crate) ids: Arc<dyn IdGeneratorPort>,
    pub(crate) settings: Arc<dyn SettingsStore>,
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

            let previo = ordenes.count_certificados(orden.id).await?;
            ordenes.touch(orden.id, now).await?;
            info!(orden = %orden.id, restantes = previo.saturating_sub(1), "certificado anulado");
            Ok(())
        }
        .await;
        finish_write(tx, outcome).await
    }

    pub(crate) fn hoy(&self) -> NaiveDate {
        self.clock.now_utc().date_naive()
    }

    pub(crate) fn contexto_fecha(&self, hoy: NaiveDate) -> ContextoFecha {
        ContextoFecha::from_config(&self.settings.snapshot().validation, hoy)
    }
}

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
