//! End-to-end exercise of the reference aggregate: use case, unit of work, repository and SQLite.
//!
//! Everything runs against a real in-memory database rather than a mock, because the behaviour
//! worth testing here — the soft-delete filter, the optimistic version check, the correlated
//! count — lives in the SQL and a mock would only confirm the mock.

use std::sync::Arc;

use certaro_application::dtos::common::ListQuery;
use certaro_application::dtos::tipos_movimiento::{TipoMovimientoFiltroDto, TipoMovimientoInput};
use certaro_application::ports::repositories::{SortDir, UnitOfWork};
use certaro_application::ports::{ClockPort, IdGeneratorPort};
use certaro_application::use_cases::tipos_movimiento::TiposMovimientoService;
use certaro_application::AppError;
use certaro_domain::clock::FixedClock;
use certaro_domain::ids::UuidV7Generator;
use certaro_infrastructure::persistence::DbHandle;
use certaro_infrastructure::persistence::{open_in_memory, SeaOrmUnitOfWork};
use pretty_assertions::assert_eq;
use sea_orm::{ConnectionTrait, DatabaseConnection};

const GASTO: &str = "00000000-0000-0000-0000-000000000002";

async fn service() -> (TiposMovimientoService, DatabaseConnection) {
    let db = open_in_memory().await.unwrap();
    let uow: Arc<dyn UnitOfWork> = Arc::new(SeaOrmUnitOfWork::new(DbHandle::new(db.clone())));
    let clock: Arc<dyn ClockPort> = Arc::new(FixedClock(
        chrono::DateTime::parse_from_rfc3339("2026-08-29T12:00:00Z")
            .unwrap()
            .into(),
    ));
    // Real v7 identifiers: these tests assert on behaviour, and a generator that repeats its last
    // value would make the second insert of a test collide on the primary key.
    let ids: Arc<dyn IdGeneratorPort> = Arc::new(UuidV7Generator);
    (TiposMovimientoService::new(uow, clock, ids), db)
}

fn input(nombre: &str, es_ingreso: bool) -> TipoMovimientoInput {
    TipoMovimientoInput {
        nombre: nombre.to_owned(),
        descripcion: None,
        es_ingreso,
    }
}

fn query(texto: Option<&str>) -> ListQuery<TipoMovimientoFiltroDto> {
    ListQuery {
        filtro: TipoMovimientoFiltroDto {
            texto: texto.map(str::to_owned),
            ..TipoMovimientoFiltroDto::default()
        },
        page: 1,
        page_size: 30,
        sort_by: None,
        sort_dir: SortDir::Asc,
    }
}

#[tokio::test]
async fn el_listado_arranca_con_los_cuatro_tipos_de_sistema() {
    let (service, _db) = service().await;
    let page = service.list(query(None)).await.unwrap();

    assert_eq!(page.total_count, 4);
    assert!(page.items.iter().all(|i| i.es_sistema));
    assert!(
        page.items.iter().all(|i| !i.puede_eliminarse),
        "ningún tipo de sistema puede borrarse"
    );
}

#[tokio::test]
async fn un_tipo_creado_se_lee_de_vuelta() {
    let (service, _db) = service().await;
    let creado = service.create(input("Viáticos", false)).await.unwrap();

    let leido = service.get(creado.id).await.unwrap();
    assert_eq!(leido.nombre, "Viáticos");
    assert!(!leido.es_ingreso);
    assert!(!leido.es_sistema);
    assert_eq!(leido.movimientos_count, 0);
    assert!(leido.puede_eliminarse);
}

#[tokio::test]
async fn el_nombre_se_recorta_al_guardar() {
    let (service, _db) = service().await;
    let creado = service.create(input("  Peajes  ", false)).await.unwrap();
    assert_eq!(creado.nombre, "Peajes");
}

#[tokio::test]
async fn no_se_admiten_dos_tipos_con_el_mismo_nombre_aunque_cambie_el_caso() {
    let (service, _db) = service().await;
    service.create(input("Peajes", false)).await.unwrap();

    let error = service.create(input("PEAJES", false)).await.unwrap_err();
    assert_eq!(
        error.message_key(),
        "Conflict.TipoMovimiento.NombreDuplicado"
    );
}

#[tokio::test]
async fn renombrar_un_tipo_a_su_propio_nombre_no_es_un_conflicto() {
    let (service, _db) = service().await;
    let creado = service.create(input("Peajes", false)).await.unwrap();

    let editado = service
        .update(creado.id, input("Peajes", true), &creado.audit.row_version)
        .await
        .unwrap();
    assert!(editado.es_ingreso);
}

#[tokio::test]
async fn una_edicion_con_version_vieja_es_un_conflicto_de_concurrencia() {
    let (service, _db) = service().await;
    let creado = service.create(input("Peajes", false)).await.unwrap();
    let vieja = creado.audit.row_version.clone();

    service
        .update(creado.id, input("Peajes", false), &vieja)
        .await
        .unwrap();

    let error = service
        .update(creado.id, input("Otro", false), &vieja)
        .await
        .unwrap_err();
    assert_eq!(error.code(), "CONCURRENCY");
}

#[tokio::test]
async fn la_version_avanza_en_cada_edicion() {
    let (service, _db) = service().await;
    let creado = service.create(input("Peajes", false)).await.unwrap();
    assert_eq!(creado.audit.row_version, "0000000000000001");

    let editado = service
        .update(
            creado.id,
            input("Peajes 2", false),
            &creado.audit.row_version,
        )
        .await
        .unwrap();
    assert_eq!(editado.audit.row_version, "0000000000000002");
}

