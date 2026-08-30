//! End-to-end exercise of `categorias`: hierarchy, per-parent uniqueness and the two dependency
//! checks that guard the delete.

use std::sync::Arc;

use eo_application::dtos::categorias::{CategoriaFiltroDto, CategoriaInput};
use eo_application::dtos::common::ListQuery;
use eo_application::ports::repositories::{SortDir, UnitOfWork};
use eo_application::ports::{ClockPort, IdGeneratorPort};
use eo_application::use_cases::categorias::CategoriasService;
use eo_application::AppError;
use eo_domain::clock::FixedClock;
use eo_domain::ids::UuidV7Generator;
use eo_infrastructure::persistence::DbHandle;
use eo_infrastructure::persistence::{open_in_memory, SeaOrmUnitOfWork};
use pretty_assertions::assert_eq;
use sea_orm::{ConnectionTrait, DatabaseConnection};
use uuid::Uuid;

const GASTO: &str = "00000000-0000-0000-0000-000000000002";

async fn service() -> (CategoriasService, DatabaseConnection) {
    let db = open_in_memory().await.unwrap();
    let uow: Arc<dyn UnitOfWork> = Arc::new(SeaOrmUnitOfWork::new(DbHandle::new(db.clone())));
    let clock: Arc<dyn ClockPort> = Arc::new(FixedClock(
        chrono::DateTime::parse_from_rfc3339("2026-08-29T12:00:00Z")
            .unwrap()
            .into(),
    ));
    let ids: Arc<dyn IdGeneratorPort> = Arc::new(UuidV7Generator);
    (CategoriasService::new(uow, clock, ids), db)
}

fn input(nombre: &str, padre: Option<Uuid>) -> CategoriaInput {
    CategoriaInput {
        nombre: nombre.to_owned(),
        descripcion: None,
        color_hex: None,
        icono: None,
        categoria_padre_id: padre,
    }
}

fn query() -> ListQuery<CategoriaFiltroDto> {
    ListQuery {
        filtro: CategoriaFiltroDto::default(),
        page: 1,
        page_size: 30,
        sort_by: None,
        sort_dir: SortDir::Asc,
    }
}

/// Inserts a movement pointing at `categoria` straight through SQL: the movement use case is
/// tested elsewhere, and here it would only be scaffolding.
async fn insertar_movimiento(db: &DatabaseConnection, categoria: Uuid) {
    let id = Uuid::now_v7().to_string();
    db.execute_unprepared(&format!(
        "INSERT INTO movimientos (id, fecha, concepto, monto, cantidad, tipo_movimiento_id, \
         moneda, categoria_id, created_at, row_version, is_deleted) VALUES \
         ('{id}', '2026-08-29T12:00:00.000Z', 'test', 10000, 10000, '{GASTO}', 0, '{categoria}', \
         '2026-08-29T12:00:00.000Z', x'0000000000000001', 0)"
    ))
    .await
    .unwrap();
}

#[tokio::test]
async fn la_base_arranca_sin_categorias() {
    let (service, _db) = service().await;
    assert_eq!(service.list(query()).await.unwrap().total_count, 0);
}

#[tokio::test]
async fn una_categoria_nueva_es_raiz_y_se_puede_borrar() {
    let (service, _db) = service().await;
    let creada = service.create(input("Materiales", None)).await.unwrap();

    assert_eq!(creada.nombre, "Materiales");
    assert_eq!(creada.categoria_padre_id, None);
    assert!(creada.puede_eliminarse);
}

#[tokio::test]
async fn el_color_se_guarda_en_mayusculas_para_que_dos_grafias_no_se_vean_distintas() {
    let (service, _db) = service().await;
    let creada = service
        .create(CategoriaInput {
            color_hex: Some("#ffaa00".into()),
            ..input("Materiales", None)
        })
        .await
        .unwrap();

    assert_eq!(creada.color_hex.as_deref(), Some("#FFAA00"));
}

#[tokio::test]
async fn el_nombre_es_unico_entre_hermanas_pero_no_entre_padres_distintos() {
    let (service, _db) = service().await;
    let a = service.create(input("Obra A", None)).await.unwrap();
    let b = service.create(input("Obra B", None)).await.unwrap();

    service
        .create(input("Materiales", Some(a.id)))
        .await
        .unwrap();
    // Same name under a different parent is a different thing and is allowed.
    service
        .create(input("Materiales", Some(b.id)))
        .await
        .unwrap();

    let error = service
        .create(input("Materiales", Some(a.id)))
        .await
        .unwrap_err();
    assert!(
        matches!(error, AppError::Conflict { code, .. } if code == "CATEGORIA_NOMBRE_DUPLICADO")
    );
}

