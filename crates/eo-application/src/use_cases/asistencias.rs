//! Use cases of `asistencia`. See `docs/09-modulos-funcionales.md` §3.10.
//!
//! Writes go by natural key, `(empleado_id, fecha)`. The grid has no save button: every click is a
//! write, so an upsert that resolves the key itself is what keeps a double click from creating a
//! second row for the same day (INV-07).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::{DateTime, Datelike, NaiveDate, Utc, Weekday};
use eo_domain::entities::{AsistenciaEmpleado, Audit, Empleado, ResumenAsistencia};
use eo_domain::TipoJornada;
use tracing::info;
use uuid::Uuid;

use crate::dtos::asistencias::{
    AsistenciaCelda, AsistenciaDia, AsistenciaFila, AsistenciaGrilla, AsistenciaGrillaQuery,
    AsistenciaRangoInput, AsistenciaUpsertInput,
};
use crate::error::AppError;
use crate::ports::repositories::{Transaction, UnitOfWork};
use crate::ports::{ClockPort, IdGeneratorPort, SettingsStore};
use crate::result::AppResult;
use crate::use_cases::shared::{finish_read, finish_write, normalise};
use crate::validation;
use crate::validation::movimientos::ContextoFecha;

const ENTITY: &str = "Empleado";

pub struct AsistenciasService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    ids: Arc<dyn IdGeneratorPort>,
    settings: Arc<dyn SettingsStore>,
}

impl AsistenciasService {
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

    /// The whole grid in one round trip: employees, days, holidays and records. One query per
    /// table, not one per employee.
    pub async fn grilla(&self, query: AsistenciaGrillaQuery) -> AppResult<AsistenciaGrilla> {
        if query.desde > query.hasta {
            return Err(AppError::Validation(vec![crate::error::FieldError::new(
                "hasta",
                "Validation.Asistencia.RangoInvalid",
            )]));
        }
        let max_rango = self
            .settings
            .snapshot()
            .settlement
            .asistencia_max_rango_dias as i64;
        let dias = (query.hasta - query.desde).num_days() + 1;
        if dias > max_rango {
            return Err(AppError::Validation(vec![crate::error::FieldError::new(
                "hasta",
                "Validation.Asistencia.RangoExcedido",
            )
            .with_param("max", max_rango.to_string())
            .with_param("actual", dias.to_string())]));
        }

        let tx = self.uow.begin().await?;
        let outcome = async {
            let empleados = if query.empleado_ids.is_empty() {
                tx.empleados().activos().await?
            } else {
                let mut encontrados = Vec::new();
                for id in &query.empleado_ids {
                    if let Some(e) = tx.empleados().find_by_id(*id).await? {
                        encontrados.push(e);
                    }
                }
                encontrados
            };

            let registros = tx
                .asistencias()
                .del_periodo(query.desde, query.hasta, &query.empleado_ids)
                .await?;
            let feriados = tx.feriados().del_rango(query.desde, query.hasta).await?;

            Ok((empleados, registros, feriados))
        }
        .await;
        let (empleados, registros, feriados) = finish_read(tx, outcome).await?;

        let nombres_feriado: HashMap<NaiveDate, String> =
            feriados.into_iter().map(|f| (f.fecha, f.nombre)).collect();

        let dias: Vec<AsistenciaDia> = fechas_del_rango(query.desde, query.hasta)
            .map(|fecha| AsistenciaDia {
                fecha,
                dia_semana: dia_semana(fecha),
                es_fin_de_semana: es_fin_de_semana(fecha),
                es_feriado: nombres_feriado.contains_key(&fecha),
                feriado_nombre: nombres_feriado.get(&fecha).cloned(),
            })
            .collect();

        let mut por_empleado: HashMap<Uuid, HashMap<NaiveDate, AsistenciaEmpleado>> =
            HashMap::new();
        for registro in registros {
            por_empleado
                .entry(registro.empleado_id)
                .or_default()
                .insert(registro.fecha, registro);
        }

        let filas = empleados
            .into_iter()
            .map(|empleado| {
                let del_empleado = por_empleado.remove(&empleado.id).unwrap_or_default();
                let celdas: Vec<AsistenciaCelda> = dias
                    .iter()
                    .map(|dia| {
                        del_empleado
                            .get(&dia.fecha)
                            .map_or_else(|| AsistenciaCelda::vacia(dia.fecha), Into::into)
                    })
                    .collect();
                let resumen =
                    ResumenAsistencia::de_tipos(celdas.iter().filter_map(|c| c.tipo_jornada));
                AsistenciaFila {
                    empleado_id: empleado.id,
                    empleado_nombre: empleado.nombre,
                    empleado_cargo: empleado.cargo,
                    celdas,
                    resumen: resumen.into(),
                }
            })
            .collect();

        Ok(AsistenciaGrilla {
            desde: query.desde,
            hasta: query.hasta,
            dias,
            filas,
        })
    }

