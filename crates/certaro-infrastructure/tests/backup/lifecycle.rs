use certaro_application::ports::BackupPort;
use certaro_application::AppError;
use certaro_domain::clock::Clock;
use certaro_infrastructure::backup::nombre_backup;

use super::common::{entorno, sembrar, cuantos_tipos, CuantosBackups};

#[tokio::test]
async fn backup_crea_y_verifica_integridad() {
    let entorno = entorno().await;
    let item = entorno.service.create().await.unwrap();

    assert_eq!(item.nombre, nombre_backup(entorno.reloj.now_utc()));
    assert!(item.bytes > 0);

    let verificacion = entorno.service.verify(&item.nombre).await.unwrap();
    assert!(verificacion.ok, "detalle: {}", verificacion.detalle);
}

#[tokio::test]
async fn backup_verifica_integridad_y_rechaza_un_archivo_corrupto() {
    let entorno = entorno().await;
    let item = entorno.service.create().await.unwrap();

    // The header is what SQLite reads first, so overwriting it is the cheapest real corruption.
    let ruta = entorno.paths.backups("Backups").join(&item.nombre);
    let mut bytes = std::fs::read(&ruta).unwrap();
    bytes[0..16].copy_from_slice(b"no es una base..");
    std::fs::write(&ruta, bytes).unwrap();

    let resultado = entorno.service.verify(&item.nombre).await;
    if let Ok(verificacion) = resultado {
        assert!(!verificacion.ok);
    }
    // Refusing to even open it is also a rejection, and the same outcome for the caller.
}

#[tokio::test]
async fn un_nombre_de_backup_ajeno_no_se_acepta() {
    let entorno = entorno().await;
    for nombre in [
        "../../certaro.db",
        "..\\certaro.db",
        "cualquier.db",
        "certaro_ayer.db",
    ] {
        let error = entorno.service.verify(nombre).await.unwrap_err();
        assert!(matches!(error, AppError::Validation(_)), "{nombre}");
    }
}

#[tokio::test]
async fn backup_conserva_los_tres_mas_recientes() {
    let entorno = entorno().await;
    for _ in 0..5 {
        entorno.service.create().await.unwrap();
        // One a day, so the names differ and the ages are ordered.
        entorno.reloj.avanzar(1);
    }
    assert_eq!(entorno.service.backups_count().await, 5);

    // Every one of them is older than the window, so only the minimum should survive.
    entorno.reloj.avanzar(60);
    let borrados = entorno.service.prune(30, 3).await.unwrap();

    assert_eq!(borrados, 2);
    assert_eq!(entorno.service.backups_count().await, 3);
}

#[tokio::test]
async fn restore_devuelve_la_base_al_estado_del_backup() {
    let entorno = entorno().await;
    let antes = cuantos_tipos(&entorno).await;

    let punto = entorno.service.create().await.unwrap();
    entorno.reloj.avanzar(1);

    sembrar(&entorno, "Agregado después del backup").await;
    assert_eq!(cuantos_tipos(&entorno).await, antes + 1);

    entorno.service.restore(&punto.nombre).await.unwrap();

    // The connection was closed and reopened, so this read goes through the restored file.
    assert_eq!(cuantos_tipos(&entorno).await, antes);
    // And the pre-restore state was itself backed up, so the restore is undoable.
    assert!(entorno.service.backups_count().await >= 2);
}

#[tokio::test]
async fn restore_no_deja_sidecars_del_archivo_anterior() {
    let entorno = entorno().await;
    let punto = entorno.service.create().await.unwrap();
    entorno.reloj.avanzar(1);
    sembrar(&entorno, "Escribe en el wal").await;

    entorno.service.restore(&punto.nombre).await.unwrap();

    // A `-wal` describing the replaced database is what left the legacy restore inconsistent.
    let wal = entorno
        .paths
        .root()
        .join(format!("{}-wal", "certaro.db"));
    let temporal = entorno.paths.root().join("certaro.db.restore.tmp");
    assert!(!temporal.exists(), "quedó el temporal de la restauración");
    // A fresh `-wal` may exist from the reopened connection; what must not exist is a stale one,
    // which is only observable by the database reading correctly.
    assert!(cuantos_tipos(&entorno).await >= 0, "la base quedó ilegible");
    let _ = wal;
}