#[tokio::test]
async fn el_listado_resuelve_el_nombre_del_padre() {
    let (service, _db) = service().await;
    let padre = service.create(input("Obra", None)).await.unwrap();
    service
        .create(input("Materiales", Some(padre.id)))
        .await
        .unwrap();

    let page = service.list(query()).await.unwrap();
    let hija = page
        .items
        .iter()
        .find(|i| i.nombre == "Materiales")
        .unwrap();

    assert_eq!(hija.categoria_padre_nombre.as_deref(), Some("Obra"));
    assert_eq!(hija.hijas_count, 0);
}

#[tokio::test]
async fn solo_raiz_deja_fuera_a_las_hijas() {
    let (service, _db) = service().await;
    let padre = service.create(input("Obra", None)).await.unwrap();
    service
        .create(input("Materiales", Some(padre.id)))
        .await
        .unwrap();

    let page = service
        .list(ListQuery {
            filtro: CategoriaFiltroDto {
                solo_raiz: true,
                ..CategoriaFiltroDto::default()
            },
            ..query()
        })
        .await
        .unwrap();

    assert_eq!(page.total_count, 1);
    assert_eq!(page.items[0].nombre, "Obra");
    assert_eq!(page.items[0].hijas_count, 1);
}

#[tokio::test]
async fn una_categoria_no_puede_convertirse_en_su_propia_nieta() {
    let (service, _db) = service().await;
    let abuela = service.create(input("A", None)).await.unwrap();
    let madre = service.create(input("B", Some(abuela.id))).await.unwrap();

    // A → B → A. The field validator cannot see this one; it needs the ancestor chain.
    let error = service
        .update(
            abuela.id,
            input("A", Some(madre.id)),
            &abuela.audit.row_version,
        )
        .await
        .unwrap_err();

    assert_eq!(
        error.fields().first().map(|f| f.message_key.as_str()),
        Some("Validation.Categoria.PadreCiclico")
    );
}

#[tokio::test]
async fn una_categoria_con_hijas_no_se_borra() {
    let (service, _db) = service().await;
    let padre = service.create(input("Obra", None)).await.unwrap();
    service
        .create(input("Materiales", Some(padre.id)))
        .await
        .unwrap();

    let error = service
        .delete(padre.id, &padre.audit.row_version)
        .await
        .unwrap_err();

    assert!(
        matches!(error, AppError::DependencyInUse { code, .. } if code == "CATEGORIA_CON_HIJAS")
    );
}

#[tokio::test]
async fn una_categoria_con_movimientos_no_se_borra() {
    let (service, db) = service().await;
    let categoria = service.create(input("Materiales", None)).await.unwrap();
    insertar_movimiento(&db, categoria.id).await;

    let error = service
        .delete(categoria.id, &categoria.audit.row_version)
        .await
        .unwrap_err();

    assert!(matches!(error, AppError::DependencyInUse { code, .. } if code == "CATEGORIA_EN_USO"));

    let page = service.list(query()).await.unwrap();
    assert_eq!(page.items[0].movimientos_count, 1);
    assert!(!page.items[0].puede_eliminarse);
}

#[tokio::test]
async fn una_version_vieja_pierde_la_carrera() {
    let (service, _db) = service().await;
    let creada = service.create(input("Materiales", None)).await.unwrap();
    let vieja = creada.audit.row_version.clone();

    service
        .update(creada.id, input("Insumos", None), &vieja)
        .await
        .unwrap();

    let error = service
        .update(creada.id, input("Otro", None), &vieja)
        .await
        .unwrap_err();
    assert!(matches!(error, AppError::Concurrency { .. }));
}

#[tokio::test]
async fn una_categoria_borrada_desaparece_del_listado_y_libera_su_nombre() {
    let (service, _db) = service().await;
    let creada = service.create(input("Materiales", None)).await.unwrap();
    service
        .delete(creada.id, &creada.audit.row_version)
        .await
        .unwrap();

    assert_eq!(service.list(query()).await.unwrap().total_count, 0);
    // The row is still there, but its name is free again: the uniqueness check filters deleted.
    assert!(service.create(input("Materiales", None)).await.is_ok());
}

#[tokio::test]
async fn buscar_una_categoria_inexistente_es_un_not_found() {
    let (service, _db) = service().await;
    let error = service.get(Uuid::now_v7()).await.unwrap_err();
    assert!(matches!(error, AppError::NotFound { .. }));
}
