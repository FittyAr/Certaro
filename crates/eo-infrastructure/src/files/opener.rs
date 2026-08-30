//! Handing a file or a URL to the operating system. See `docs/13-servicios-externos-y-archivos.md`
//! §1.5.
//!
//! «Reveal in the file manager» is the reason this is not simply the Tauri plugin: the plugin opens
//! a file, and showing it selected in its folder needs the platform command.

use std::path::Path;
use std::process::Command;

use eo_application::ports::OpenerPort;
use eo_application::result::AppResult;
use eo_application::AppError;

#[derive(Debug, Default)]
pub struct SystemOpener;

impl OpenerPort for SystemOpener {
    fn open(&self, ruta: &Path) -> AppResult<()> {
        if !ruta.exists() {
            return Err(AppError::io(anyhow::anyhow!(
                "opener.open missing {}",
                ruta.display()
            )));
        }
        lanzar(abrir(ruta))
    }

    fn reveal(&self, ruta: &Path) -> AppResult<()> {
        if !ruta.exists() {
            return Err(AppError::io(anyhow::anyhow!(
                "opener.reveal missing {}",
                ruta.display()
            )));
        }
        lanzar(revelar(ruta))
    }

    fn open_url(&self, url: &str) -> AppResult<()> {
        // Only the two schemes the application produces: a deep link for mail or the browser. This
        // refuses `file:` and anything else before it reaches a shell.
        if !(url.starts_with("https://") || url.starts_with("mailto:")) {
            return Err(AppError::io(anyhow::anyhow!("opener.url rejected scheme")));
        }
        lanzar(abrir_url(url))
    }
}

fn lanzar(mut comando: Command) -> AppResult<()> {
    comando
        .spawn()
        .map(|_| ())
        .map_err(|e| AppError::io(anyhow::anyhow!("opener.spawn: {e}")))
}

#[cfg(target_os = "windows")]
fn abrir(ruta: &Path) -> Command {
    // `explorer.exe <path>` opens with the registered application. Deliberately not `cmd /C start`,
    // which would put the path through a shell.
    let mut c = Command::new("explorer.exe");
    c.arg(ruta);
    c
}

#[cfg(target_os = "windows")]
fn revelar(ruta: &Path) -> Command {
    let mut c = Command::new("explorer.exe");
    c.arg(format!("/select,{}", ruta.display()));
    c
}

#[cfg(target_os = "windows")]
fn abrir_url(url: &str) -> Command {
    let mut c = Command::new("explorer.exe");
    c.arg(url);
    c
}

#[cfg(target_os = "macos")]
fn abrir(ruta: &Path) -> Command {
    let mut c = Command::new("open");
    c.arg(ruta);
    c
}

#[cfg(target_os = "macos")]
fn revelar(ruta: &Path) -> Command {
    let mut c = Command::new("open");
    c.arg("-R").arg(ruta);
    c
}

#[cfg(target_os = "macos")]
fn abrir_url(url: &str) -> Command {
    let mut c = Command::new("open");
    c.arg(url);
    c
}

#[cfg(target_os = "linux")]
fn abrir(ruta: &Path) -> Command {
    let mut c = Command::new("xdg-open");
    c.arg(ruta);
    c
}

#[cfg(target_os = "linux")]
fn revelar(ruta: &Path) -> Command {
    // No portable «select the file» on Linux, so the folder is what opens.
    let mut c = Command::new("xdg-open");
    c.arg(ruta.parent().unwrap_or(ruta));
    c
}

#[cfg(target_os = "linux")]
fn abrir_url(url: &str) -> Command {
    let mut c = Command::new("xdg-open");
    c.arg(url);
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_archivo_que_no_existe_no_se_intenta_abrir() {
        let error = SystemOpener
            .open(Path::new("D:/no/existe/adjunto.pdf"))
            .unwrap_err();
        assert!(matches!(error, AppError::Io(_)));
    }

    #[test]
    fn solo_se_abren_las_urls_que_la_aplicacion_produce() {
        for url in [
            "file:///C:/Windows/System32/cmd.exe",
            "javascript:alert(1)",
            "http://insegura.example",
            "",
        ] {
            assert!(SystemOpener.open_url(url).is_err(), "{url}");
        }
    }
}
