use pretty_assertions::assert_eq;
use super::common::*;
use certaro_application::dtos::common::ListQuery;
use certaro_application::dtos::movimientos::MovimientoFiltroDto;
use certaro_application::AppError;
use certaro_domain::constants::tipos_movimiento;
use certaro_domain::{Moneda, Money};
use uuid::Uuid;

#[tokio::test]
async fn el_orden_por_defecto_es_la_fecha_descendente() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    for day in [10, 20, 15] {
        let mut dto = input(&format!("Dia {day}"), "1000.0000", "1.0000", categoria);
        dto.fecha = fecha(day);
        f.movimientos.create(dto).await.unwrap();
    }

    let result = f
        .movimientos
        .list(query(MovimientoFiltroDto::default()))
        .await
        .unwrap();
    let conceptos: Vec<&str> = result
        .page
        .items
        .iter()
        .map(|i| i.concepto.as_str())
        .collect();

    assert_eq!(conceptos, ["Dia 20", "Dia 15", "Dia 10"]);
}

#[tokio::test]
async fn un_campo_de_orden_no_permitido_se_rechaza() {
    let f = fixture().await;
    let error = f
        .movimientos
        .list(ListQuery {
            sort_by: Some("rowVersion".into()),
            ..query(MovimientoFiltroDto::default())
        })
        .await
        .unwrap_err();

    assert_eq!(
        error.fields().first().map(|f| f.message_key.as_str()),
        Some("Validation.Common.SortByNotAllowed")
    );
}

#[tokio::test]
async fn una_categoria_inexistente_marca_el_campo_y_no_revienta_la_foreign_key() {
    let f = fixture().await;
    let error = f
        .movimientos
        .create(input("Cable", "1000.0000", "1.0000", Uuid::now_v7()))
        .await
        .unwrap_err();

    let field = error.fields().first().unwrap();
    assert_eq!(field.field, "categoriaId");
    assert_eq!(field.message_key, "Validation.Common.ReferenciaInexistente");
}

#[tokio::test]
async fn en_pesos_no_se_guarda_cotizacion() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    let creado = f
        .movimientos
        .create(input("Cable", "1000.0000", "1.0000", categoria))
        .await
        .unwrap();

    assert_eq!(creado.item.cotizacion_aplicada, None);
}

#[tokio::test]
async fn en_dolares_la_cotizacion_viaja_de_ida_y_de_vuelta() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    let mut dto = input("Herramienta", "500.0000", "1.0000", categoria);
    dto.moneda = Moneda::Usd;
    dto.cotizacion_aplicada = Some(Money::parse("1350.5000").unwrap());

    let creado = f.movimientos.create(dto).await.unwrap();
    let leido = f.movimientos.get(creado.item.id).await.unwrap();

    assert_eq!(leido.item.moneda, Moneda::Usd);
    assert_eq!(
        leido.item.cotizacion_aplicada.unwrap().to_decimal_string(),
        "1350.5000"
    );
}

#[tokio::test]
async fn una_version_vieja_pierde_la_carrera() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    let creado = f
        .movimientos
        .create(input("Cable", "1000.0000", "1.0000", categoria))
        .await
        .unwrap();
    let vieja = creado.item.row_version.clone();

    f.movimientos
        .update(
            creado.item.id,
            input("Cable nuevo", "1200.0000", "1.0000", categoria),
            &vieja,
        )
        .await
        .unwrap();

    let error = f
        .movimientos
        .update(
            creado.item.id,
            input("Otro", "1300.0000", "1.0000", categoria),
            &vieja,
        )
        .await
        .unwrap_err();

    assert!(matches!(error, AppError::Concurrency { .. }));
}

#[tokio::test]
async fn un_movimiento_borrado_desaparece_del_listado_y_del_resumen() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    let creado = f
        .movimientos
        .create(input("Cable", "1000.0000", "1.0000", categoria))
        .await
        .unwrap();

    f.movimientos
        .delete(creado.item.id, &creado.item.row_version)
        .await
        .unwrap();

    let result = f
        .movimientos
        .list(query(MovimientoFiltroDto::default()))
        .await
        .unwrap();
    assert_eq!(result.page.total_count, 0);
    assert_eq!(result.resumen.cantidad, 0);
    assert!(matches!(
        f.movimientos.get(creado.item.id).await.unwrap_err(),
        AppError::NotFound { .. }
    ));
}

