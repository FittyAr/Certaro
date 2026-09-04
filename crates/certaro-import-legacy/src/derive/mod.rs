//! Phase 5: derived data. See `docs/15-migracion-de-datos.md` §5.
//!
//! Populates three new tables (certificados, certificado_items, liquidacion_adelantos) and
//! performs additional derivations (contacts from email, holidays from config, invoice state
//! reclassification).

pub mod adelantos;
pub mod certificados;
pub mod contactos;
pub mod facturas;
pub mod feriados;

use anyhow::Result;
use sea_orm::DatabaseTransaction;
use sqlx::sqlite::SqlitePool;

use crate::report::{DerivedReport, ImportReport};

pub use adelantos::derive_liquidacion_adelantos;
pub use certificados::derive_certificados;
pub use contactos::derive_contactos;
pub use facturas::reclassify_facturas;
pub use feriados::derive_feriados;

/// Runs all derivations.
pub async fn derive_all(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    report: &mut ImportReport,
) -> Result<()> {
    let cert_count = derive_certificados(db, report).await?;
    let adelantos_count = derive_liquidacion_adelantos(db, report).await?;
    let contactos_count = derive_contactos(db, report).await?;
    let feriados_count = derive_feriados(db, legacy, report).await?;
    let reclasificadas = reclassify_facturas(db).await?;

    report.derived = DerivedReport {
        certificados: cert_count.0,
        certificado_items: cert_count.1,
        liquidacion_adelantos: adelantos_count,
        contactos_creados: contactos_count,
        feriados_recuperados: feriados_count,
        facturas_reclasificadas: reclasificadas,
        vencimientos_estimados: 0, // Already counted during transfer.
    };

    Ok(())
}
