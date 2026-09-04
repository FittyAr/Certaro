//! The 21 tables of `docs/03-modelo-de-datos.md` §3, in dependency order.
//!
//! The DDL is written as literal SQL rather than through the schema builder on purpose: the
//! document is the authoritative definition, and partial indexes (`WHERE is_deleted = 0`),
//! expression indexes (`IFNULL(...)`) and `WITHOUT ROWID` have no faithful representation in the
//! builder. Keeping the text identical makes a discrepancy visible in a diff.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const SCHEMA: &str = include_str!("up.sql");
const DROP: &str = include_str!("down.sql");

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(SCHEMA).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(DROP).await?;
        Ok(())
    }
}
