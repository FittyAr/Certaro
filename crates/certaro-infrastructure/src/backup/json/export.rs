use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement,
};
use tracing::info;

use certaro_application::ports::ImportResumen;
use certaro_application::result::AppResult;
use certaro_application::AppError;

use super::common::{columnas_de, valor_json, Documento, Tabla, TABLAS, FORMAT_VERSION};

pub async fn exportar(
    db: &DatabaseConnection,
    app_version: &str,
    schema_version: String,
    exported_at: String,
) -> AppResult<(Documento, ImportResumen)> {
    let mut tables = Vec::with_capacity(TABLAS.len());
    let mut filas_totales = 0_u64;

    for tabla in TABLAS {
        let columns = columnas_de(db, tabla).await?;
        let lista = columns
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let filas = db
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!("SELECT {lista} FROM \"{tabla}\""),
            ))
            .await
            .map_err(AppError::persistence)?;

        let mut rows = Vec::with_capacity(filas.len());
        for fila in &filas {
            let mut valores = Vec::with_capacity(columns.len());
            for columna in &columns {
                valores.push(valor_json(fila, columna));
            }
            rows.push(valores);
        }
        filas_totales += rows.len() as u64;
        tables.push(Tabla {
            name: tabla.to_owned(),
            columns,
            rows,
        });
    }

    let resumen = ImportResumen {
        tablas: tables.len() as u32,
        filas: filas_totales,
    };
    info!(
        tablas = resumen.tablas,
        filas = resumen.filas,
        "base exportada a JSON"
    );

    Ok((
        Documento {
            format_version: FORMAT_VERSION,
            app_version: app_version.to_owned(),
            schema_version,
            exported_at,
            tables,
        },
        resumen,
    ))
}
