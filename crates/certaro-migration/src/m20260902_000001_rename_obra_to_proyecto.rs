use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // Check current schema: if `proyectos` already exists (new installs), skip rename
        let has_obras: Option<String> = {
            let stmt = sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='obras'".to_owned(),
            );
            let row = db.query_one(stmt).await?;
            row.map(|r| r.try_get::<String>("", "name").unwrap_or_default())
        };
        let has_proyectos: Option<String> = {
            let stmt = sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='proyectos'".to_owned(),
            );
            let row = db.query_one(stmt).await?;
            row.map(|r| r.try_get::<String>("", "name").unwrap_or_default())
        };
        if has_obras.is_some() && has_proyectos.is_none() {
            db.execute_unprepared("ALTER TABLE obras RENAME TO proyectos")
                .await?;
        }
        // Rename column if still named obra_id
        let has_obra_id: bool = {
            let stmt = sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT COUNT(*) as cnt FROM pragma_table_info('trabajos') WHERE name='obra_id'".to_owned(),
            );
            if let Some(row) = db.query_one(stmt).await? {
                let cnt: i64 = row.try_get::<i64>("", "cnt").unwrap_or(0);
                cnt > 0
            } else {
                false
            }
        };
        if has_obra_id {
            db.execute_unprepared("ALTER TABLE trabajos RENAME COLUMN obra_id TO proyecto_id")
                .await?;
        }
        db.execute_unprepared("DROP INDEX IF EXISTS ux_obras_numero")
            .await?;
        // Only create new indexes if they don't already exist (new installs already have them)
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS ux_proyectos_numero ON proyectos (numero)",
        )
        .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS ix_obras_cliente_id")
            .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS ix_proyectos_cliente_id ON proyectos (cliente_id)",
        )
        .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS ix_obras_estado")
            .await?;
        db.execute_unprepared("CREATE INDEX IF NOT EXISTS ix_proyectos_estado ON proyectos (estado)")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS ix_obras_is_deleted")
            .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS ix_proyectos_is_deleted ON proyectos (is_deleted)",
        )
        .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS ix_trabajos_obra_id")
            .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS ix_trabajos_proyecto_id ON trabajos (proyecto_id)",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let has_proyectos: bool = {
            let stmt = sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='proyectos'".to_owned(),
            );
            let row = db.query_one(stmt).await?;
            row.is_some()
        };
        if has_proyectos {
            let _ = db.execute_unprepared("DROP INDEX IF EXISTS ux_proyectos_numero").await;
            let _ = db.execute_unprepared("DROP INDEX IF EXISTS ix_proyectos_cliente_id").await;
            let _ = db.execute_unprepared("DROP INDEX IF EXISTS ix_proyectos_estado").await;
            let _ = db.execute_unprepared("DROP INDEX IF EXISTS ix_proyectos_is_deleted").await;
            let _ = db.execute_unprepared("DROP INDEX IF EXISTS ix_trabajos_proyecto_id").await;
            let _ = db.execute_unprepared("ALTER TABLE proyectos RENAME TO obras").await;
            let _ = db.execute_unprepared("CREATE UNIQUE INDEX IF NOT EXISTS ux_obras_numero ON obras (numero)").await;
            let _ = db.execute_unprepared("CREATE INDEX IF NOT EXISTS ix_obras_cliente_id ON obras (cliente_id)").await;
            let _ = db.execute_unprepared("CREATE INDEX IF NOT EXISTS ix_obras_estado ON obras (estado)").await;
            let _ = db.execute_unprepared("CREATE INDEX IF NOT EXISTS ix_obras_is_deleted ON obras (is_deleted)").await;
            let _ = db.execute_unprepared("ALTER TABLE trabajos RENAME COLUMN proyecto_id TO obra_id").await;
            let _ = db.execute_unprepared("CREATE INDEX IF NOT EXISTS ix_trabajos_obra_id ON trabajos (obra_id)").await;
        }
        Ok(())
    }
}
