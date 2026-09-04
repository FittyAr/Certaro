use certaro_application::result::AppResult;
use certaro_application::AppError;
use sea_orm::{ConnectionTrait, DatabaseBackend, JsonValue, Statement, Value};
use serde::{Deserialize, Serialize};

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
pub fn identificador_valido(nombre: &str) -> bool {
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

/// Reads a column as JSON, trying the three types SQLite actually stores.
pub fn valor_json(fila: &sea_orm::QueryResult, columna: &str) -> JsonValue {
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

pub fn valor_sql(valor: &JsonValue) -> Value {
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

pub fn rechazo(clave: &str, valor: &str) -> AppError {
    AppError::Validation(vec![certaro_application::error::FieldError::new(
        "archivo", clave,
    )
    .with_param("valor", valor)])
}
