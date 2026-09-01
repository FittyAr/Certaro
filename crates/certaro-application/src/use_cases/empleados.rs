//! Use cases of `empleados`. See `docs/09-modulos-funcionales.md` §3.9.

use std::sync::Arc;

use certaro_domain::entities::{Audit, Empleado};
use tracing::info;
use uuid::Uuid;

use crate::dtos::common::{ListQuery, LookupItem};
use crate::dtos::empleados::{EmpleadoDetalle, EmpleadoFiltroDto, EmpleadoInput, EmpleadoListItem};
use crate::error::AppError;
use crate::paging::PagedResult;
use crate::ports::repositories::{EmpleadoRepository, UnitOfWork};
use crate::ports::{ClockPort, IdGeneratorPort};
use crate::result::AppResult;
use crate::use_cases::shared::{
    checked_sort, finish_read, finish_write, normalise, parse_row_version,
};
use crate::validation;
use crate::validation::empleados::normalizar_dni;

const ENTITY: &str = "Empleado";

const SORTABLE: [&str; 5] = [
    "nombre",
    "cargo",
    "tarifaDiaria",
    "sueldoBase",
    "fechaIngreso",
];

pub struct EmpleadosService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    ids: Arc<dyn IdGeneratorPort>,
}

impl EmpleadosService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        ids: Arc<dyn IdGeneratorPort>,
    ) -> Self {
        Self { uow, clock, ids }
    }

    pub async fn list(
        &self,
        query: ListQuery<EmpleadoFiltroDto>,
    ) -> AppResult<PagedResult<EmpleadoListItem>> {
        let sort_by = checked_sort(query.sort_by.as_deref(), &SORTABLE)?;
        let page = query.page_request();
        page.validate()?;
        let filtro = query.filtro.into();

        let tx = self.uow.begin().await?;
        let result = tx
            .empleados()
            .search(&filtro, page, sort_by, query.sort_dir)
            .await;
        let page = finish_read(tx, result).await?;
        Ok(page.map(EmpleadoListItem::from))
    }

    pub async fn get(&self, id: Uuid) -> AppResult<EmpleadoDetalle> {
        let tx = self.uow.begin().await?;
        let loaded = load_detalle(tx.empleados(), id).await;
        finish_read(tx, loaded).await
    }

    pub async fn lookup(
        &self,
        solo_activos: Option<bool>,
        texto: Option<String>,
        limite: Option<u64>,
    ) -> AppResult<Vec<LookupItem>> {
        let tx = self.uow.begin().await?;
        let result = tx
            .empleados()
            .lookup(
                texto.as_deref(),
                solo_activos.unwrap_or(true),
                limite.unwrap_or(50),
            )
            .await;
        let empleados = finish_read(tx, result).await?;
        Ok(empleados
            .into_iter()
            .map(|e| {
                // The rate travels with the option: the settlement wizard shows it next to the name
                // and would otherwise need one call per employee.
                LookupItem::new(e.id, e.nombre)
                    .with_meta("tarifaDiaria", e.tarifa_diaria.to_decimal_string())
            })
            .collect())
    }

    /// The roles in use, for the filter dropdown.
    pub async fn cargos(&self) -> AppResult<Vec<String>> {
        let tx = self.uow.begin().await?;
        let result = tx.empleados().cargos().await;
        finish_read(tx, result).await
    }

    pub async fn create(&self, input: EmpleadoInput) -> AppResult<EmpleadoDetalle> {
        validation::empleados::validate(&input)?;

        let now = self.clock.now_utc();
        let entity = self.build(self.ids.new_id(), &input, Audit::new(now));

        let tx = self.uow.begin().await?;
        let outcome = async {
            tx.empleados().insert(&entity).await?;
            load_detalle(tx.empleados(), entity.id).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, "empleado creado");
        Ok(detalle)
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: EmpleadoInput,
        row_version: &str,
    ) -> AppResult<EmpleadoDetalle> {
        validation::empleados::validate(&input)?;
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.empleados();
            let previo = repo
                .find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            let mut entity = self.build(id, &input, previo.audit);
            entity.audit.touch(now);

            repo.update(&entity, esperado).await?;
            load_detalle(repo, id).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, "empleado actualizado");
        Ok(detalle)
    }

    /// An employee with settlements is not deleted: the settlements are the record of what was
    /// paid, and deleting the employee would leave them orphaned. Deactivating is the way out.
    pub async fn delete(&self, id: Uuid, row_version: &str) -> AppResult<()> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.empleados();
            repo.find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            for (count, code, key) in [
                (
                    repo.count_liquidaciones(id).await?,
                    "EMPLEADO_CON_LIQUIDACIONES",
                    "Conflict.Empleado.ConLiquidaciones",
                ),
                (
                    repo.count_movimientos(id).await?,
                    "EMPLEADO_CON_MOVIMIENTOS",
                    "Conflict.Empleado.ConMovimientos",
                ),
            ] {
                if count > 0 {
                    return Err(AppError::DependencyInUse {
                        code,
                        message_key: key,
                        params: [("count".to_owned(), count.to_string())].into(),
                    });
                }
            }

            repo.soft_delete(id, esperado, now).await
        }
        .await;
        finish_write(tx, outcome).await?;

        info!(%id, "empleado eliminado");
        Ok(())
    }

    fn build(&self, id: Uuid, input: &EmpleadoInput, audit: Audit) -> Empleado {
        Empleado {
            id,
            nombre: input.nombre.trim().to_owned(),
            // Stored as digits: the same document typed with or without dots has to compare equal.
            dni: normalise(input.dni.clone()).map(|d| normalizar_dni(&d)),
            cargo: normalise(input.cargo.clone()),
            sueldo_base: input.sueldo_base,
            pago_frecuencia: input.pago_frecuencia,
            tarifa_diaria: input.tarifa_diaria,
            multiplicador_sabado: input.multiplicador_sabado,
            multiplicador_domingo: input.multiplicador_domingo,
            multiplicador_feriado: input.multiplicador_feriado,
            email: normalise(input.email.clone()),
            telefono: normalise(input.telefono.clone()),
            fecha_ingreso: input.fecha_ingreso,
            fecha_egreso: input.fecha_egreso,
            activo: input.activo,
            audit,
        }
    }
}

async fn load_detalle(repo: &dyn EmpleadoRepository, id: Uuid) -> AppResult<EmpleadoDetalle> {
    let entity = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::not_found(ENTITY, id))?;
    let libre = repo.count_liquidaciones(id).await? == 0 && repo.count_movimientos(id).await? == 0;
    Ok(EmpleadoDetalle::build(&entity, libre))
}
