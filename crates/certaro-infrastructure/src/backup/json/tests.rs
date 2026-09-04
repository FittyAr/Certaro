use super::common::*;
use sea_orm::{JsonValue, Value};

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