    /// One cell. `tipo_jornada: None` clears it, which is the last step of the click cycle.
    pub async fn upsert(&self, input: AsistenciaUpsertInput) -> AppResult<AsistenciaCelda> {
        validation::asistencias::validate(&input, &self.contexto_fecha())?;

        let now = self.clock.now_utc();
        let tx = self.uow.begin().await?;
        let outcome = async {
            let empleado = cargar_empleado(&*tx, input.empleado_id).await?;
            verificar_egreso(&empleado, input.fecha)?;

            let repo = tx.asistencias();
            let existente = repo
                .find_por_empleado_fecha(input.empleado_id, input.fecha)
                .await?;

            match (input.tipo_jornada, existente) {
                (None, Some(_)) => {
                    repo.soft_delete_por_empleado_fecha(input.empleado_id, input.fecha, now)
                        .await?;
                    Ok(AsistenciaCelda::vacia(input.fecha))
                }
                // Clearing a cell that has no record is not an error: the grid may be a click ahead
                // of what the server has.
                (None, None) => Ok(AsistenciaCelda::vacia(input.fecha)),
                (Some(tipo), Some(mut entity)) => {
                    entity.tipo_jornada = tipo;
                    entity.trabajo_id = input.trabajo_id;
                    entity.observaciones = normalise(input.observaciones.clone());
                    reactivar(&mut entity.audit, now);
                    repo.update(&entity).await?;
                    Ok(AsistenciaCelda::from(&entity))
                }
                (Some(tipo), None) => {
                    let entity = AsistenciaEmpleado {
                        id: self.ids.new_id(),
                        empleado_id: input.empleado_id,
                        fecha: input.fecha,
                        tipo_jornada: tipo,
                        trabajo_id: input.trabajo_id,
                        observaciones: normalise(input.observaciones.clone()),
                        audit: Audit::new(now),
                    };
                    repo.insert(&entity).await?;
                    Ok(AsistenciaCelda::from(&entity))
                }
            }
        }
        .await;
        let celda = finish_write(tx, outcome).await?;

        info!(empleado = %input.empleado_id, fecha = %input.fecha, "asistencia registrada");
        Ok(celda)
    }

