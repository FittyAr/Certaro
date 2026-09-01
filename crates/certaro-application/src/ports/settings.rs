//! Settings port. The loader and the file format live in infrastructure.

use crate::config::AppConfig;
use crate::result::AppResult;
use async_trait::async_trait;

#[async_trait]
pub trait SettingsStore: Send + Sync {
    /// An immutable snapshot. Reads never block on the writer.
    fn snapshot(&self) -> AppConfig;

    /// Validates, persists only the keys that differ from the default, and swaps the snapshot.
    async fn save(&self, config: AppConfig) -> AppResult<()>;
}
