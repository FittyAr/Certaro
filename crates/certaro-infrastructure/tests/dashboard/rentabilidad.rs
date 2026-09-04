use pretty_assertions::assert_eq;
use super::common::*;
use certaro_domain::Decimal4;

#[tokio::test]
async fn la_rentabilidad_por_proyecto_imputa_a_traves_del_trabajo() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    let cliente = f.cliente("Cliente").await;
    let proyecto = f.proyecto(1, "Edificio Norte", cliente).await;
    let trabajo = f.trabajo(proyecto, "Tablero").await;

    f.movimiento(
        "Cobro",
        "3000.0000",
        "1.0000",
        true,
        5,
        Some(categoria),
        None,
        Some(trabajo),
    )
    .await;
    f.movimiento(
        "Gasto",
        "2000.0000",
        "1.0000",
        false,
        5,
        Some(categoria),
        None,
        Some(trabajo),
    )
    .await;
    // Without a job the movement is imputed to no site at all.
    f.movimiento(
        "Suelto",
        "5000.0000",
        "1.0000",
        false,
        5,
        Some(categoria),
        None,
        None,
    )
    .await;

    let ranking = f.comercial.rentabilidad_proyectos(None).await.unwrap();

    assert_eq!(ranking.len(), 1);
    let fila = &ranking[0];
    assert_eq!(fila.ingresos.to_decimal_string(), "3000.0000");
    assert_eq!(fila.gastos.to_decimal_string(), "2000.0000");
    assert_eq!(fila.rentabilidad.to_decimal_string(), "1000.0000");
    assert_eq!(fila.margen_porcentaje, Decimal4::parse("33.33").unwrap());
}

#[tokio::test]
async fn un_proyecto_sin_ingresos_da_margen_cero_y_no_divide_por_cero() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    let cliente = f.cliente("Cliente").await;
    let proyecto = f.proyecto(1, "Solo gastos", cliente).await;
    let trabajo = f.trabajo(proyecto, "Zanjeo").await;

    f.movimiento(
        "Gasto",
        "1500.0000",
        "1.0000",
        false,
        5,
        Some(categoria),
        None,
        Some(trabajo),
    )
    .await;

    let ranking = f.comercial.rentabilidad_proyectos(None).await.unwrap();

    assert_eq!(ranking[0].rentabilidad.to_decimal_string(), "-1500.0000");
    assert_eq!(ranking[0].margen_porcentaje, Decimal4::ZERO);
}

#[tokio::test]
async fn la_rentabilidad_por_trabajo_se_puede_filtrar_por_proyecto() {
    let f = fixture().await;
    let categoria = f.categoria("Materiales").await;
    let cliente = f.cliente("Cliente").await;
    let proyecto_a = f.proyecto(1, "Proyecto A", cliente).await;
    let proyecto_b = f.proyecto(2, "Proyecto B", cliente).await;
    let trabajo_a = f.trabajo(proyecto_a, "Tablero A").await;
    let trabajo_b = f.trabajo(proyecto_b, "Tablero B").await;

    f.movimiento(
        "Cobro A",
        "1000.0000",
        "1.0000",
        true,
        5,
        Some(categoria),
        None,
        Some(trabajo_a),
    )
    .await;
    f.movimiento(
        "Cobro B",
        "2000.0000",
        "1.0000",
        true,
        5,
        Some(categoria),
        None,
        Some(trabajo_b),
    )
    .await;

    let todos = f.comercial.rentabilidad_trabajos(None, None).await.unwrap();
    assert_eq!(todos.len(), 2);
    assert_eq!(todos[0].nombre, "Tablero B");
    assert_eq!(todos[0].contexto, "Proyecto B");

    let solo_a = f
        .comercial
        .rentabilidad_trabajos(Some(proyecto_a), None)
        .await
        .unwrap();
    assert_eq!(solo_a.len(), 1);
    assert_eq!(solo_a[0].id, trabajo_a);
}
