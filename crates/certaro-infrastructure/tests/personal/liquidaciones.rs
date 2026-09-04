use pretty_assertions::assert_eq;
use super::common::*;
use certaro_application::dtos::liquidaciones::*;
use certaro_application::AppError;
use certaro_domain::{Decimal4, Money, TipoJornada};

#[tokio::test]
async fn la_sugerencia_toma_los_dias_de_la_asistencia_y_los_adelantos_del_periodo() {
    let f = fixture().await;
    let id = f.empleado("Juan", "10000.0000").await;
    f.marcar(id, dia(6, 16), Some(TipoJornada::Completa)).await;
    f.marcar(id, dia(6, 17), Some(TipoJornada::Completa)).await;
    f.marcar(id, dia(6, 18), Some(TipoJornada::Media)).await;
    f.adelanto(id, dia(6, 17), "5000.0000").await;

    let sugerencias = f
        .liquidaciones
        .suggest(LiquidacionSugerenciaQuery {
            empleado_ids: vec![id],
            desde: dia(6, 15),
            hasta: dia(6, 19),
            dias_manuales: Default::default(),
        })
        .await
        .unwrap();

    let s = &sugerencias[0];
    assert_eq!(s.origen, OrigenLiquidacion::Asistencia);
    assert_eq!(s.dias_trabajados, Decimal4::parse("2.5").unwrap());
    assert_eq!(s.total_bruto, Money::parse("25000.0000").unwrap());
    assert_eq!(s.total_adelantos, Money::parse("5000.0000").unwrap());
    assert_eq!(s.total_neto, Money::parse("20000.0000").unwrap());
    assert_eq!(s.adelantos.len(), 1);
    assert!(!s.adelantos[0].ya_descontado);
}

#[tokio::test]
async fn un_adelanto_ya_liquidado_se_muestra_tachado_y_no_se_vuelve_a_descontar() {
    let f = fixture().await;
    let id = f.empleado("Juan", "10000.0000").await;
    let movimiento_id = f.adelanto(id, dia(6, 3), "5000.0000").await;

    let mut input = liquidacion_input(id, dia(6, 1), dia(6, 15), "10.0000");
    input.total_adelantos = Money::parse("5000.0000").unwrap();
    input.adelantos = vec![LiquidacionAdelantoInput {
        movimiento_id,
        fecha: dia(6, 3),
        concepto: "Adelanto".to_owned(),
        monto: Money::parse("5000.0000").unwrap(),
    }];
    f.liquidaciones.create(input).await.unwrap();

    let sugerencias = f
        .liquidaciones
        .suggest(LiquidacionSugerenciaQuery {
            empleado_ids: vec![id],
            desde: dia(6, 1),
            hasta: dia(6, 15),
            dias_manuales: Default::default(),
        })
        .await
        .unwrap();

    let adelanto = &sugerencias[0].adelantos[0];
    assert!(adelanto.ya_descontado);
    assert!(!adelanto.incluir);
    assert_eq!(sugerencias[0].total_adelantos, Money::ZERO);
}

#[tokio::test]
async fn dos_liquidaciones_del_mismo_periodo_no_conviven() {
    let f = fixture().await;
    let id = f.empleado("Juan", "10000.0000").await;
    f.liquidaciones
        .create(liquidacion_input(id, dia(6, 1), dia(6, 15), "10.0000"))
        .await
        .unwrap();

    let error = f
        .liquidaciones
        .create(liquidacion_input(id, dia(6, 10), dia(6, 20), "5.0000"))
        .await
        .unwrap_err();

    assert!(matches!(error, AppError::Conflict { .. }));
    let total = f
        .liquidaciones
        .list(query(LiquidacionFiltroDto::default()))
        .await
        .unwrap()
        .total_count;
    assert_eq!(total, 1);
}

#[tokio::test]
async fn el_lote_es_atomico_y_dice_que_empleado_falla() {
    let f = fixture().await;
    let uno = f.empleado("Uno", "10000.0000").await;
    let dos = f.empleado("Dos", "10000.0000").await;
    f.liquidaciones
        .create(liquidacion_input(dos, dia(6, 1), dia(6, 15), "10.0000"))
        .await
        .unwrap();

    let error = f
        .liquidaciones
        .create_batch(LiquidacionBatchInput {
            dtos: vec![
                liquidacion_input(uno, dia(6, 16), dia(6, 30), "10.0000"),
                liquidacion_input(dos, dia(6, 10), dia(6, 20), "10.0000"),
            ],
        })
        .await
        .unwrap_err();

    assert_eq!(
        error.params().get("empleado").map(String::as_str),
        Some("Dos")
    );
    // The first settlement of the batch is not saved either: the only surviving one is the
    // pre-existing one.
    let listado = f
        .liquidaciones
        .list(query(LiquidacionFiltroDto::default()))
        .await
        .unwrap();
    assert_eq!(listado.total_count, 1);
    assert_eq!(listado.items[0].empleado_id, dos);
}

#[tokio::test]
async fn borrar_una_liquidacion_libera_sus_adelantos() {
    let f = fixture().await;
    let id = f.empleado("Juan", "10000.0000").await;
    let movimiento_id = f.adelanto(id, dia(6, 3), "5000.0000").await;

    let mut input = liquidacion_input(id, dia(6, 1), dia(6, 15), "10.0000");
    input.total_adelantos = Money::parse("5000.0000").unwrap();
    input.adelantos = vec![LiquidacionAdelantoInput {
        movimiento_id,
        fecha: dia(6, 3),
        concepto: "Adelanto".to_owned(),
        monto: Money::parse("5000.0000").unwrap(),
    }];
    let creada = f.liquidaciones.create(input.clone()).await.unwrap();
    assert_eq!(creada.adelantos.len(), 1);

    f.liquidaciones
        .delete(creada.id, &creada.audit.row_version)
        .await
        .unwrap();

    // The same period and the same advance can be settled again.
    let nueva = f.liquidaciones.create(input).await.unwrap();
    assert_eq!(nueva.adelantos.len(), 1);
    assert_eq!(nueva.total_adelantos, Money::parse("5000.0000").unwrap());
}

#[tokio::test]
async fn el_detalle_congela_las_reglas_y_deriva_el_neto() {
    let f = fixture().await;
    let id = f.empleado("Juan", "10000.0000").await;
    let mut input = liquidacion_input(id, dia(6, 1), dia(6, 15), "10.0000");
    input.incluir_feriados = true;
    input.multiplicador_feriado = Decimal4::parse("2.0").unwrap();

    let creada = f.liquidaciones.create(input).await.unwrap();

    assert!(creada.incluir_feriados);
    assert_eq!(
        creada.multiplicador_feriado,
        Decimal4::parse("2.0").unwrap()
    );
    assert_eq!(creada.total_neto, Money::parse("100000.0000").unwrap());
    assert!(creada.admite_cambio_de_importes);

    // Editing the notes must not disturb the frozen amounts.
    let editada = f
        .liquidaciones
        .update(
            creada.id,
            LiquidacionUpdateInput {
                dias_trabajados: creada.dias_trabajados,
                tarifa_aplicada: creada.tarifa_aplicada,
                total_bruto: creada.total_bruto,
                total_adelantos: creada.total_adelantos,
                observaciones: Some("Revisada".to_owned()),
            },
            &creada.audit.row_version,
        )
        .await
        .unwrap();

    assert_eq!(editada.observaciones.as_deref(), Some("Revisada"));
    assert_eq!(editada.total_neto, creada.total_neto);
}
