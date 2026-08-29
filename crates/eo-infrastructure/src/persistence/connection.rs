//! Opening the database. See `docs/03-modelo-de-datos.md` §1 and `docs/02-arquitectura.md` §9.

use std::path::Path;
use std::time::Duration;

use eo_migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbErr, Statement};
use tracing::info;

/// PRAGMAs applied to every connection of the pool.
///
/// `foreign_keys` is off by default in SQLite and is per-connection, not per-database, so it has
/// to be set here; without it the `RESTRICT` clauses of the schema are decoration. WAL lets a
/// backup read the file while the application writes, and the busy timeout is what keeps that
/// from surfacing as a `database is locked` error in the user's face.
const PRAGMAS: &[&str] = &[
    "PRAGMA foreign_keys = ON;",
    "PRAGMA journal_mode = WAL;",
    "PRAGMA busy_timeout = 5000;",
    "PRAGMA synchronous = NORMAL;",
];

/// Opens the database at `path`, creating the file if it is not there, and runs the migrations.
pub async fn open(path: &Path) -> Result<DatabaseConnection, DbErr> {
    let url = sqlite_url(path);
    let db = connect(&url).await?;
    Migrator::up(&db, None).await?;
    info!(path = %path.display(), "database ready");
    Ok(db)
}

/// An in-memory database with the schema applied, for tests.
pub async fn open_in_memory() -> Result<DatabaseConnection, DbErr> {
    let db = connect("sqlite::memory:").await?;
    Migrator::up(&db, None).await?;
    Ok(db)
}

async fn connect(url: &str) -> Result<DatabaseConnection, DbErr> {
    let mut options = ConnectOptions::new(url.to_owned());
    options
        .max_connections(8)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .sqlx_logging(false);

    let db = Database::connect(options).await?;
    apply_pragmas(&db).await?;
    Ok(db)
}

async fn apply_pragmas(db: &DatabaseConnection) -> Result<(), DbErr> {
    for pragma in PRAGMAS {
        db.execute(Statement::from_string(db.get_database_backend(), *pragma))
            .await?;
    }
    Ok(())
}

fn sqlite_url(path: &Path) -> String {
    format!(
        "sqlite://{}?mode=rwc",
        path.display().to_string().replace('\\', "/")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_windows_path_becomes_a_valid_sqlite_url() {
        let url = sqlite_url(Path::new(r"C:\data\electroobra.db"));
        assert_eq!(url, "sqlite://C:/data/electroobra.db?mode=rwc");
    }
}
