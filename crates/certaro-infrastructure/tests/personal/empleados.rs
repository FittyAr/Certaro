use pretty_assertions::assert_eq;
use super::common::*;
use certaro_application::dtos::empleados::*;
use certaro_application::AppError;
use certaro_domain::Money;

#[tokio::test]
async fn un_empleado_creado_se_lee_de_vuelta_y_ofrece_su_tarifa_sugerida() {
    let f = fixture().await;
    let mut input = empleado_input("Juan Pérez", "10000.0000");
    input.sueldo_base = Money::parse("300000.0000").unwrap();

    let creado = f.empleados.create(input).await.unwrap();

    assert_eq!(creado.nombre, "Juan Pérez");
    assert_eq!(creado.tarifa_diaria, Money::parse("10000.0000").unwrap());
    // Fortnightly: the salary spread over the 15 days of the period.
    assert_eq!(
        creado.tarifa_diaria_sugerida,
        Money::parse("20000.0000").unwrap()
    );
    assert!(creado.puede_eliminarse);
}

#[tokio::test]
async fn el_listado_filtra_por_activo_y_ofrece_los_cargos_en_uso() {
    let f = fixture().await;
    f.empleado("Activo", "1000.0000").await;
    let baja = f
        .empleados
        .create(empleado_input("Inactivo", "1000.0000"))
        .await
        .unwrap();
    let mut input = empleado_input("Inactivo", "1000.0000");
    input.activo = false;
    f.empleados
        .update(baja.id, input, &baja.audit.row_version)
        .await
        .unwrap();

    let activos = f
        .empleados
        .list(query(EmpleadoFiltroDto::default()))
        .await
        .unwrap();
    assert_eq!(activos.total_count, 1);
    assert_eq!(activos.items[0].nombre, "Activo");

    let cargos = f.empleados.cargos().await.unwrap();
    assert_eq!(cargos, vec!["Oficial".to_owned()]);
}

#[tokio::test]
async fn un_empleado_con_liquidaciones_no_se_borra() {
    let f = fixture().await;
    let id = f.empleado("Con historia", "10000.0000").await;
    f.liquidaciones
        .create(liquidacion_input(id, dia(6, 1), dia(6, 15), "10.0000"))
        .await
        .unwrap();

    let detalle = f.empleados.get(id).await.unwrap();
    assert!(!detalle.puede_eliminarse);

    let error = f
        .empleados
        .delete(id, &detalle.audit.row_version)
        .await
        .unwrap_err();
    assert!(matches!(error, AppError::DependencyInUse { .. }));
}
