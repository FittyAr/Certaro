//! JSON of movements. See `docs/12-reportes-y-exportaciones.md` §2.5.
//!
//! Amounts are **strings** with four decimals, not JSON numbers: a consumer that parses them as
//! doubles loses cents, and the legacy export handed out floating point. Keys are camelCase, enums
//! travel by name, and null fields are omitted.

use chrono::NaiveDate;
use eo_application::dtos::movimientos::MovimientoListItem;
use eo_application::dtos::reportes::{GeneratedReport, ReporteMovimientos};
use eo_application::result::AppResult;
use serde::Serialize;

use super::{filename, io_error, ReportContext};

/// Bumped whenever the shape changes, so a consumer can tell which contract it is reading.
const VERSION: u32 = 1;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Documento {
    version: u32,
    generado_en: String,
    reporte: &'static str,
    filtro: serde_json::Value,
    resumen: Resumen,
    items: Vec<Item>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Resumen {
    total_ingresos: String,
    total_gastos: String,
    balance: String,
    cantidad: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    id: String,
    fecha: NaiveDate,
    concepto: String,
    monto: String,
    cantidad: String,
    total: String,
    tipo_movimiento: String,
    es_ingreso: bool,
    moneda: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    categoria: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cliente: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    obra: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trabajo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cotizacion_aplicada: Option<String>,
}

impl From<&MovimientoListItem> for Item {
    fn from(m: &MovimientoListItem) -> Self {
        Self {
            id: m.id.to_string(),
            fecha: m.fecha.with_timezone(&chrono::Local).date_naive(),
            concepto: m.concepto.clone(),
            monto: m.monto.to_decimal_string(),
            cantidad: m.cantidad.to_decimal_string(),
            total: m.total.to_decimal_string(),
            tipo_movimiento: m.tipo_movimiento_nombre.clone(),
            es_ingreso: m.es_ingreso,
            moneda: m.moneda.iso(),
            categoria: m.categoria_nombre.clone(),
            cliente: m.cliente_nombre.clone(),
            obra: m.obra_nombre.clone(),
            trabajo: m.trabajo_descripcion.clone(),
            cotizacion_aplicada: m.cotizacion_aplicada.map(|c| c.to_decimal_string()),
        }
    }
}

pub fn movimientos(data: &ReporteMovimientos, ctx: &ReportContext) -> AppResult<GeneratedReport> {
    let filtro =
        serde_json::to_value(&data.filtro).map_err(|e| io_error("export.json.filtro", e))?;

    let documento = Documento {
        version: VERSION,
        generado_en: ctx
            .generado_en
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        reporte: "Movimientos",
        filtro: prune_nulls(filtro),
        resumen: Resumen {
            total_ingresos: data.resumen.total_ingresos.to_decimal_string(),
            total_gastos: data.resumen.total_gastos.to_decimal_string(),
            balance: data.resumen.balance.to_decimal_string(),
            cantidad: data.resumen.cantidad,
        },
        items: data.items.iter().map(Item::from).collect(),
    };

    let bytes =
        serde_json::to_vec_pretty(&documento).map_err(|e| io_error("export.json.serialize", e))?;

    Ok(GeneratedReport {
        bytes,
        registros: data.items.len() as u64,
        nombre_sugerido: filename::movimientos(ctx.generado_en, filename::FormatoExport::Json),
    })
}

/// Drops the null members of an object. The filter DTO has one field per criterion and most of
/// them are empty on any real export, so keeping them would bury the two that matter.
fn prune_nulls(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let kept: serde_json::Map<String, serde_json::Value> = map
                .into_iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| (k, prune_nulls(v)))
                .collect();
            serde_json::Value::Object(kept)
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporting::tests_support::{contexto, movimiento, reporte};

    fn documento(data: &ReporteMovimientos) -> serde_json::Value {
        let generado = movimientos(data, &contexto()).unwrap();
        serde_json::from_slice(&generado.bytes).unwrap()
    }

    #[test]
    fn json_importes_son_string() {
        let doc = documento(&reporte(vec![movimiento("Cable", "1500.5", "2")]));
        for clave in ["totalIngresos", "totalGastos", "balance"] {
            assert!(doc["resumen"][clave].is_string(), "{clave} no es string");
        }
        let item = &doc["items"][0];
        for clave in ["monto", "cantidad", "total"] {
            assert!(item[clave].is_string(), "{clave} no es string");
        }
        assert_eq!(item["total"], "3001.0000");
    }

    #[test]
    fn json_es_camel_case() {
        let doc = documento(&reporte(vec![movimiento("Cable", "10", "1")]));
        fn revisar(value: &serde_json::Value) {
            if let serde_json::Value::Object(map) = value {
                for (clave, hijo) in map {
                    assert!(
                        clave.chars().next().is_some_and(char::is_lowercase),
                        "la clave {clave} no es camelCase"
                    );
                    revisar(hijo);
                }
            }
        }
        revisar(&doc);
    }

    #[test]
    fn json_lleva_version_y_fecha_de_generacion_en_utc() {
        let doc = documento(&reporte(vec![]));
        assert_eq!(doc["version"], 1);
        assert_eq!(doc["generadoEn"], "2026-08-29T15:30:12Z");
    }

    #[test]
    fn json_omite_los_campos_nulos() {
        let doc = documento(&reporte(vec![movimiento("Cable", "10", "1")]));
        assert!(doc["items"][0].get("cotizacionAplicada").is_none());
        assert!(doc["filtro"].as_object().unwrap().is_empty());
    }

    #[test]
    fn json_esta_indentado_para_poder_leerlo() {
        let generado = movimientos(&reporte(vec![]), &contexto()).unwrap();
        let texto = String::from_utf8(generado.bytes).unwrap();
        assert!(texto.contains("\n  \"version\""), "{texto}");
    }

    #[test]
    fn reporte_vacio_genera_un_json_valido_con_items_vacios() {
        let doc = documento(&reporte(vec![]));
        assert_eq!(doc["items"].as_array().unwrap().len(), 0);
        assert_eq!(doc["resumen"]["cantidad"], 0);
    }
}
