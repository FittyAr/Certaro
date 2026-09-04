//! Seeding engine orchestrator for development and testing demo data.

mod data;
mod core_entities;
mod operations;
mod financials;

use chrono::Utc;
use certaro_application::result::AppResult;
use certaro_application::AppError;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait,
};
use serde::Serialize;

use crate::persistence::models::proyecto;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedSummary {
    pub categorias: usize,
    pub tipos_movimiento: usize,
    pub empleados: usize,
    pub asistencias: usize,
    pub clientes: usize,
    pub contactos: usize,
    pub proyectos: usize,
    pub trabajos: usize,
    pub ordenes_trabajo: usize,
    pub orden_trabajo_items: usize,
    pub certificados: usize,
    pub certificado_items: usize,
    pub facturas: usize,
    pub pagos_factura: usize,
    pub movimientos: usize,
    pub liquidaciones: usize,
    pub liquidacion_adelantos: usize,
    pub feriados: usize,
    pub adjuntos: usize,
}

pub async fn seed_demo_data(db: &DatabaseConnection) -> AppResult<SeedSummary> {
    let tx = db.begin().await.map_err(AppError::persistence)?;
    let now = Utc::now().to_rfc3339();

    // Idempotency: if seed data already exists, no-op (avoids UNIQUE violations on re-run).
    let already_seeded = proyecto::Entity::find()
        .filter(proyecto::Column::Numero.eq(1_i32))
        .one(&tx)
        .await
        .map_err(AppError::persistence)?;
    if already_seeded.is_some() {
        tx.rollback().await.map_err(AppError::persistence)?;
        return Ok(SeedSummary {
            categorias: 0,
            tipos_movimiento: 0,
            empleados: 0,
            asistencias: 0,
            clientes: 0,
            contactos: 0,
            proyectos: 0,
            trabajos: 0,
            ordenes_trabajo: 0,
            orden_trabajo_items: 0,
            certificados: 0,
            certificado_items: 0,
            facturas: 0,
            pagos_factura: 0,
            movimientos: 0,
            liquidaciones: 0,
            liquidacion_adelantos: 0,
            feriados: 0,
            adjuntos: 0,
        });
    }

    // 1 - 4: Catalogs, Tipos, Employees, Clients
    let (categorias_ids, tipos_ids, empleados_ids, clientes_ids, contactos_count) =
        core_entities::seed_catalogs_and_people(&tx, &now).await?;

    // 5 - 9: Projects, Jobs, Attendance, Work Orders, Certificates
    let (proyectos_ids, trabajos_ids, ordenes_ids, items_ids, asistencias_count, certificados_count, cert_items_count) =
        operations::seed_projects_and_jobs(&tx, &now, &clientes_ids, &empleados_ids).await?;

    // 10 - 14: Invoices, Cash Movements, Payroll, Holidays, Attachments
    let (facturas_ids, pagos_count, movimientos_count, liquidaciones_count, liq_adelantos_count, feriados_count, adjuntos_count) =
        financials::seed_financials_and_attachments(
            &tx,
            &now,
            &clientes_ids,
            &empleados_ids,
            &proyectos_ids,
            &trabajos_ids,
            &categorias_ids,
            &tipos_ids,
        ).await?;

    tx.commit().await.map_err(AppError::persistence)?;

    Ok(SeedSummary {
        categorias: categorias_ids.len(),
        tipos_movimiento: tipos_ids.len(),
        empleados: empleados_ids.len(),
        asistencias: asistencias_count,
        clientes: clientes_ids.len(),
        contactos: contactos_count,
        proyectos: proyectos_ids.len(),
        trabajos: trabajos_ids.len(),
        ordenes_trabajo: ordenes_ids.len(),
        orden_trabajo_items: items_ids.len(),
        certificados: certificados_count,
        certificado_items: cert_items_count,
        facturas: facturas_ids.len(),
        pagos_factura: pagos_count,
        movimientos: movimientos_count,
        liquidaciones: liquidaciones_count,
        liquidacion_adelantos: liq_adelantos_count,
        feriados: feriados_count,
        adjuntos: adjuntos_count,
    })
}
