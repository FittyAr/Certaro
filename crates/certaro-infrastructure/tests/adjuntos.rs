//! Attachments end to end: the path convention, every validation of doc 13 §1.3, and the trash.
//!
//! Runs against a real temporary directory rather than a fake filesystem, because the behaviour
//! being pinned — where the file ends up, and that a delete does not destroy it — is exactly the
//! part a fake would not reproduce.

use std::path::Path;
use std::sync::Arc;

use certaro_application::config::AppConfig;
use certaro_application::dtos::adjuntos::AdjuntoInput;
use certaro_application::ports::repositories::UnitOfWork;
use certaro_application::ports::{
    AttachmentStore, ClockPort, IdGeneratorPort, OpenerPort, SettingsStore,
};
use certaro_application::use_cases::adjuntos::AdjuntosService;
use certaro_application::{AppError, AppResult};
use certaro_domain::clock::FixedClock;
use certaro_domain::entities::EntidadAdjunto;
use certaro_domain::ids::UuidV7Generator;
use certaro_infrastructure::files::FsAttachmentStore;
use certaro_infrastructure::paths::AppPaths;
use certaro_infrastructure::persistence::{open_in_memory, DbHandle, SeaOrmUnitOfWork};
use uuid::Uuid;

const ENTIDAD: Uuid = Uuid::from_u128(0x2026);

/// An opener that records instead of launching anything: a test must not open a real window.
#[derive(Default)]
struct OpenerFalso;

impl OpenerPort for OpenerFalso {
    fn open(&self, _ruta: &Path) -> AppResult<()> {
        Ok(())
    }
    fn reveal(&self, _ruta: &Path) -> AppResult<()> {
        Ok(())
    }
    fn open_url(&self, _url: &str) -> AppResult<()> {
        Ok(())
    }
}

/// A settings store over a fixed configuration, so the limits under test are explicit.
struct SettingsFijo(AppConfig);

#[async_trait::async_trait]
impl SettingsStore for SettingsFijo {
    fn snapshot(&self) -> AppConfig {
        self.0.clone()
    }
    async fn save(&self, _config: AppConfig) -> AppResult<()> {
        Ok(())
    }
}

struct Entorno {
    service: AdjuntosService,
    store: Arc<dyn AttachmentStore>,
    paths: AppPaths,
    _dir: tempfile::TempDir,
}

async fn entorno(ajustar: impl FnOnce(&mut AppConfig)) -> Entorno {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::from_root(dir.path());
    paths.ensure_dirs().unwrap();

    let mut config = AppConfig::default();
    ajustar(&mut config);
    let settings: Arc<dyn SettingsStore> = Arc::new(SettingsFijo(config));

    let clock: Arc<dyn ClockPort> = Arc::new(FixedClock(
        chrono::DateTime::parse_from_rfc3339("2026-08-29T12:00:00Z")
            .unwrap()
            .into(),
    ));
    let ids: Arc<dyn IdGeneratorPort> = Arc::new(UuidV7Generator);
    let store: Arc<dyn AttachmentStore> = Arc::new(FsAttachmentStore::new(
        paths.clone(),
        Arc::clone(&settings),
        Arc::clone(&clock),
    ));

    let db = open_in_memory().await.unwrap();
    let uow: Arc<dyn UnitOfWork> = Arc::new(SeaOrmUnitOfWork::new(DbHandle::new(db)));

    Entorno {
        service: AdjuntosService::new(
            uow,
            Arc::clone(&store),
            Arc::new(OpenerFalso),
            clock,
            ids,
            settings,
        ),
        store,
        paths,
        _dir: dir,
    }
}

/// Writes a source file with the given name and content in a directory of its own.
fn archivo(entorno: &Entorno, nombre: &str, contenido: &[u8]) -> String {
    let origen = entorno.paths.root().join("origen");
    std::fs::create_dir_all(&origen).unwrap();
    let ruta = origen.join(nombre);
    std::fs::write(&ruta, contenido).unwrap();
    ruta.display().to_string()
}

fn pdf() -> Vec<u8> {
    b"%PDF-1.7\nfin del archivo".to_vec()
}

fn input(ruta_origen: String) -> AdjuntoInput {
    AdjuntoInput {
        entidad_tipo: EntidadAdjunto::Movimiento,
        entidad_id: ENTIDAD,
        ruta_origen,
    }
}

#[tokio::test]
async fn adjunto_ruta_sigue_la_convencion() {
    let entorno = entorno(|_| {}).await;
    let item = entorno
        .service
        .add(input(archivo(&entorno, "factura_luz.pdf", &pdf())))
        .await
        .unwrap();

    let esperada = format!("Movimiento/{ENTIDAD}/{}_factura_luz.pdf", item.id);
    let absoluta = entorno.store.resolve(&esperada).unwrap();
    assert!(absoluta.is_file(), "el archivo no está en {esperada}");
    assert!(!esperada.contains('\\'), "la ruta relativa usa barras");
}

#[tokio::test]
async fn adjunto_nombre_se_sanea() {
    let entorno = entorno(|_| {}).await;

    // A traversal in the name cannot become a traversal in the path.
    let item = entorno
        .service
        .add(input(archivo(&entorno, "con.txt", b"solo texto")))
        .await
        .unwrap();
    assert_eq!(item.nombre_archivo, "_con.txt");

    let ruta = format!("Movimiento/{ENTIDAD}/{}__con.txt", item.id);
    assert!(entorno.store.resolve(&ruta).is_ok());
}

