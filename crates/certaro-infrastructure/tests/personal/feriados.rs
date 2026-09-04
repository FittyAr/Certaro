use pretty_assertions::assert_eq;
use super::common::*;
use certaro_application::dtos::feriados::*;
use certaro_domain::entities::OrigenFeriado;

#[tokio::test]
async fn los_feriados_manuales_ganan_sobre_la_api() {
    let f = fixture().await;
    f.feriados
        .add(FeriadoInput {
            fecha: dia(6, 15),
            nombre: "Cargado a mano".to_owned(),
        })
        .await
        .unwrap();

    let resultado = f.feriados.sync(vec![2026]).await.unwrap();

    assert_eq!(
        resultado,
        FeriadoSyncResult {
            agregados: 0,
            total: 1,
            anios_con_error: 0
        }
    );
    let lista = f.feriados.list(2026).await.unwrap();
    assert_eq!(lista.len(), 1);
    assert_eq!(lista[0].nombre, "Cargado a mano");
    assert_eq!(lista[0].origen, OrigenFeriado::Manual);
}

#[tokio::test]
async fn un_error_de_la_api_no_borra_los_feriados_existentes() {
    let f = fixture_con(FakeHolidays {
        feriados: vec![],
        falla: true,
    })
    .await;
    f.feriados
        .add(FeriadoInput {
            fecha: dia(6, 15),
            nombre: "Existente".to_owned(),
        })
        .await
        .unwrap();

    let resultado = f.feriados.sync(vec![2026]).await.unwrap();

    assert_eq!(resultado.anios_con_error, 1);
    assert_eq!(f.feriados.list(2026).await.unwrap().len(), 1);
}

#[tokio::test]
async fn borrar_un_feriado_lo_saca_del_calendario_de_verdad() {
    let f = fixture().await;
    f.feriados.sync(vec![2026]).await.unwrap();

    let restantes = f.feriados.delete(dia(6, 15)).await.unwrap();

    assert!(restantes.is_empty());
    // A real delete, so the sync can bring it back.
    assert_eq!(f.feriados.sync(vec![2026]).await.unwrap().agregados, 1);
}
