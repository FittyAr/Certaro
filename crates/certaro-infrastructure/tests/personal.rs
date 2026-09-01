//! End-to-end exercise of the personnel modules against a real database: employees, the attendance
//! grid with its click cycle, the holiday table and the settlement with its frozen advances.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use certaro_application::config::AppConfig;
use certaro_application::dtos::asistencias::{
    AsistenciaGrillaQuery, AsistenciaRangoInput, AsistenciaUpsertInput,
};
use certaro_application::dtos::categorias::CategoriaInput;
use certaro_application::dtos::common::ListQuery;
use certaro_application::dtos::empleados::{EmpleadoFiltroDto, EmpleadoInput};
use certaro_application::dtos::feriados::{FeriadoInput, FeriadoSyncResult};
use certaro_application::dtos::liquidaciones::{
    LiquidacionAdelantoInput, LiquidacionBatchInput, LiquidacionFiltroDto, LiquidacionInput,
    LiquidacionSugerenciaQuery, LiquidacionUpdateInput, OrigenLiquidacion,
};
use certaro_application::dtos::movimientos::MovimientoInput;
use certaro_application::ports::holidays::HolidayProvider;
use certaro_application::ports::repositories::{SortDir, UnitOfWork};
use certaro_application::ports::{ClockPort, IdGeneratorPort, SettingsStore};
use certaro_application::use_cases::asistencias::AsistenciasService;
use certaro_application::use_cases::categorias::CategoriasService;
use certaro_application::use_cases::empleados::EmpleadosService;
use certaro_application::use_cases::feriados::FeriadosService;
use certaro_application::use_cases::liquidaciones::LiquidacionesService;
use certaro_application::use_cases::movimientos::MovimientosService;
use certaro_application::{AppError, AppResult};
use certaro_domain::clock::FixedClock;
use certaro_domain::constants::tipos_movimiento;
use certaro_domain::entities::{Feriado, OrigenFeriado};
use certaro_domain::ids::UuidV7Generator;
use certaro_domain::{Decimal4, FrecuenciaPago, Moneda, Money, TipoJornada};
use certaro_infrastructure::config::FileSettingsStore;
use certaro_infrastructure::persistence::DbHandle;
use certaro_infrastructure::persistence::{open_in_memory, SeaOrmUnitOfWork};
use pretty_assertions::assert_eq;
use uuid::Uuid;

/// A provider with a fixed calendar, so the tests never touch the network. Returning an error is
/// what proves the degradation path.
struct FakeHolidays {
    feriados: Vec<(NaiveDate, &'static str)>,
    falla: bool,
}

#[async_trait]
impl HolidayProvider for FakeHolidays {
    async fn fetch(&self, anio: i32) -> AppResult<Vec<Feriado>> {
        if self.falla {
            return Err(AppError::ExternalUnavailable { service: "test" });
        }
        let now = ahora();
        Ok(self
            .feriados
            .iter()
            .filter(|(fecha, _)| fecha.format("%Y").to_string() == format!("{anio:04}"))
            .map(|(fecha, nombre)| Feriado {
                fecha: *fecha,
                nombre: (*nombre).to_owned(),
                tipo: Some("inamovible".to_owned()),
                origen: OrigenFeriado::Api,
                created_at: now,
                updated_at: None,
            })
            .collect())
    }
}

struct Fixture {
    empleados: EmpleadosService,
    asistencias: AsistenciasService,
    liquidaciones: LiquidacionesService,
    feriados: FeriadosService,
    movimientos: MovimientosService,
    categorias: CategoriasService,
}

fn ahora() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 6, 30, 12, 0, 0)
        .single()
        .unwrap_or_else(Utc::now)
}

fn query<F>(filtro: F) -> ListQuery<F> {
    ListQuery {
        filtro,
        page: 1,
        page_size: 30,
        sort_by: None,
        sort_dir: SortDir::Asc,
    }
}

fn dia(mes: u32, dia: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, mes, dia).unwrap()
}

