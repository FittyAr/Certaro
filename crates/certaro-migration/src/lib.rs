//! Database migrations. No business logic lives here. See `docs/03-modelo-de-datos.md`.

#![forbid(unsafe_code)]

pub use sea_orm_migration::prelude::*;

mod m20260101_000001_create_schema;
mod m20260101_000002_seed_system_rows;
mod m20260902_000001_rename_obra_to_proyecto;
mod m20260902_000002_create_auth_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260101_000001_create_schema::Migration),
            Box::new(m20260101_000002_seed_system_rows::Migration),
            Box::new(m20260902_000001_rename_obra_to_proyecto::Migration),
            Box::new(m20260902_000002_create_auth_tables::Migration),
        ]
    }
}
