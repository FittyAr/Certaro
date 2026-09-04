//! Use cases of `liquidaciones`. See `docs/06-casos-de-uso-y-formulas.md` §6.

pub mod calculation;
pub use calculation::construir_sugerencia;

use std::collections::HashSet;
use std::sync::Arc;

use chrono::{Datelike, NaiveDate, Weekday};
use certaro_domain::entities::{Audit, Empleado, Liquidacion, LiquidacionAdelanto, ReglasLiquidacion};
use certaro_domain::{Decimal4, Money, TipoJornada};
use tracing::info;
use uuid::Uuid;

use crate::dtos::common::ListQuery;
use crate::dtos::liquidaciones::{
    LiquidacionAdelantoSugerido, LiquidacionBatchInput, LiquidacionBatchResult,
    LiquidacionDesglose, LiquidacionDetalle, LiquidacionFiltroDto, LiquidacionInput,
    LiquidacionListItem, LiquidacionSugerencia, LiquidacionSugerenciaQuery, LiquidacionUpdateInput,
    OrigenLiquidacion,
};
use crate::error::AppError;
use crate::paging::PagedResult;
use crate::ports::repositories::{LiquidacionRepository, Transaction, UnitOfWork};
use crate::ports::{ClockPort, IdGeneratorPort};
use crate::result::AppResult;
use crate::use_cases::shared::{
    checked_sort, finish_read, finish_write, normalise, parse_row_version,
};
use crate::validation;

const ENTITY: &str = "Liquidacion";

const SORTABLE: [&str; 6] = [
    "empleadoNombre",
    "fechaInicio",
    "fechaFin",
    "diasTrabajados",
    "totalBruto",
    "totalNeto",
];

pub struct LiquidacionesService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    ids: Arc<dyn IdGeneratorPort>,
}