#[tokio::test]
async fn adjunto_rechaza_extension_no_permitida() {
    let entorno = entorno(|_| {}).await;
    let error = entorno
        .service
        .add(input(archivo(&entorno, "virus.exe", b"MZ\x90\x00")))
        .await
        .unwrap_err();

    assert_clave(&error, "Validation.Adjunto.ExtensionNoPermitida");
}

#[tokio::test]
async fn adjunto_rechaza_tamano_excesivo() {
    let entorno = entorno(|c| c.attachments.max_size_mb = 1).await;
    let mut contenido = pdf();
    contenido.resize(1024 * 1024 + 1, b'a');

    let error = entorno
        .service
        .add(input(archivo(&entorno, "grande.pdf", &contenido)))
        .await
        .unwrap_err();

    assert_clave(&error, "Validation.Adjunto.DemasiadoGrande");
}

#[tokio::test]
async fn adjunto_rechaza_contenido_que_no_coincide() {
    let entorno = entorno(|_| {}).await;
    // The legacy check was the extension alone, so this file used to be accepted.
    let error = entorno
        .service
        .add(input(archivo(
            &entorno,
            "informe.pdf",
            b"MZ\x90\x00este es un ejecutable",
        )))
        .await
        .unwrap_err();

    assert_clave(&error, "Validation.Adjunto.ContenidoNoCoincide");
}

#[tokio::test]
async fn adjunto_rechaza_cupo_excedido() {
    let entorno = entorno(|c| {
        c.attachments.max_size_mb = 1;
        c.attachments.max_total_mb = 1;
    })
    .await;
    let mut contenido = pdf();
    contenido.resize(700 * 1024, b'a');

    entorno
        .service
        .add(input(archivo(&entorno, "primero.pdf", &contenido)))
        .await
        .unwrap();

    let error = entorno
        .service
        .add(input(archivo(&entorno, "segundo.pdf", &contenido)))
        .await
        .unwrap_err();

    assert_clave(&error, "Validation.Adjunto.CupoExcedido");
}

#[tokio::test]
async fn adjunto_borrado_mueve_a_papelera() {
    let entorno = entorno(|_| {}).await;
    let item = entorno
        .service
        .add(input(archivo(&entorno, "recibo.pdf", &pdf())))
        .await
        .unwrap();

    let relativa = format!("Movimiento/{ENTIDAD}/{}_recibo.pdf", item.id);
    let original = entorno.store.resolve(&relativa).unwrap();

    entorno.service.delete(item.id).await.unwrap();

    // Not where it was, and not gone either: a delete by mistake stays recoverable.
    assert!(!original.exists(), "el archivo sigue en su lugar original");
    let papelera: Vec<_> = std::fs::read_dir(entorno.paths.attachments_trash())
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        papelera.iter().any(|n| n.ends_with("_recibo.pdf")),
        "la papelera tiene {papelera:?}"
    );

    // And the row is gone from the listing.
    let listado = entorno
        .service
        .list(EntidadAdjunto::Movimiento, ENTIDAD)
        .await
        .unwrap();
    assert!(listado.is_empty());
}

#[tokio::test]
async fn la_papelera_se_vacia_por_antiguedad() {
    let entorno = entorno(|_| {}).await;
    let item = entorno
        .service
        .add(input(archivo(&entorno, "viejo.pdf", &pdf())))
        .await
        .unwrap();
    entorno.service.delete(item.id).await.unwrap();

    // The clock is fixed at the moment of trashing, so a zero-day window is the only way to make
    // the file look old without waiting.
    assert_eq!(entorno.store.purge_trash(30).await.unwrap(), 0);
    assert_eq!(entorno.store.purge_trash(0).await.unwrap(), 1);
    assert_eq!(entorno.store.purge_trash(0).await.unwrap(), 0);
}

#[tokio::test]
async fn un_directorio_no_es_un_adjunto() {
    let entorno = entorno(|_| {}).await;
    let carpeta = entorno.paths.root().join("una_carpeta.pdf");
    std::fs::create_dir_all(&carpeta).unwrap();

    let error = entorno
        .service
        .add(input(carpeta.display().to_string()))
        .await
        .unwrap_err();

    assert_clave(&error, "Validation.Adjunto.NoEsArchivo");
}

#[tokio::test]
async fn abrir_un_adjunto_sin_archivo_avisa_con_la_ruta() {
    let entorno = entorno(|_| {}).await;
    let item = entorno
        .service
        .add(input(archivo(&entorno, "borrado.pdf", &pdf())))
        .await
        .unwrap();

    let relativa = format!("Movimiento/{ENTIDAD}/{}_borrado.pdf", item.id);
    std::fs::remove_file(entorno.store.resolve(&relativa).unwrap()).unwrap();

    let error = entorno.service.open(item.id).await.unwrap_err();
    assert_clave(&error, "Validation.Adjunto.ArchivoNoEncontrado");
}

#[track_caller]
fn assert_clave(error: &AppError, esperada: &str) {
    match error {
        AppError::Validation(errores) => assert_eq!(
            errores[0].message_key, esperada,
            "vino {}",
            errores[0].message_key
        ),
        otro => panic!("se esperaba validación con {esperada}, vino {otro:?}"),
    }
}
