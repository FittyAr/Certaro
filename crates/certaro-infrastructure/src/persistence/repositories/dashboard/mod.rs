//! Aggregated reads for the dashboard and the commercial analysis.
//! See `docs/06-casos-de-uso-y-formulas.md` §4.5, §4.6, §7 y §9.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use certaro_application::ports::repositories::{
    DashboardRepository, EstadoBase, FacturaPendiente, MovimientoResumen, RentabilidadFila,
    SortDir, TotalMensual, TotalPorNombre,
};
use certaro_application::{AppError, AppResult};
use sea_orm::{DatabaseTransaction, DbBackend, FromQueryResult, Statement, Value};
use uuid::Uuid;

mod analytics;
mod common;
mod metrics;

use common::ConteoRow;

pub struct SeaOrmDashboardRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmDashboardRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    pub(crate) fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }

    pub(crate) async fn scalar(&self, sql: &str, values: Vec<Value>) -> AppResult<u64> {
        let row = ConteoRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            sql,
            values,
        ))
        .one(self.conn())
        .await
        .map_err(AppError::persistence)?;
        Ok(row.map_or(0, |r| r.total.max(0) as u64))
    }
}

#[async_trait]
impl DashboardRepository for SeaOrmDashboardRepository {
    async fn resumen_rango(
        &self,
        desde: DateTime<Utc>,
        hasta: DateTime<Utc>,
    ) -> AppResult<MovimientoResumen> {
        self.impl_resumen_rango(desde, hasta).await
    }

    async fn clientes_activos(&self, desde: DateTime<Utc>, hasta: DateTime<Utc>) -> AppResult<u64> {
        self.impl_clientes_activos(desde, hasta).await
    }

    async fn trabajos_pendientes(&self) -> AppResult<u64> {
        self.impl_trabajos_pendientes().await
    }

    async fn proyectos_pausadas(&self) -> AppResult<u64> {
        self.impl_proyectos_pausadas().await
    }

    async fn facturas_vencidas(&self, umbral: NaiveDate) -> AppResult<u64> {
        self.impl_facturas_vencidas(umbral).await
    }

    async fn liquidaciones_pendientes(&self, anio: i32, mes: u32) -> AppResult<u64> {
        self.impl_liquidaciones_pendientes(anio, mes).await
    }

    async fn top_clientes(
        &self,
        desde: DateTime<Utc>,
        hasta: DateTime<Utc>,
        limite: u64,
    ) -> AppResult<Vec<TotalPorNombre>> {
        self.impl_top_clientes(desde, hasta, limite).await
    }

    async fn gastos_por_categoria(
        &self,
        desde: DateTime<Utc>,
        hasta: DateTime<Utc>,
        limite: u64,
    ) -> AppResult<Vec<TotalPorNombre>> {
        self.impl_gastos_por_categoria(desde, hasta, limite).await
    }

    async fn serie_mensual(&self, anio: i32) -> AppResult<Vec<TotalMensual>> {
        self.impl_serie_mensual(anio).await
    }

    async fn rentabilidad_proyectos(
        &self,
        dir: SortDir,
        limite: u64,
    ) -> AppResult<Vec<RentabilidadFila>> {
        self.impl_rentabilidad_proyectos(dir, limite).await
    }

    async fn rentabilidad_trabajos(
        &self,
        proyecto_id: Option<Uuid>,
        limite: u64,
    ) -> AppResult<Vec<RentabilidadFila>> {
        self.impl_rentabilidad_trabajos(proyecto_id, limite).await
    }

    async fn facturas_pendientes(
        &self,
        cliente_id: Option<Uuid>,
        incluir_pagadas: bool,
    ) -> AppResult<Vec<FacturaPendiente>> {
        self.impl_facturas_pendientes(cliente_id, incluir_pagadas).await
    }

    async fn estado_base(&self) -> AppResult<EstadoBase> {
        self.impl_estado_base().await
    }
}
