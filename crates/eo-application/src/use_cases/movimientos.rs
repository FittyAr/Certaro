//! Use cases of `movimientos`. See `docs/06-casos-de-uso-y-formulas.md` §3.
//!
//! This is the only listing paged and filtered on the server: it is the table that grows without
//! bound, and shipping it whole to the frontend stops working after the first year of use.

use std::sync::Arc;

use eo_domain::entities::{Audit, Movimiento};
use tracing::info;
use uuid::Uuid;

use crate::dtos::common::ListQuery;
use crate::dtos::movimientos::{
    MovimientoDetalle, MovimientoFiltroDto, MovimientoInput, MovimientoListResult,
    MovimientoResumenDto,
};
use crate::error::AppError;
use crate::ports::repositories::{MovimientoRepository, ReferenciaTabla, UnitOfWork};
use crate::ports::{ClockPort, IdGeneratorPort, SettingsStore};
use crate::result::AppResult;
use crate::use_cases::shared::{checked_sort, finish_read, finish_write, parse_row_version};
use crate::validation;
use crate::validation::movimientos::ContextoFecha;

const ENTITY: &str = "Movimiento";

/// Closed list from `docs/11-contratos-tauri.md` §5.1.
const SORTABLE: [&str; 6] = [
    "fecha",
    "concepto",
    "monto",
    "total",
    "tipoMovimientoNombre",
    "categoriaNombre",
];

pub struct MovimientosService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    ids: Arc<dyn IdGeneratorPort>,
    settings: Arc<dyn SettingsStore>,
}

impl MovimientosService {
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

    /// The page and the summary come from the same transaction, so the totals under the table
    /// always describe the rows above it.
    pub async fn list(
        &self,
        query: ListQuery<MovimientoFiltroDto>,
    ) -> AppResult<MovimientoListResult> {
        let sort_by = checked_sort(query.sort_by.as_deref(), &SORTABLE)?;
        let page = query.page_request();
        page.validate()?;
        let filtro = query.filtro.into();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.movimientos();
            let rows = repo.search(&filtro, page, sort_by, query.sort_dir).await?;
            let resumen = repo.resumen(&filtro).await?;
            MovimientoListResult::build(rows, resumen)
        }
        .await;
        finish_read(tx, outcome).await
    }

    pub async fn get(&self, id: Uuid) -> AppResult<MovimientoDetalle> {
        let tx = self.uow.begin().await?;
        let outcome = async {
            tx.movimientos()
                .find_detalle(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))
                .and_then(MovimientoDetalle::try_from)
        }
        .await;
        finish_read(tx, outcome).await
    }

    pub async fn resumen(&self, filtro: MovimientoFiltroDto) -> AppResult<MovimientoResumenDto> {
        let filtro = filtro.into();
        let tx = self.uow.begin().await?;
        let result = tx.movimientos().resumen(&filtro).await;
        Ok(finish_read(tx, result).await?.into())
    }

    pub async fn create(&self, input: MovimientoInput) -> AppResult<MovimientoDetalle> {
        self.validar(&input)?;

        let now = self.clock.now_utc();
        let id = self.ids.new_id();
        let entity = self.build(id, &input, Audit::new(now));

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.movimientos();
            ensure_referencias(repo, &input).await?;
            repo.insert(&entity).await?;
            repo.find_detalle(entity.id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, entity.id))
                .and_then(MovimientoDetalle::try_from)
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(
            id = %detalle.item.id,
            concepto = %detalle.item.concepto,
            monto = %detalle.item.monto.to_decimal_string(),
            "movimiento creado"
        );
        Ok(detalle)
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: MovimientoInput,
        row_version: &str,
    ) -> AppResult<MovimientoDetalle> {
        self.validar(&input)?;
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.movimientos();
            let actual = repo
                .find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            ensure_editable(repo, id).await?;
            ensure_referencias(repo, &input).await?;

            let mut audit = actual.audit;
            audit.touch(now);
            let entity = self.build(id, &input, audit);

            repo.update(&entity, esperado).await?;
            repo.find_detalle(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))
                .and_then(MovimientoDetalle::try_from)
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(%id, "movimiento actualizado");
        Ok(detalle)
    }

    pub async fn delete(&self, id: Uuid, row_version: &str) -> AppResult<()> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.movimientos();
            repo.find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;
            ensure_editable(repo, id).await?;
            repo.soft_delete(id, esperado, now).await
        }
        .await;
        finish_write(tx, outcome).await?;

        info!(%id, "movimiento eliminado");
        Ok(())
    }

    fn validar(&self, input: &MovimientoInput) -> AppResult<()> {
        let config = self.settings.snapshot();
        let contexto = ContextoFecha::from_config(&config.validation, self.clock.now_utc().date_naive());
        validation::movimientos::validate(input, &contexto)
    }

    fn build(&self, id: Uuid, input: &MovimientoInput, audit: Audit) -> Movimiento {
        Movimiento {
            id,
            fecha: input.fecha,
            concepto: input.concepto.trim().to_owned(),
            monto: input.monto,
            cantidad: input.cantidad,
            tipo_movimiento_id: input.tipo_movimiento_id,
            moneda: input.moneda,
            // A rate only means something in foreign currency; the validator already refused a
            // peso movement carrying one, and this keeps a zero from being stored as a rate.
            cotizacion_aplicada: input
                .cotizacion_aplicada
                .filter(|_| input.moneda.requiere_cotizacion()),
            tipo_concepto_pago_id: input.tipo_concepto_pago_id,
            categoria_id: input.categoria_id,
            cliente_id: input.cliente_id,
            trabajo_id: input.trabajo_id,
            empleado_id: input.empleado_id,
            factura_id: input.factura_id,
            audit,
        }
    }
}

/// An advance already consumed by a payroll run is frozen: editing its amount would change a
/// settlement that was signed off, and deleting it would leave the discount without its source.
async fn ensure_editable(repo: &dyn MovimientoRepository, id: Uuid) -> AppResult<()> {
    if repo.esta_en_liquidacion(id).await? {
        return Err(AppError::DependencyInUse {
            code: "MOVIMIENTO_ADELANTO_LIQUIDADO",
            message_key: "Validation.Movimiento.AdelantoLiquidado",
            params: Default::default(),
        });
    }
    Ok(())
}

/// Checks every foreign key before writing. The constraints would catch these too, but a
/// `FOREIGN KEY constraint failed` names no field, and the form needs to know which one to mark.
async fn ensure_referencias(
    repo: &dyn MovimientoRepository,
    input: &MovimientoInput,
) -> AppResult<()> {
    let obligatorias = [(ReferenciaTabla::TipoMovimiento, Some(input.tipo_movimiento_id))];
    let opcionales = [
        (ReferenciaTabla::Categoria, input.categoria_id),
        (ReferenciaTabla::TipoConceptoPago, input.tipo_concepto_pago_id),
        (ReferenciaTabla::Cliente, input.cliente_id),
        (ReferenciaTabla::Trabajo, input.trabajo_id),
        (ReferenciaTabla::Empleado, input.empleado_id),
        (ReferenciaTabla::Factura, input.factura_id),
    ];

    for (tabla, id) in obligatorias.into_iter().chain(opcionales) {
        let Some(id) = id else { continue };
        if !repo.existe_referencia(tabla, id).await? {
            return Err(AppError::Validation(vec![crate::FieldError::new(
                tabla.campo(),
                "Validation.Common.ReferenciaInexistente",
            )
            .with_param("entidad", tabla.entidad())]));
        }
    }
    Ok(())
}