#[tokio::test]
async fn un_tipo_de_sistema_no_puede_cambiar_de_signo() {
    let (service, _db) = service().await;
    let gasto = service.get(GASTO.parse().unwrap()).await.unwrap();

    let error = service
        .update(gasto.id, input("Gasto", true), &gasto.audit.row_version)
        .await
        .unwrap_err();
    assert_eq!(
        error.message_key(),
        "Conflict.TipoMovimiento.SignoDeSistema"
    );
}

#[tokio::test]
async fn un_tipo_de_sistema_no_se_borra() {
    let (service, _db) = service().await;
    let gasto = service.get(GASTO.parse().unwrap()).await.unwrap();

    let error = service
        .delete(gasto.id, &gasto.audit.row_version)
        .await
        .unwrap_err();
    assert_eq!(error.message_key(), "Conflict.TipoMovimiento.EsDeSistema");
}

#[tokio::test]
async fn un_tipo_con_movimientos_no_se_borra_y_el_error_dice_cuantos() {
    let (service, db) = service().await;
    let creado = service.create(input("Peajes", false)).await.unwrap();

    db.execute_unprepared(&format!(
        "INSERT INTO categorias (id, nombre, created_at) \
             VALUES ('c1','Varios','2026-01-01T00:00:00.000Z');
         INSERT INTO movimientos (id, fecha, concepto, monto, tipo_movimiento_id, categoria_id, created_at) \
             VALUES ('m1','2026-01-01T00:00:00.000Z','Peaje',10000,'{}','c1','2026-01-01T00:00:00.000Z');",
        creado.id
    ))
    .await
    .unwrap();

    let error = service
        .delete(creado.id, &creado.audit.row_version)
        .await
        .unwrap_err();

    assert_eq!(error.code(), "DEPENDENCY_IN_USE");
    assert_eq!(error.params().get("count").map(String::as_str), Some("1"));
}

#[tokio::test]
async fn un_tipo_borrado_desaparece_del_listado_y_de_la_lectura() {
    let (service, _db) = service().await;
    let creado = service.create(input("Peajes", false)).await.unwrap();

    service
        .delete(creado.id, &creado.audit.row_version)
        .await
        .unwrap();

    assert_eq!(service.list(query(None)).await.unwrap().total_count, 4);
    assert!(matches!(
        service.get(creado.id).await.unwrap_err(),
        AppError::NotFound { .. }
    ));
}

#[tokio::test]
async fn el_conteo_de_uso_solo_cuenta_movimientos_vivos() {
    let (service, db) = service().await;
    db.execute_unprepared(
        "INSERT INTO categorias (id, nombre, created_at) \
             VALUES ('c1','Varios','2026-01-01T00:00:00.000Z');
         INSERT INTO movimientos (id, fecha, concepto, monto, tipo_movimiento_id, categoria_id, created_at, is_deleted) \
             VALUES ('m1','2026-01-01T00:00:00.000Z','Vivo',10000,'00000000-0000-0000-0000-000000000002','c1','2026-01-01T00:00:00.000Z',0),
                    ('m2','2026-01-01T00:00:00.000Z','Borrado',10000,'00000000-0000-0000-0000-000000000002','c1','2026-01-01T00:00:00.000Z',1);",
    )
    .await
    .unwrap();

    let gasto = service.get(GASTO.parse().unwrap()).await.unwrap();
    assert_eq!(gasto.movimientos_count, 1);
}

#[tokio::test]
async fn el_texto_del_filtro_ignora_mayusculas_y_busca_en_la_descripcion() {
    let (service, _db) = service().await;
    service
        .create(TipoMovimientoInput {
            nombre: "Peajes".into(),
            descripcion: Some("Autopistas y rutas".into()),
            es_ingreso: false,
        })
        .await
        .unwrap();

    assert_eq!(
        service.list(query(Some("PEAJ"))).await.unwrap().total_count,
        1
    );
    assert_eq!(
        service
            .list(query(Some("autopista")))
            .await
            .unwrap()
            .total_count,
        1
    );
    assert_eq!(
        service.list(query(Some("tren"))).await.unwrap().total_count,
        0
    );
}

#[tokio::test]
async fn un_sort_by_desconocido_se_rechaza_en_lugar_de_llegar_al_order_by() {
    let (service, _db) = service().await;
    let mut q = query(None);
    q.sort_by = Some("nombre; DROP TABLE movimientos".into());

    let error = service.list(q).await.unwrap_err();
    assert_eq!(error.code(), "VALIDATION");
    assert_eq!(error.fields()[0].field, "sortBy");
}

#[tokio::test]
async fn el_lookup_devuelve_a_lo_sumo_el_limite_pedido() {
    let (service, _db) = service().await;
    let items = service.lookup(None, Some(2)).await.unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(
        items[0].meta.get("esSistema").map(String::as_str),
        Some("true")
    );
}

#[tokio::test]
async fn una_descripcion_en_blanco_se_guarda_como_ausente() {
    let (service, _db) = service().await;
    let creado = service
        .create(TipoMovimientoInput {
            nombre: "Peajes".into(),
            descripcion: Some("   ".into()),
            es_ingreso: false,
        })
        .await
        .unwrap();
    assert_eq!(creado.descripcion, None);
}

#[tokio::test]
async fn un_nombre_vacio_no_llega_a_la_base() {
    let (service, _db) = service().await;
    let error = service.create(input("  ", true)).await.unwrap_err();
    assert_eq!(
        error.fields()[0].message_key,
        "Validation.TipoMovimiento.NombreRequired"
    );
}
