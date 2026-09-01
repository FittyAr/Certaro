//! End-to-end exercise of `movimientos`: the server-side filter, the summary over the whole
//! filter, the foreign-key checks and the freeze on a settled advance.

use std::sync::Arc;

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use certaro_application::config::AppConfig;
use certaro_application::dtos::categorias::CategoriaInput;
use certaro_application::dtos::common::ListQuery;
use certaro_application::dtos::movimientos::{MovimientoFiltroDto, MovimientoInput};
use certaro_application::ports::repositories::{SortDir, UnitOfWork};
use certaro_application::ports::{ClockPort, IdGeneratorPort, SettingsStore};
use certaro_application::use_cases::categorias::CategoriasService;
use certaro_application::use_cases::movimientos::MovimientosService;
use certaro_application::AppError;
use certaro_domain::clock::FixedClock;
use certaro_domain::constants::tipos_movimiento;
use certaro_domain::ids::UuidV7Generator;
use certaro_domain::{Decimal4, Moneda, Money};
use certaro_infrastructure::config::FileSettingsStore;
use certaro_infrastructure::persistence::DbHandle;
use certaro_infrastructure::persistence::{open_in_memory, SeaOrmUnitOfWork};
use pretty_assertions::assert_eq;
use sea_orm::{ConnectionTrait, DatabaseConnection};
use uuid::Uuid;

struct Fixture {
    movimientos: MovimientosService,
    categorias: CategoriasService,
    db: DatabaseConnection,
}

async fn fixture() -> Fixture {
    let db = open_in_memory().await.unwrap();
    let uow: Arc<dyn UnitOfWork> = Arc::new(SeaOrmUnitOfWork::new(DbHandle::new(db.clone())));
    let clock: Arc<dyn ClockPort> = Arc::new(FixedClock(
        DateTime::parse_from_rfc3339("2026-08-29T12:00:00Z")
            .unwrap()
            .into(),
    ));
    let ids: Arc<dyn IdGeneratorPort> = Arc::new(UuidV7Generator);
    let settings: Arc<dyn SettingsStore> = Arc::new(FileSettingsStore::new(
        std::env::temp_dir().join("eo-test-config.json"),
        AppConfig::default(),
    ));

    Fixture {
        movimientos: MovimientosService::new(
            Arc::clone(&uow),
            Arc::clone(&clock),
            Arc::clone(&ids),
            settings,
        ),
        categorias: CategoriasService::new(uow, clock, ids),
        db,
    }
}

impl Fixture {
    async fn categoria(&self, nombre: &str) -> Uuid {
        self.categorias
            .create(CategoriaInput {
                nombre: nombre.to_owned(),
                descripcion: None,
                color_hex: Some("#FFAA00".into()),
                icono: None,
                categoria_padre_id: None,
            })
            .await
            .unwrap()
            .id
    }
}

fn fecha(day: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, day, 10, 0, 0).unwrap()
}

fn input(concepto: &str, monto: &str, cantidad: &str, categoria: Uuid) -> MovimientoInput {
    MovimientoInput {
        fecha: fecha(15),
        concepto: concepto.to_owned(),
        monto: Money::parse(monto).unwrap(),
        cantidad: Decimal4::parse(cantidad).unwrap(),
        tipo_movimiento_id: tipos_movimiento::GASTO,
        moneda: Moneda::Ars,
        cotizacion_aplicada: None,
        tipo_concepto_pago_id: None,
        categoria_id: Some(categoria),
        cliente_id: None,
        trabajo_id: None,
        empleado_id: None,
        factura_id: None,
    }
}

fn query(filtro: MovimientoFiltroDto) -> ListQuery<MovimientoFiltroDto> {
    ListQuery {
        filtro,
        page: 1,
        page_size: 30,
        sort_by: None,
        sort_dir: SortDir::Asc,
    }
}

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
