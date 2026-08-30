//! Use cases of `feriados`. See `docs/13-servicios-externos-y-archivos.md` §3.
//!
//! The calendar lives in the database and the settlement only ever reads the table. The network is
//! touched by the sync and by nothing else, so a payroll run cannot be blocked by a service being
//! down.

use std::sync::Arc;

use chrono::{Datelike, NaiveDate};
use eo_domain::entities::{Feriado, OrigenFeriado};
use tracing::{info, warn};

use crate::dtos::feriados::{FeriadoDto, FeriadoInput, FeriadoSyncResult};
use crate::ports::repositories::UnitOfWork;
use crate::ports::{ClockPort, HolidayProvider, SettingsStore};
use crate::result::AppResult;
use crate::use_cases::shared::{finish_read, finish_write};

pub struct FeriadosService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    provider: Arc<dyn HolidayProvider>,
    settings: Arc<dyn SettingsStore>,
}

impl FeriadosService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        provider: Arc<dyn HolidayProvider>,
        settings: Arc<dyn SettingsStore>,
    ) -> Self {
        Self {
            uow,
            clock,
            provider,
            settings,
        }
    }

    pub async fn list(&self, anio: i32) -> AppResult<Vec<FeriadoDto>> {
        let tx = self.uow.begin().await?;
        let result = tx.feriados().del_anio(anio).await;
        let feriados = finish_read(tx, result).await?;
        Ok(feriados.into_iter().map(Into::into).collect())
    }

    /// Fetches the years given and inserts only what is missing. A year the provider cannot serve
    /// is counted and skipped: the calendar stays as it was rather than being emptied.
    pub async fn sync(&self, anios: Vec<i32>) -> AppResult<FeriadoSyncResult> {
        let now = self.clock.now_utc();
        let mut traidos = Vec::new();
        let mut anios_con_error = 0_u32;

        for anio in anios {
            match self.provider.fetch(anio).await {
                Ok(mut feriados) => traidos.append(&mut feriados),
                Err(e) => {
                    anios_con_error += 1;
                    warn!(anio, error = %e, "no se pudieron traer los feriados del año");
                }
            }
        }

        let feriados: Vec<Feriado> = traidos
            .into_iter()
            .map(|f| Feriado {
                origen: OrigenFeriado::Api,
                created_at: now,
                updated_at: None,
                ..f
            })
            .collect();

        let tx = self.uow.begin().await?;
        let outcome = tx.feriados().insertar_faltantes(&feriados).await;
        let agregados = finish_write(tx, outcome).await?;

        info!(agregados, total = feriados.len(), "feriados sincronizados");
        Ok(FeriadoSyncResult {
            agregados,
            total: feriados.len() as u64,
            anios_con_error,
        })
    }

    /// Sync on start, only when the calendar of the current year is empty. Failure is logged and
    /// swallowed: the application has to open regardless.
    pub async fn sync_al_iniciar(&self) -> AppResult<()> {
        let config = self.settings.snapshot().settlement;
        if !config.sincronizar_feriados_al_iniciar {
            return Ok(());
        }

        let anio_actual = self.clock.now_utc().date_naive().year();
        let tx = self.uow.begin().await?;
        let result = tx.feriados().count_anio(anio_actual).await;
        let existentes = finish_read(tx, result).await?;
        if existentes > 0 {
            return Ok(());
        }

        let cantidad = i32::from(config.anios_feriados_a_sincronizar.max(1));
        let anios = (anio_actual..anio_actual + cantidad).collect();
        if let Err(e) = self.sync(anios).await {
            warn!(error = %e, "la sincronización inicial de feriados falló");
        }
        Ok(())
    }

    pub async fn add(&self, input: FeriadoInput) -> AppResult<Vec<FeriadoDto>> {
        let now = self.clock.now_utc();
        let entity = Feriado {
            fecha: input.fecha,
            nombre: input.nombre.trim().to_owned(),
            tipo: None,
            origen: OrigenFeriado::Manual,
            created_at: now,
            updated_at: Some(now),
        };

        let anio = entity.fecha.year();
        let tx = self.uow.begin().await?;
        let outcome = async {
            tx.feriados().upsert_manual(&entity).await?;
            tx.feriados().del_anio(anio).await
        }
        .await;
        let feriados = finish_write(tx, outcome).await?;

        info!(fecha = %input.fecha, "feriado manual agregado");
        Ok(feriados.into_iter().map(Into::into).collect())
    }

    pub async fn delete(&self, fecha: NaiveDate) -> AppResult<Vec<FeriadoDto>> {
        let anio = fecha.year();
        let tx = self.uow.begin().await?;
        let outcome = async {
            tx.feriados().delete(fecha).await?;
            tx.feriados().del_anio(anio).await
        }
        .await;
        let feriados = finish_write(tx, outcome).await?;

        info!(%fecha, "feriado eliminado");
        Ok(feriados.into_iter().map(Into::into).collect())
    }
}
