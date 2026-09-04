use pretty_assertions::assert_eq;
use super::common::*;
use certaro_application::dtos::asistencias::*;
use certaro_domain::TipoJornada;
use chrono::NaiveDate;
use uuid::Uuid;

#[tokio::test]
async fn la_grilla_devuelve_una_celda_por_dia_y_marca_los_feriados() {
    let f = fixture().await;
    let id = f.empleado("Juan", "10000.0000").await;
    f.feriados.sync(vec![2026]).await.unwrap();
    f.marcar(id, dia(6, 16), Some(TipoJornada::Completa)).await;

    let grilla = f
        .asistencias
        .grilla(AsistenciaGrillaQuery {
            desde: dia(6, 15),
            hasta: dia(6, 21),
            empleado_ids: vec![],
        })
        .await
        .unwrap();

    assert_eq!(grilla.dias.len(), 7);
    assert_eq!(grilla.filas.len(), 1);
    // Same length as `dias`, so the frontend can render by index.
    assert_eq!(grilla.filas[0].celdas.len(), 7);
    assert!(grilla.dias[0].es_feriado);
    assert_eq!(grilla.dias[0].feriado_nombre.as_deref(), Some("Prueba"));
    assert!(grilla.dias[6].es_fin_de_semana);
    assert_eq!(
        grilla.filas[0].celdas[1].tipo_jornada,
        Some(TipoJornada::Completa)
    );
    assert_eq!(grilla.filas[0].celdas[0].tipo_jornada, None);
    assert_eq!(grilla.filas[0].resumen.completas, 1);
}

#[tokio::test]
async fn el_ciclo_de_click_recorre_los_tipos_y_vuelve_al_vacio() {
    let f = fixture().await;
    let id = f.empleado("Juan", "10000.0000").await;
    let fecha = dia(6, 16);

    let mut actual: Option<TipoJornada> = None;
    let mut recorrido = Vec::new();
    for _ in 0..6 {
        let siguiente = TipoJornada::siguiente(actual);
        f.marcar(id, fecha, siguiente).await;
        assert_eq!(celda_de(&f, id, fecha).await, siguiente);
        recorrido.push(siguiente);
        actual = siguiente;
    }

    // Clearing the cell has to be reachable, otherwise a mistaken click could never be undone.
    assert_eq!(recorrido.last().copied(), Some(None));
    assert_eq!(celda_de(&f, id, fecha).await, None);

    // And a cleared cell can be marked again: the soft-deleted row is reused instead of colliding
    // with the unique key.
    f.marcar(id, fecha, Some(TipoJornada::Media)).await;
    assert_eq!(celda_de(&f, id, fecha).await, Some(TipoJornada::Media));
}

async fn celda_de(f: &Fixture, empleado_id: Uuid, fecha: NaiveDate) -> Option<TipoJornada> {
    f.asistencias
        .grilla(AsistenciaGrillaQuery {
            desde: fecha,
            hasta: fecha,
            empleado_ids: vec![empleado_id],
        })
        .await
        .unwrap()
        .filas[0]
        .celdas[0]
        .tipo_jornada
}

#[tokio::test]
async fn la_carga_masiva_saltea_fines_de_semana_y_feriados() {
    let f = fixture().await;
    let id = f.empleado("Juan", "10000.0000").await;
    f.feriados.sync(vec![2026]).await.unwrap();

    let celdas = f
        .asistencias
        .upsert_rango(AsistenciaRangoInput {
            empleado_id: id,
            // Monday the 15th is a holiday, the 20th and 21st are the weekend.
            desde: dia(6, 15),
            hasta: dia(6, 21),
            tipo_jornada: TipoJornada::Completa,
            solo_dias_habiles: true,
            trabajo_id: None,
        })
        .await
        .unwrap();

    assert_eq!(celdas.len(), 4);
    assert_eq!(
        celdas.iter().map(|c| c.fecha).collect::<Vec<_>>(),
        vec![dia(6, 16), dia(6, 17), dia(6, 18), dia(6, 19)]
    );
}