impl LiquidacionesService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        ids: Arc<dyn IdGeneratorPort>,
    ) -> Self {
        Self { uow, clock, ids }
    }

    pub async fn list(
        &self,
        query: ListQuery<LiquidacionFiltroDto>,
    ) -> AppResult<PagedResult<LiquidacionListItem>> {
        let sort_by = checked_sort(query.sort_by.as_deref(), &SORTABLE)?;
        let page = query.page_request();
        page.validate()?;
        let filtro = query.filtro.into();

        let tx = self.uow.begin().await?;
        let result = tx
            .liquidaciones()
            .search(&filtro, page, sort_by, query.sort_dir)
            .await;
        let page = finish_read(tx, result).await?;
        Ok(page.map(LiquidacionListItem::from))
    }

    pub async fn get(&self, id: Uuid) -> AppResult<LiquidacionDetalle> {
        let tx = self.uow.begin().await?;
        let loaded = load_detalle(tx.liquidaciones(), id).await;
        finish_read(tx, loaded).await
    }

    /// Pure: computes and persists nothing. The wizard calls it on entering step two and again on
    /// every recalculation.
    pub async fn suggest(
        &self,
        query: LiquidacionSugerenciaQuery,
    ) -> AppResult<Vec<LiquidacionSugerencia>> {
        if query.desde > query.hasta {
            return Err(AppError::Validation(vec![crate::error::FieldError::new(
                "hasta",
                "Validation.Liquidacion.FechaInicioInvalid",
            )]));
        }

        let tx = self.uow.begin().await?;
        let outcome = async {
            let feriados: HashSet<NaiveDate> = tx
                .feriados()
                .del_rango(query.desde, query.hasta)
                .await?
                .into_iter()
                .map(|f| f.fecha)
                .collect();

            let mut sugerencias = Vec::with_capacity(query.empleado_ids.len());
            for empleado_id in &query.empleado_ids {
                let empleado = tx
                    .empleados()
                    .find_by_id(*empleado_id)
                    .await?
                    .ok_or_else(|| AppError::not_found("Empleado", *empleado_id))?;

                let asistencias = tx
                    .asistencias()
                    .del_periodo(query.desde, query.hasta, &[*empleado_id])
                    .await?;
                let jornadas: Vec<(NaiveDate, TipoJornada)> = asistencias
                    .iter()
                    .map(|a| (a.fecha, a.tipo_jornada))
                    .collect();

                let candidatos = tx
                    .liquidaciones()
                    .adelantos_candidatos(*empleado_id, query.desde, query.hasta)
                    .await?;

                let dias_manuales = query.dias_manuales.get(empleado_id).copied();
                sugerencias.push(construir_sugerencia(
                    &empleado,
                    query.desde,
                    query.hasta,
                    dias_manuales,
                    &jornadas,
                    &feriados,
                    &candidatos,
                )?);
            }
            Ok(sugerencias)
        }
        .await;
        finish_read(tx, outcome).await
    }

    /// The whole batch in one transaction: partially settling a payroll leaves the person who was
    /// skipped unpaid and nobody looking for them.
    pub async fn create_batch(
        &self,
        input: LiquidacionBatchInput,
    ) -> AppResult<LiquidacionBatchResult> {
        validation::liquidaciones::validate_batch(&input)?;

        let now = self.clock.now_utc();
        let tx = self.uow.begin().await?;
        let outcome = async {
            let mut creadas = Vec::with_capacity(input.dtos.len());
            for dto in &input.dtos {
                let empleado = cargar_empleado(&*tx, dto.empleado_id).await?;

                if let Some(otra) = tx
                    .liquidaciones()
                    .periodo_solapado(dto.empleado_id, dto.fecha_inicio, dto.fecha_fin, None)
                    .await?
                {
                    return Err(periodo_solapado(&empleado, &otra));
                }

                let entity = Liquidacion {
                    id: self.ids.new_id(),
                    empleado_id: dto.empleado_id,
                    fecha_inicio: dto.fecha_inicio,
                    fecha_fin: dto.fecha_fin,
                    dias_trabajados: dto.dias_trabajados,
                    tarifa_aplicada: dto.tarifa_aplicada,
                    reglas: dto.reglas(),
                    total_bruto: dto.total_bruto,
                    total_adelantos: dto.total_adelantos,
                    observaciones: normalise(dto.observaciones.clone()),
                    pdf_generado_at: None,
                    adelantos: Vec::new(),
                    audit: Audit::new(now),
                };
                tx.liquidaciones().insert(&entity).await?;

                let mut suma = Money::ZERO;
                for adelanto in &dto.adelantos {
                    let fila = LiquidacionAdelanto {
                        id: self.ids.new_id(),
                        liquidacion_id: entity.id,
                        movimiento_id: adelanto.movimiento_id,
                        monto: adelanto.monto,
                        fecha: adelanto.fecha,
                        concepto: adelanto.concepto.trim().to_owned(),
                        audit: Audit::new(now),
                    };
                    // The unique index on `movimiento_id` is what actually enforces INV-05; this
                    // read only turns the collision into a message the user can act on.
                    tx.liquidaciones().insert_adelanto(&fila).await?;
                    suma = suma.checked_add(fila.monto)?;
                }

                // The frozen total and the sum of the frozen lines have to agree, or the PDF would
                // not add up to what was paid.
                if suma != dto.total_adelantos {
                    return Err(AppError::unexpected(anyhow::anyhow!(
                        "la suma de adelantos ({}) no coincide con el total ({})",
                        suma.to_decimal_string(),
                        dto.total_adelantos.to_decimal_string()
                    )));
                }

                creadas.push(entity.id);
            }
            Ok(creadas)
        }
        .await;
        let creadas = finish_write(tx, outcome).await?;

        info!(cantidad = creadas.len(), "liquidaciones creadas");
        Ok(LiquidacionBatchResult { creadas })
    }

    pub async fn create(&self, input: LiquidacionInput) -> AppResult<LiquidacionDetalle> {
        validation::liquidaciones::validate(&input)?;
        let batch = self
            .create_batch(LiquidacionBatchInput { dtos: vec![input] })
            .await?;
        let id = batch.creadas.first().copied().ok_or_else(|| {
            AppError::unexpected(anyhow::anyhow!("el lote no devolvió ninguna liquidación"))
        })?;
        self.get(id).await
    }

    /// Amounts stay editable until the PDF is handed over; after that only the notes move.
    pub async fn update(
        &self,
        id: Uuid,
        input: LiquidacionUpdateInput,
        row_version: &str,
    ) -> AppResult<LiquidacionDetalle> {
        validation::liquidaciones::validate_update(&input)?;
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.liquidaciones();
            let mut entity = repo
                .find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            let cambia_importes = entity.dias_trabajados != input.dias_trabajados
                || entity.tarifa_aplicada != input.tarifa_aplicada
                || entity.total_bruto != input.total_bruto
                || entity.total_adelantos != input.total_adelantos;
            if cambia_importes && !entity.admite_cambio_de_importes() {
                return Err(ya_entregada(&entity));
            }

            if cambia_importes {
                entity.dias_trabajados = input.dias_trabajados;
                entity.tarifa_aplicada = input.tarifa_aplicada;
                entity.total_bruto = input.total_bruto;
                entity.total_adelantos = input.total_adelantos;
            }
            entity.observaciones = normalise(input.observaciones.clone());
            entity.audit.touch(now);

            repo.update(&entity, esperado).await?;
            load_detalle(repo, id).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, "liquidación actualizada");
        Ok(detalle)
    }

    /// Voiding frees the advances it consumed, so they can be settled again.
    pub async fn delete(&self, id: Uuid, row_version: &str) -> AppResult<()> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.liquidaciones();
            repo.find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;
            repo.soft_delete(id, esperado, now).await
        }
        .await;
        finish_write(tx, outcome).await?;

        info!(%id, "liquidación anulada");
        Ok(())
    }

    /// Records that the document was handed over, which is what freezes the amounts.
    pub async fn marcar_pdf_generado(&self, id: Uuid) -> AppResult<()> {
        let now = self.clock.now_utc();
        let tx = self.uow.begin().await?;
        let outcome = tx.liquidaciones().marcar_pdf_generado(id, now).await;
        finish_write(tx, outcome).await
    }
}

