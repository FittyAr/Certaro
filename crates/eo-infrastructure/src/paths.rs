//! Where the application keeps its files.
//!
//! Every path derives from one data directory, so a portable install or a test only has to
//! override that one value. See `docs/13-servicios-externos-y-archivos.md`.

use std::path::{Path, PathBuf};

/// Directory names under the data directory. Literals live here and nowhere else.
pub const DB_FILE: &str = "electroobra.db";
pub const CONFIG_FILE: &str = "config.json";
pub const CONFIG_BACKUP_FILE: &str = "config.json.bak";
pub const ATTACHMENTS_DIR: &str = "Adjuntos";
pub const ATTACHMENTS_TRASH_DIR: &str = "Papelera";
pub const BACKUPS_DIR: &str = "Backups";
pub const LOGS_DIR: &str = "Logs";
pub const EXPORTS_DIR: &str = "Exportaciones";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    root: PathBuf,
}

impl AppPaths {
    /// Resolves the data directory: the configured override, or the platform location.
    ///
    /// On Windows that is `%LOCALAPPDATA%\ElectroObra`, on Linux `~/.local/share/ElectroObra`,
    /// on macOS `~/Library/Application Support/ElectroObra`.
    #[must_use]
    pub fn resolve(override_dir: Option<&Path>, app_name: &str) -> Self {
        let root = match override_dir {
            Some(p) => p.to_path_buf(),
            None => platform_data_dir().join(app_name),
        };
        Self { root }
    }

    #[must_use]
    pub fn from_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn database(&self) -> PathBuf {
        self.root.join(DB_FILE)
    }

    #[must_use]
    pub fn config(&self) -> PathBuf {
        self.root.join(CONFIG_FILE)
    }

    #[must_use]
    pub fn config_backup(&self) -> PathBuf {
        self.root.join(CONFIG_BACKUP_FILE)
    }

    #[must_use]
    pub fn attachments(&self) -> PathBuf {
        self.root.join(ATTACHMENTS_DIR)
    }

    #[must_use]
    pub fn attachments_trash(&self) -> PathBuf {
        self.root.join(ATTACHMENTS_DIR).join(ATTACHMENTS_TRASH_DIR)
    }

    /// The backup directory is configurable, so it may be absolute or relative to the root.
    #[must_use]
    pub fn backups(&self, configured: &str) -> PathBuf {
        let p = Path::new(configured);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.root.join(p)
        }
    }

    #[must_use]
    pub fn logs(&self) -> PathBuf {
        self.root.join(LOGS_DIR)
    }

    #[must_use]
    pub fn exports(&self) -> PathBuf {
        self.root.join(EXPORTS_DIR)
    }

    /// Creates the directories the application always needs.
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        for dir in [
            self.root.clone(),
            self.attachments(),
            self.attachments_trash(),
            self.logs(),
        ] {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn platform_data_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(target_os = "macos")]
fn platform_data_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(|h| PathBuf::from(h).join("Library").join("Application Support"))
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform_data_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("share")))
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_path_hangs_off_the_root() {
        let p = AppPaths::from_root("/data");
        assert!(p.database().starts_with("/data"));
        assert!(p.attachments_trash().starts_with(p.attachments()));
    }

    #[test]
    fn an_absolute_backup_directory_is_respected() {
        let p = AppPaths::from_root("/data");
        let abs = if cfg!(windows) { r"C:\bk" } else { "/bk" };
        assert_eq!(p.backups(abs), PathBuf::from(abs));
    }
}
