//! Database migrations. No business logic lives here. See `docs/03-modelo-de-datos.md`.

#![forbid(unsafe_code)]

pub use sea_orm_migration::prelude::*;

mod m20260101_000001_create_schema;
mod m20260101_000002_seed_system_rows;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260101_000001_create_schema::Migration),
            Box::new(m20260101_000002_seed_system_rows::Migration),
        ]
    }
}
