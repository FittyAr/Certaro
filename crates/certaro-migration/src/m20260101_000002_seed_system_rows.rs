//! System rows of `docs/03-modelo-de-datos.md` §5.
//!
//! The identifiers are fixed and the timestamp is a constant, so running the seed twice produces
//! the same database. `INSERT OR IGNORE` makes it idempotent: a user who renamed a system row
//! keeps their name.
//!
//! There is no seed for `categorias` on purpose (§5.3): the suggested list is configuration, and
//! the user is allowed to end up with none.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const SEED: &str = r#"
INSERT OR IGNORE INTO tipos_movimiento
    (id, nombre, descripcion, es_ingreso, es_sistema, created_at, row_version, is_deleted)
VALUES
    ('00000000-0000-0000-0000-000000000001','Ingreso' ,NULL,1,1,'2026-01-01T00:00:00.000Z',X'0000000000000001',0),
    ('00000000-0000-0000-0000-000000000002','Gasto'   ,NULL,0,1,'2026-01-01T00:00:00.000Z',X'0000000000000001',0),
    ('00000000-0000-0000-0000-000000000003','Adelanto',NULL,0,1,'2026-01-01T00:00:00.000Z',X'0000000000000001',0),
    ('00000000-0000-0000-0000-000000000004','Ajuste'  ,NULL,1,1,'2026-01-01T00:00:00.000Z',X'0000000000000001',0);

INSERT OR IGNORE INTO tipos_concepto_pago
    (id, nombre, es_sistema, created_at, row_version, is_deleted)
VALUES
    ('00000000-0000-0000-0000-000000000101','Adelanto'   ,1,'2026-01-01T00:00:00.000Z',X'0000000000000001',0),
    ('00000000-0000-0000-0000-000000000102','Quincena'   ,1,'2026-01-01T00:00:00.000Z',X'0000000000000001',0),
    ('00000000-0000-0000-0000-000000000103','Liquidación',1,'2026-01-01T00:00:00.000Z',X'0000000000000001',0),
    ('00000000-0000-0000-0000-000000000104','Viático'    ,1,'2026-01-01T00:00:00.000Z',X'0000000000000001',0);

INSERT OR IGNORE INTO app_metadata (key, value, updated_at)
VALUES ('SystemSeedVersion','1','2026-01-01T00:00:00.000Z');
"#;

const UNSEED: &str = r#"
DELETE FROM tipos_movimiento WHERE es_sistema = 1;
DELETE FROM tipos_concepto_pago WHERE es_sistema = 1;
DELETE FROM app_metadata WHERE key = 'SystemSeedVersion';
"#;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(SEED).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(UNSEED).await?;
        Ok(())
    }
}