    /// Bulk load of a range, which is what makes a full month bearable to enter.
    pub async fn upsert_rango(
        &self,
        input: AsistenciaRangoInput,
    ) -> AppResult<Vec<AsistenciaCelda>> {
        validation::asistencias::validate_rango(&input, &self.contexto_fecha())?;

        let now = self.clock.now_utc();
        let tx = self.uow.begin().await?;
        let outcome = async {
            let empleado = cargar_empleado(&*tx, input.empleado_id).await?;

            let feriados: HashSet<NaiveDate> = tx
                .feriados()
                .del_rango(input.desde, input.hasta)
                .await?
                .into_iter()
                .map(|f| f.fecha)
                .collect();

            let repo = tx.asistencias();
            let mut celdas = Vec::new();
            for fecha in fechas_del_rango(input.desde, input.hasta) {
                if input.solo_dias_habiles && (es_fin_de_semana(fecha) || feriados.contains(&fecha))
                {
                    continue;
                }
                if !empleado.admite_asistencia_en(fecha) {
                    continue;
                }

                match repo
                    .find_por_empleado_fecha(input.empleado_id, fecha)
                    .await?
                {
                    Some(mut entity) => {
                        entity.tipo_jornada = input.tipo_jornada;
                        entity.trabajo_id = input.trabajo_id;
                        reactivar(&mut entity.audit, now);
                        repo.update(&entity).await?;
                        celdas.push(AsistenciaCelda::from(&entity));
                    }
                    None => {
                        let entity = AsistenciaEmpleado {
                            id: self.ids.new_id(),
                            empleado_id: input.empleado_id,
                            fecha,
                            tipo_jornada: input.tipo_jornada,
                            trabajo_id: input.trabajo_id,
                            observaciones: None,
                            audit: Audit::new(now),
                        };
                        repo.insert(&entity).await?;
                        celdas.push(AsistenciaCelda::from(&entity));
                    }
                }
            }
            Ok(celdas)
        }
        .await;
        let celdas = finish_write(tx, outcome).await?;

        info!(
            empleado = %input.empleado_id,
            dias = celdas.len(),
            "asistencia cargada por rango"
        );
        Ok(celdas)
    }

    pub async fn delete(&self, empleado_id: Uuid, fecha: NaiveDate) -> AppResult<()> {
        let now = self.clock.now_utc();
        let tx = self.uow.begin().await?;
        let outcome = tx
            .asistencias()
            .soft_delete_por_empleado_fecha(empleado_id, fecha, now)
            .await;
        finish_write(tx, outcome).await
    }

    /// The next value of the click cycle, exposed so the frontend never has to know the order.
    #[must_use]
    pub fn siguiente_jornada(actual: Option<TipoJornada>) -> Option<TipoJornada> {
        TipoJornada::siguiente(actual)
    }

    fn contexto_fecha(&self) -> ContextoFecha {
        ContextoFecha::from_config(
            &self.settings.snapshot().validation,
            self.clock.now_utc().date_naive(),
        )
    }
}

pub(crate) fn fechas_del_rango(
    desde: NaiveDate,
    hasta: NaiveDate,
) -> impl Iterator<Item = NaiveDate> {
    desde.iter_days().take_while(move |d| *d <= hasta)
}

/// Clicking a day that was cleared earlier has to bring the row back rather than fail: the unique
/// index covers deleted rows too, so there is only ever one row per employee and day to revive.
fn reactivar(audit: &mut Audit, now: DateTime<Utc>) {
    if audit.is_deleted {
        audit.restore(now);
    } else {
        audit.touch(now);
    }
}

fn dia_semana(fecha: NaiveDate) -> u8 {
    // 1 is Monday, 7 is Sunday: the frontend renders the header from this, and zero-based weekdays
    // are how a Monday ends up under the Sunday column.
    u8::try_from(fecha.weekday().number_from_monday()).unwrap_or(1)
}

fn es_fin_de_semana(fecha: NaiveDate) -> bool {
    matches!(fecha.weekday(), Weekday::Sat | Weekday::Sun)
}

async fn cargar_empleado(tx: &dyn Transaction, id: Uuid) -> AppResult<Empleado> {
    tx.empleados()
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::not_found(ENTITY, id))
}

fn verificar_egreso(empleado: &Empleado, fecha: NaiveDate) -> AppResult<()> {
    if empleado.admite_asistencia_en(fecha) {
        return Ok(());
    }
    Err(AppError::Conflict {
        code: "ASISTENCIA_EMPLEADO_EGRESADO",
        message_key: "State.Asistencia.EmpleadoEgresado",
        params: [(
            "fechaEgreso".to_owned(),
            empleado
                .fecha_egreso
                .map(|f| f.to_string())
                .unwrap_or_default(),
        )]
        .into(),
    })
}