#[tokio::test]
async fn un_adelanto_ya_liquidado_queda_congelado() {
    let f = fixture().await;
    let categoria = f.categoria("Sueldos").await;
    let empleado = Uuid::now_v7();
    f.db.execute_unprepared(&format!(
        "INSERT INTO empleados (id, nombre, tarifa_diaria, fecha_ingreso, created_at, \
         row_version, is_deleted) VALUES \
         ('{empleado}', 'Juan', 1000000, '2026-01-01', '2026-08-01T00:00:00.000Z', \
         x'0000000000000001', 0)"
    ))
    .await
    .unwrap();

    let mut dto = input("Adelanto quincena", "50000.0000", "1.0000", categoria);
    dto.tipo_movimiento_id = tipos_movimiento::ADELANTO;
    dto.empleado_id = Some(empleado);
    let adelanto = f.movimientos.create(dto).await.unwrap();

    let liquidacion = Uuid::now_v7();
    f.db.execute_unprepared(&format!(
        "INSERT INTO liquidaciones (id, empleado_id, fecha_inicio, fecha_fin, dias_trabajados, \
         tarifa_aplicada, total_bruto, total_adelantos, created_at, row_version, is_deleted) \
         VALUES ('{liquidacion}', '{empleado}', '2026-08-01', '2026-08-15', 10, 1000000, \
         10000000, 500000, '2026-08-16T00:00:00.000Z', x'0000000000000001', 0)"
    ))
    .await
    .unwrap();
    let vinculo = Uuid::now_v7();
    f.db.execute_unprepared(&format!(
        "INSERT INTO liquidacion_adelantos (id, liquidacion_id, movimiento_id, monto, fecha, \
         concepto, created_at, row_version, is_deleted) VALUES \
         ('{vinculo}', '{liquidacion}', '{}', 500000, '2026-08-15', 'Adelanto quincena', \
         '2026-08-16T00:00:00.000Z', x'0000000000000001', 0)",
        adelanto.item.id
    ))
    .await
    .unwrap();

    // Editing it would change a settlement that was already signed off.
    let mut edicion = input("Adelanto corregido", "60000.0000", "1.0000", categoria);
    edicion.tipo_movimiento_id = tipos_movimiento::ADELANTO;
    edicion.empleado_id = Some(empleado);
    let error = f
        .movimientos
        .update(adelanto.item.id, edicion, &adelanto.item.row_version)
        .await
        .unwrap_err();
    assert!(
        matches!(error, AppError::DependencyInUse { code, .. } if code == "MOVIMIENTO_ADELANTO_LIQUIDADO")
    );

    let error = f
        .movimientos
        .delete(adelanto.item.id, &adelanto.item.row_version)
        .await
        .unwrap_err();
    assert!(
        matches!(error, AppError::DependencyInUse { code, .. } if code == "MOVIMIENTO_ADELANTO_LIQUIDADO")
    );

    let result = f
        .movimientos
        .list(query(MovimientoFiltroDto::default()))
        .await
        .unwrap();
    let item = result
        .page
        .items
        .iter()
        .find(|i| i.id == adelanto.item.id)
        .unwrap();
    assert!(item.bloqueado_por_liquidacion);
}

#[tokio::test]
async fn la_paginacion_no_repite_ni_saltea_filas_del_mismo_instante() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    // Same instant on every row: without the identifier as tie-breaker the order is undefined and
    // the second page can repeat what the first already showed.
    for i in 0..20 {
        f.movimientos
            .create(input(
                &format!("Fila {i:02}"),
                "1000.0000",
                "1.0000",
                categoria,
            ))
            .await
            .unwrap();
    }

    let mut vistos = Vec::new();
    for page in 1..=2 {
        let result = f
            .movimientos
            .list(ListQuery {
                page,
                page_size: 10,
                ..query(MovimientoFiltroDto::default())
            })
            .await
            .unwrap();
        vistos.extend(result.page.items.into_iter().map(|i| i.id));
    }

    let unicos: std::collections::HashSet<_> = vistos.iter().collect();
    assert_eq!(vistos.len(), 20);
    assert_eq!(unicos.len(), 20);
}
