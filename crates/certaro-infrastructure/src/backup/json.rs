//! The JSON dump of the database. See `docs/13-servicios-externos-y-archivos.md` §5.
//!
//! A verbatim dump, not a report: amounts stay scaled integers and dates stay the text SQLite holds.
//! Fidelity is the whole point — this has to be able to rebuild the database exactly.

use std::collections::BTreeMap;

use certaro_application::ports::ImportResumen;
use certaro_application::result::AppResult;
use certaro_application::AppError;
use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, DatabaseTransaction, JsonValue,
    Statement, TransactionTrait, Value,
};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Bumped when the shape of the file changes, so an old export can still be recognised — and
/// refused with a clear message rather than half-imported.
pub const FORMAT_VERSION: u32 = 2;

/// Every table, in an order that satisfies the foreign keys when inserting.
///
/// Taken from the model rather than from `sqlite_master`, so the migration bookkeeping tables never
/// take part and an unknown name in a file is caught by comparing against this.
pub const TABLAS: [&str; 38] = [
    "roles",
    "permisos",
    "usuarios",
    "usuario_roles",
    "rol_permisos",
    "sesiones",
    "auth_externo",
    "tipos_movimiento",
    "categorias",
    "tipos_concepto_pago",
    "clientes",
    "cliente_contactos",
    "proyectos",
    "trabajos",
    "facturas",
    "pagos_factura",
    "ordenes_trabajo",
    "orden_trabajo_items",
    "certificados",
    "certificado_items",
    "empleados",
    "asistencias_empleado",
    "liquidaciones",
    "liquidacion_adelantos",
    "movimientos",
    "adjuntos",
    "feriados",
    "kanban_tableros",
    "kanban_columnas",
    "kanban_tarjetas",
    "kanban_etiquetas",
    "kanban_tarjeta_etiquetas",
    "kanban_tarjeta_checklist",
    "kanban_tarjeta_asignados",
    "calendario_grupos_recurso",
    "calendario_recursos",
    "calendario_eventos",
    "calendario_evento_recursos",
];

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Documento {
    pub format_version: u32,
    pub app_version: String,
    /// Name of the last applied migration. Compared on import: a dump of a different schema is
    /// refused, which is what the legacy format made impossible by not recording it.
    pub schema_version: String,
    pub exported_at: String,
    /// Insertion order matters, so this is a vector of pairs and not a map.
    pub tables: Vec<Tabla>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tabla {
    pub name: String,
    /// Declared once for the whole table. The legacy format repeated them on every row, which
    /// multiplied the size of a real database several times over.
    pub columns: Vec<String>,
    pub rows: Vec<Vec<JsonValue>>,
}

/// A table name that is not a plain identifier never reaches SQL, no matter what the whitelist says.
fn identificador_valido(nombre: &str) -> bool {
    !nombre.is_empty()
        && nombre
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && nombre
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The columns of a table, read from the schema itself so the whitelist cannot drift from it.
pub async fn columnas_de<C: ConnectionTrait>(conn: &C, tabla: &str) -> AppResult<Vec<String>> {
    if !identificador_valido(tabla) {
        return Err(rechazo("Validation.Import.TablaDesconocida", tabla));
    }
    let filas = conn
        .query_all(Statement::from_string(
            DatabaseBackend::Sqlite,
            format!("PRAGMA table_info(\"{tabla}\")"),
        ))
        .await
        .map_err(AppError::persistence)?;

    filas
        .iter()
        .map(|fila| {
            fila.try_get::<String>("", "name")
                .map_err(AppError::persistence)
        })
        .collect()
}

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

/// Reads a column as JSON, trying the three types SQLite actually stores.
fn valor_json(fila: &sea_orm::QueryResult, columna: &str) -> JsonValue {
    if let Ok(v) = fila.try_get::<Option<i64>>("", columna) {
        return v.map_or(JsonValue::Null, JsonValue::from);
    }
    if let Ok(v) = fila.try_get::<Option<f64>>("", columna) {
        return v.map_or(JsonValue::Null, JsonValue::from);
    }
    if let Ok(v) = fila.try_get::<Option<String>>("", columna) {
        return v.map_or(JsonValue::Null, JsonValue::from);
    }
    if let Ok(v) = fila.try_get::<Option<Vec<u8>>>("", columna) {
        // `row_version` is a BLOB. Base64 would be shorter, but an array of numbers survives a
        // hand edit of the file, which is a thing people do to a JSON backup.
        return v.map_or(JsonValue::Null, |bytes| {
            JsonValue::from(bytes.into_iter().map(JsonValue::from).collect::<Vec<_>>())
        });
    }
    JsonValue::Null
}

fn valor_sql(valor: &JsonValue) -> Value {
    match valor {
        JsonValue::Null => Value::String(None),
        JsonValue::Bool(b) => Value::Bool(Some(*b)),
        JsonValue::Number(n) => n.as_i64().map_or_else(
            || Value::Double(n.as_f64()),
            |entero| Value::BigInt(Some(entero)),
        ),
        JsonValue::String(s) => Value::String(Some(Box::new(s.clone()))),
        JsonValue::Array(items) => {
            let bytes = items
                .iter()
                .filter_map(|v| v.as_u64())
                .map(|n| n as u8)
                .collect::<Vec<u8>>();
            Value::Bytes(Some(Box::new(bytes)))
        }
        JsonValue::Object(_) => Value::String(None),
    }
}

fn rechazo(clave: &str, valor: &str) -> AppError {
    AppError::Validation(vec![certaro_application::error::FieldError::new(
        "archivo", clave,
    )
    .with_param("valor", valor)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_lista_de_tablas_no_tiene_repetidos_ni_nombres_raros() {
        let mut vistas = std::collections::BTreeSet::new();
        for tabla in TABLAS {
            assert!(vistas.insert(tabla), "{tabla} está repetida");
            assert!(identificador_valido(tabla), "{tabla}");
        }
    }

    #[test]
    fn un_nombre_con_sql_adentro_no_es_un_identificador() {
        for nombre in [
            "movimientos; DROP TABLE clientes",
            "\"movimientos\"",
            "1tabla",
            "",
            "tabla-guion",
        ] {
            assert!(!identificador_valido(nombre), "{nombre}");
        }
    }

    #[test]
    fn las_dependencias_van_antes_que_quienes_las_usan() {
        let posicion = |tabla: &str| TABLAS.iter().position(|t| *t == tabla).unwrap();
        assert!(posicion("clientes") < posicion("proyectos"));
        assert!(posicion("proyectos") < posicion("trabajos"));
        assert!(posicion("trabajos") < posicion("facturas"));
        assert!(posicion("facturas") < posicion("pagos_factura"));
        assert!(posicion("ordenes_trabajo") < posicion("certificados"));
        assert!(posicion("empleados") < posicion("liquidaciones"));
        assert!(posicion("liquidaciones") < posicion("liquidacion_adelantos"));
        assert!(posicion("tipos_movimiento") < posicion("movimientos"));
    }

    #[test]
    fn los_valores_json_se_convierten_al_tipo_de_sqlite() {
        assert!(matches!(
            valor_sql(&JsonValue::from(42_i64)),
            Value::BigInt(Some(42))
        ));
        assert!(matches!(valor_sql(&JsonValue::Null), Value::String(None)));
        assert!(matches!(
            valor_sql(&JsonValue::from(vec![
                JsonValue::from(1),
                JsonValue::from(2)
            ])),
            Value::Bytes(_)
        ));
    }
}
