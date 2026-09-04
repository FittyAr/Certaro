use pretty_assertions::assert_eq;
use super::common::*;
use certaro_application::dtos::common::ListQuery;
use certaro_application::dtos::movimientos::MovimientoFiltroDto;
use certaro_domain::constants::tipos_movimiento;
use certaro_domain::Money;
use chrono::NaiveDate;

#[tokio::test]
async fn el_total_se_deriva_y_nunca_se_guarda() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;

    let creado = f
        .movimientos
        .create(input("Cable", "1500.5000", "2.0000", categoria))
        .await
        .unwrap();

    assert_eq!(creado.item.total.to_decimal_string(), "3001.0000");

    // The column does not exist; anything that claims a total computed it.
    let columnas =
        f.db.query_all(sea_orm::Statement::from_string(
            sea_orm::DbBackend::Sqlite,
            "PRAGMA table_info(movimientos)".to_owned(),
        ))
        .await
        .unwrap();
    let nombres: Vec<String> = columnas
        .iter()
        .map(|r| r.try_get::<String>("", "name").unwrap())
        .collect();
    assert!(!nombres.iter().any(|n| n == "total"));
}

#[tokio::test]
async fn el_listado_trae_el_nombre_del_tipo_y_de_la_categoria() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    f.movimientos
        .create(input("Cable", "1000.0000", "1.0000", categoria))
        .await
        .unwrap();

    let result = f
        .movimientos
        .list(query(MovimientoFiltroDto::default()))
        .await
        .unwrap();
    let item = &result.page.items[0];

    assert_eq!(item.tipo_movimiento_nombre, "Gasto");
    assert!(!item.es_ingreso);
    assert_eq!(item.categoria_nombre.as_deref(), Some("Materiales"));
    assert_eq!(item.categoria_color.as_deref(), Some("#FFAA00"));
    assert!(!item.bloqueado_por_liquidacion);
}

#[tokio::test]
async fn el_resumen_cubre_todo_el_filtro_y_no_la_pagina() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;

    for i in 0..5 {
        let mut dto = input(&format!("Gasto {i}"), "1000.0000", "1.0000", categoria);
        dto.fecha = fecha(10 + i);
        f.movimientos.create(dto).await.unwrap();
    }
    let mut ingreso = input("Cobro", "20000.0000", "1.0000", categoria);
    ingreso.tipo_movimiento_id = tipos_movimiento::INGRESO;
    f.movimientos.create(ingreso).await.unwrap();

    let result = f
        .movimientos
        .list(ListQuery {
            page_size: 10,
            ..query(MovimientoFiltroDto::default())
        })
        .await
        .unwrap();

    // Six rows on one page of ten, but the point is the totals come from the filter, not the page.
    assert_eq!(result.resumen.cantidad, 6);
    assert_eq!(
        result.resumen.total_ingresos.to_decimal_string(),
        "20000.0000"
    );
    assert_eq!(result.resumen.total_gastos.to_decimal_string(), "5000.0000");
    assert_eq!(result.resumen.balance.to_decimal_string(), "15000.0000");
}

#[tokio::test]
async fn el_resumen_de_una_base_vacia_es_cero_y_no_un_error() {
    let f = fixture().await;
    let resumen = f
        .movimientos
        .resumen(MovimientoFiltroDto::default())
        .await
        .unwrap();

    assert_eq!(resumen.cantidad, 0);
    assert_eq!(resumen.balance, Money::ZERO);
}

#[tokio::test]
async fn el_filtro_de_fechas_incluye_los_dos_extremos_completos() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;

    for day in [10, 15, 20] {
        let mut dto = input(&format!("Dia {day}"), "1000.0000", "1.0000", categoria);
        dto.fecha = fecha(day);
        f.movimientos.create(dto).await.unwrap();
    }

    let result = f
        .movimientos
        .list(query(MovimientoFiltroDto {
            fecha_desde: NaiveDate::from_ymd_opt(2026, 8, 10),
            // A movement booked at 10:00 on the last day must be inside the range: the bound
            // covers the whole civil day, not its first instant.
            fecha_hasta: NaiveDate::from_ymd_opt(2026, 8, 15),
            ..MovimientoFiltroDto::default()
        }))
        .await
        .unwrap();

    assert_eq!(result.page.total_count, 2);
    assert_eq!(result.resumen.cantidad, 2);
}

#[tokio::test]
async fn el_filtro_de_concepto_no_distingue_mayusculas() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    f.movimientos
        .create(input("Cable Subterráneo", "1000.0000", "1.0000", categoria))
        .await
        .unwrap();

    let result = f
        .movimientos
        .list(query(MovimientoFiltroDto {
            concepto: Some("SUBTERRÁNEO".into()),
            ..MovimientoFiltroDto::default()
        }))
        .await
        .unwrap();

    assert_eq!(result.page.total_count, 1);
}

#[tokio::test]
async fn un_concepto_de_solo_espacios_no_filtra_nada() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    f.movimientos
        .create(input("Cable", "1000.0000", "1.0000", categoria))
        .await
        .unwrap();

    let result = f
        .movimientos
        .list(query(MovimientoFiltroDto {
            concepto: Some("   ".into()),
            ..MovimientoFiltroDto::default()
        }))
        .await
        .unwrap();

    assert_eq!(result.page.total_count, 1);
}

#[tokio::test]
async fn el_filtro_de_monto_compara_el_unitario_y_no_el_total() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    // Unit amount 100, total 1000: a filter with a maximum of 500 must keep it.
    f.movimientos
        .create(input("Cable", "100.0000", "10.0000", categoria))
        .await
        .unwrap();

    let result = f
        .movimientos
        .list(query(MovimientoFiltroDto {
            monto_max: Some(Money::parse("500.0000").unwrap()),
            ..MovimientoFiltroDto::default()
        }))
        .await
        .unwrap();

    assert_eq!(result.page.total_count, 1);
    assert_eq!(result.page.items[0].total.to_decimal_string(), "1000.0000");
}