async fn fixture_con(provider: FakeHolidays) -> Fixture {
    let db = open_in_memory().await.unwrap();
    let uow: Arc<dyn UnitOfWork> = Arc::new(SeaOrmUnitOfWork::new(DbHandle::new(db)));
    let clock: Arc<dyn ClockPort> = Arc::new(FixedClock(ahora()));
    let ids: Arc<dyn IdGeneratorPort> = Arc::new(UuidV7Generator);
    let settings: Arc<dyn SettingsStore> = Arc::new(FileSettingsStore::new(
        std::env::temp_dir().join("eo-test-personal.json"),
        AppConfig::default(),
    ));
    let holidays: Arc<dyn HolidayProvider> = Arc::new(provider);

    Fixture {
        empleados: EmpleadosService::new(Arc::clone(&uow), Arc::clone(&clock), Arc::clone(&ids)),
        asistencias: AsistenciasService::new(
            Arc::clone(&uow),
            Arc::clone(&clock),
            Arc::clone(&ids),
            Arc::clone(&settings),
        ),
        liquidaciones: LiquidacionesService::new(
            Arc::clone(&uow),
            Arc::clone(&clock),
            Arc::clone(&ids),
        ),
        feriados: FeriadosService::new(
            Arc::clone(&uow),
            Arc::clone(&clock),
            holidays,
            Arc::clone(&settings),
        ),
        categorias: CategoriasService::new(Arc::clone(&uow), Arc::clone(&clock), Arc::clone(&ids)),
        movimientos: MovimientosService::new(uow, clock, ids, settings),
    }
}

async fn fixture() -> Fixture {
    fixture_con(FakeHolidays {
        feriados: vec![(dia(6, 15), "Prueba")],
        falla: false,
    })
    .await
}

fn empleado_input(nombre: &str, tarifa: &str) -> EmpleadoInput {
    EmpleadoInput {
        nombre: nombre.to_owned(),
        dni: None,
        cargo: Some("Oficial".to_owned()),
        sueldo_base: Money::ZERO,
        pago_frecuencia: FrecuenciaPago::Quincenal,
        tarifa_diaria: Money::parse(tarifa).unwrap(),
        multiplicador_sabado: Decimal4::parse("1.5").unwrap(),
        multiplicador_domingo: Decimal4::parse("2.0").unwrap(),
        multiplicador_feriado: Decimal4::parse("2.0").unwrap(),
        email: None,
        telefono: None,
        fecha_ingreso: dia(1, 5),
        fecha_egreso: None,
        activo: true,
    }
}

impl Fixture {
    async fn empleado(&self, nombre: &str, tarifa: &str) -> Uuid {
        self.empleados
            .create(empleado_input(nombre, tarifa))
            .await
            .unwrap()
            .id
    }

    async fn marcar(&self, empleado_id: Uuid, fecha: NaiveDate, tipo: Option<TipoJornada>) {
        self.asistencias
            .upsert(AsistenciaUpsertInput {
                empleado_id,
                fecha,
                tipo_jornada: tipo,
                trabajo_id: None,
                observaciones: None,
            })
            .await
            .unwrap();
    }

    /// An advance, as the movements module records it: the payroll finds it by the seeded type.
    async fn adelanto(&self, empleado_id: Uuid, fecha: NaiveDate, monto: &str) -> Uuid {
        let categoria = self
            .categorias
            .create(CategoriaInput {
                nombre: format!("Sueldos {fecha}"),
                descripcion: None,
                color_hex: None,
                icono: None,
                categoria_padre_id: None,
            })
            .await
            .unwrap()
            .id;

        self.movimientos
            .create(MovimientoInput {
                fecha: fecha
                    .and_hms_opt(10, 0, 0)
                    .map(|d| Utc.from_utc_datetime(&d))
                    .unwrap(),
                concepto: "Adelanto".to_owned(),
                monto: Money::parse(monto).unwrap(),
                cantidad: Decimal4::ONE,
                tipo_movimiento_id: tipos_movimiento::ADELANTO,
                moneda: Moneda::Ars,
                cotizacion_aplicada: None,
                tipo_concepto_pago_id: None,
                categoria_id: Some(categoria),
                cliente_id: None,
                trabajo_id: None,
                empleado_id: Some(empleado_id),
                factura_id: None,
            })
            .await
            .unwrap()
            .item
            .id
    }
}

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

fn liquidacion_input(
    empleado_id: Uuid,
    desde: NaiveDate,
    hasta: NaiveDate,
    dias: &str,
) -> LiquidacionInput {
    let dias_trabajados = Decimal4::parse(dias).unwrap();
    let tarifa = Money::parse("10000.0000").unwrap();
    LiquidacionInput {
        empleado_id,
        fecha_inicio: desde,
        fecha_fin: hasta,
        dias_trabajados,
        tarifa_aplicada: tarifa,
        incluir_sabados: false,
        incluir_domingos: false,
        incluir_feriados: false,
        multiplicador_sabado: Decimal4::ONE,
        multiplicador_domingo: Decimal4::ONE,
        multiplicador_feriado: Decimal4::ONE,
        total_bruto: tarifa.checked_mul(dias_trabajados).unwrap(),
        total_adelantos: Money::ZERO,
        observaciones: None,
        adelantos: vec![],
    }
}

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
