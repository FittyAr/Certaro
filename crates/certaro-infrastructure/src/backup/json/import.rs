use std::collections::BTreeMap;

use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, DatabaseTransaction, Statement,
    TransactionTrait, Value,
};
use tracing::info;

use certaro_application::ports::ImportResumen;
use certaro_application::result::AppResult;
use certaro_application::AppError;

use super::common::{columnas_de, rechazo, valor_sql, Documento, TABLAS, FORMAT_VERSION};

/// Replaces the contents of the database with the dump, all inside one transaction.
///
/// Atomicity is the point: the legacy import ran table by table without one, so a failure on the
/// twelfth table left eleven replaced and the rest stale, with no way to tell.
pub async fn importar(
    db: &DatabaseConnection,
    documento: &Documento,
    schema_version: &str,
) -> AppResult<ImportResumen> {
    if documento.format_version != FORMAT_VERSION {
        return Err(rechazo(
            "Validation.Import.FormatoIncompatible",
            &documento.format_version.to_string(),
        ));
    }
    if documento.schema_version != schema_version {
        return Err(rechazo(
            "Validation.Import.EsquemaIncompatible",
            &documento.schema_version,
        ));
    }

    // Off for the duration of the load, because the file arrives in dependency order but a table
    // with a self-reference — categories with a parent — cannot be satisfied row by row.
    ejecutar(db, "PRAGMA foreign_keys = OFF").await?;

    let resultado = cargar(db, documento).await;

    ejecutar(db, "PRAGMA foreign_keys = ON").await?;

    let resumen = resultado?;

    let violaciones = db
        .query_all(Statement::from_string(
            DatabaseBackend::Sqlite,
            "PRAGMA foreign_key_check".to_owned(),
        ))
        .await
        .map_err(AppError::persistence)?;
    if !violaciones.is_empty() {
        // The transaction has already been rolled back inside `cargar` if it failed; reaching here
        // means the rows loaded but do not hold together, which is equally unusable.
        return Err(rechazo(
            "Validation.Import.IntegridadReferencial",
            &violaciones.len().to_string(),
        ));
    }

    info!(
        tablas = resumen.tablas,
        filas = resumen.filas,
        "base importada desde JSON"
    );
    Ok(resumen)
}

async fn cargar(db: &DatabaseConnection, documento: &Documento) -> AppResult<ImportResumen> {
    let tx = db.begin().await.map_err(AppError::persistence)?;
    let resultado = cargar_en(&tx, documento).await;
    match resultado {
        Ok(resumen) => {
            tx.commit().await.map_err(AppError::persistence)?;
            Ok(resumen)
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

async fn cargar_en(tx: &DatabaseTransaction, documento: &Documento) -> AppResult<ImportResumen> {
    let mut filas_totales = 0_u64;

    // Deleted in reverse dependency order so the children go before their parents.
    for tabla in TABLAS.iter().rev() {
        tx.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            format!("DELETE FROM \"{tabla}\""),
        ))
        .await
        .map_err(AppError::persistence)?;
    }

    let esperadas: BTreeMap<&str, Vec<String>> = {
        let mut mapa = BTreeMap::new();
        for tabla in TABLAS {
            mapa.insert(tabla, columnas_de(tx, tabla).await?);
        }
        mapa
    };

    for tabla in &documento.tables {
        let Some(permitidas) = esperadas.get(tabla.name.as_str()) else {
            return Err(rechazo("Validation.Import.TablaDesconocida", &tabla.name));
        };
        for columna in &tabla.columns {
            if !permitidas.contains(columna) {
                return Err(rechazo("Validation.Import.ColumnaDesconocida", columna));
            }
        }
        if tabla.rows.is_empty() {
            continue;
        }

        let lista = tabla
            .columns
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let marcadores = vec!["?"; tabla.columns.len()].join(", ");
        let sql = format!(
            "INSERT INTO \"{}\" ({lista}) VALUES ({marcadores})",
            tabla.name
        );

        for fila in &tabla.rows {
            if fila.len() != tabla.columns.len() {
                return Err(rechazo("Validation.Import.FilaIncompleta", &tabla.name));
            }
            // Values always go as parameters. Nothing from the file is ever concatenated into SQL.
            let valores: Vec<Value> = fila.iter().map(valor_sql).collect();
            tx.execute(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                &sql,
                valores,
            ))
            .await
            .map_err(AppError::persistence)?;
            filas_totales += 1;
        }
    }

    Ok(ImportResumen {
        tablas: documento.tables.len() as u32,
        filas: filas_totales,
    })
}

async fn ejecutar(db: &DatabaseConnection, sql: &str) -> AppResult<()> {
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        sql.to_owned(),
    ))
    .await
    .map_err(AppError::persistence)?;
    Ok(())
}
