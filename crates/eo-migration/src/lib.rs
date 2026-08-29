//! Database migrations. No business logic lives here. See `docs/03-modelo-de-datos.md`.

#![forbid(unsafe_code)]

pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![]
    }
}