/// §6.6 of doc 06, in one place so the branches can be tested without a database.
#[allow(clippy::too_many_arguments)]


pub(crate) async fn load_detalle(
    repo: &dyn LiquidacionRepository,
    id: Uuid,
) -> AppResult<LiquidacionDetalle> {
    let mut row = repo
        .find_detalle(id)
        .await?
        .ok_or_else(|| AppError::not_found(ENTITY, id))?;
    row.liquidacion.adelantos = repo.adelantos_de(id).await?;
    Ok(LiquidacionDetalle::build(&row))
}

async fn cargar_empleado(tx: &dyn Transaction, id: Uuid) -> AppResult<Empleado> {
    tx.empleados()
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::not_found("Empleado", id))
}

fn periodo_solapado(empleado: &Empleado, otra: &Liquidacion) -> AppError {
    AppError::Conflict {
        code: "PERIODO_SOLAPADO",
        message_key: "Validation.Liquidacion.PeriodoSolapado",
        params: std::collections::BTreeMap::from([
            ("empleado".to_owned(), empleado.nombre.clone()),
            ("desde".to_owned(), otra.fecha_inicio.to_string()),
            ("hasta".to_owned(), otra.fecha_fin.to_string()),
        ]),
    }
}

fn ya_entregada(liquidacion: &Liquidacion) -> AppError {
    AppError::Conflict {
        code: "LIQUIDACION_YA_ENTREGADA",
        message_key: "State.Liquidacion.YaEntregada",
        params: std::collections::BTreeMap::from([(
            "fecha".to_owned(),
            liquidacion
                .pdf_generado_at
                .map(|f| f.to_rfc3339())
                .unwrap_or_default(),
        )]),
    }
}

