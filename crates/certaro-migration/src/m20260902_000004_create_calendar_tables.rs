use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS calendario_grupos_recurso (
    id           TEXT    NOT NULL PRIMARY KEY,
    nombre       TEXT    NOT NULL,
    color        TEXT        NULL,
    created_at   TEXT    NOT NULL,
    updated_at   TEXT        NULL,
    row_version  BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted   INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at   TEXT        NULL
);
CREATE INDEX IF NOT EXISTS ix_cal_grupos_is_deleted ON calendario_grupos_recurso (is_deleted);

CREATE TABLE IF NOT EXISTS calendario_recursos (
    id           TEXT    NOT NULL PRIMARY KEY,
    grupo_id     TEXT        NULL REFERENCES calendario_grupos_recurso (id) ON DELETE SET NULL,
    nombre       TEXT    NOT NULL,
    tipo         TEXT    NOT NULL,
    empleado_id  TEXT        NULL REFERENCES empleados (id) ON DELETE SET NULL,
    color        TEXT        NULL,
    activo       INTEGER NOT NULL DEFAULT 1 CHECK (activo IN (0,1)),
    created_at   TEXT    NOT NULL,
    updated_at   TEXT        NULL,
    row_version  BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted   INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at   TEXT        NULL
);
CREATE INDEX IF NOT EXISTS ix_cal_recursos_grupo ON calendario_recursos (grupo_id);
CREATE INDEX IF NOT EXISTS ix_cal_recursos_empleado ON calendario_recursos (empleado_id);
CREATE INDEX IF NOT EXISTS ix_cal_recursos_is_deleted ON calendario_recursos (is_deleted);

CREATE TABLE IF NOT EXISTS calendario_eventos (
    id                 TEXT    NOT NULL PRIMARY KEY,
    titulo             TEXT    NOT NULL,
    descripcion        TEXT        NULL,
    tipo               TEXT    NOT NULL,
    inicio             TEXT    NOT NULL,
    fin                TEXT    NOT NULL,
    todo_el_dia        INTEGER NOT NULL DEFAULT 0 CHECK (todo_el_dia IN (0,1)),
    color              TEXT        NULL,
    trabajo_id         TEXT        NULL REFERENCES trabajos (id) ON DELETE SET NULL,
    kanban_tarjeta_id  TEXT        NULL REFERENCES kanban_tarjetas (id) ON DELETE SET NULL,
    created_at         TEXT    NOT NULL,
    updated_at         TEXT        NULL,
    row_version        BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted         INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at         TEXT        NULL
);
CREATE INDEX IF NOT EXISTS ix_cal_eventos_inicio ON calendario_eventos (inicio);
CREATE INDEX IF NOT EXISTS ix_cal_eventos_fin ON calendario_eventos (fin);
CREATE INDEX IF NOT EXISTS ix_cal_eventos_trabajo ON calendario_eventos (trabajo_id);
CREATE INDEX IF NOT EXISTS ix_cal_eventos_is_deleted ON calendario_eventos (is_deleted);

CREATE TABLE IF NOT EXISTS calendario_evento_recursos (
    evento_id   TEXT NOT NULL REFERENCES calendario_eventos (id) ON DELETE CASCADE,
    recurso_id  TEXT NOT NULL REFERENCES calendario_recursos (id) ON DELETE CASCADE,
    PRIMARY KEY (evento_id, recurso_id)
);
CREATE INDEX IF NOT EXISTS ix_cal_ev_rec_recurso ON calendario_evento_recursos (recurso_id);

-- Semilla de Grupos de Recurso iniciales
INSERT OR IGNORE INTO calendario_grupos_recurso (id, nombre, color, created_at, row_version, is_deleted)
VALUES
('00000000-0000-0000-0003-000000000001', 'Personal', '#3b82f6', '2026-01-01T00:00:00.000Z', X'0000000000000001', 0),
('00000000-0000-0000-0003-000000000002', 'Vehículos', '#10b981', '2026-01-01T00:00:00.000Z', X'0000000000000001', 0),
('00000000-0000-0000-0003-000000000003', 'Equipos', '#f59e0b', '2026-01-01T00:00:00.000Z', X'0000000000000001', 0);
"#;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();

        for raw_stmt in SCHEMA.split(';') {
            let trimmed = raw_stmt.trim();
            if trimmed.is_empty() {
                continue;
            }

            let stmt = match backend {
                sea_orm::DatabaseBackend::Postgres => {
                    let mut s = trimmed.to_string();
                    s = s.replace("BLOB NOT NULL DEFAULT X'0000000000000001'", "BYTEA NOT NULL DEFAULT '\\x0000000000000001'::bytea");
                    s = s.replace("BLOB", "BYTEA");
                    s = s.replace("X'0000000000000001'", "'\\x0000000000000001'::bytea");
                    s = s.replace("INTEGER NOT NULL DEFAULT 0 CHECK (todo_el_dia IN (0,1))", "BOOLEAN NOT NULL DEFAULT FALSE");
                    s = s.replace("INTEGER NOT NULL DEFAULT 1 CHECK (activo IN (0,1))", "BOOLEAN NOT NULL DEFAULT TRUE");
                    s = s.replace("INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1))", "BOOLEAN NOT NULL DEFAULT FALSE");
                    s = s.replace("INSERT OR IGNORE INTO", "INSERT INTO");
                    s = s.replace("WHERE is_deleted = 0", "WHERE is_deleted = FALSE");
                    s
                }
                sea_orm::DatabaseBackend::MySql => {
                    let mut s = trimmed.to_string();
                    s = s.replace("BLOB NOT NULL DEFAULT X'0000000000000001'", "BINARY(8) NOT NULL DEFAULT 0x0000000000000001");
                    s = s.replace("X'0000000000000001'", "0x0000000000000001");
                    s = s.replace("INSERT OR IGNORE INTO", "INSERT IGNORE INTO");
                    s = s.replace("TEXT NOT NULL PRIMARY KEY", "VARCHAR(36) NOT NULL PRIMARY KEY");
                    s = s.replace("TEXT NOT NULL REFERENCES", "VARCHAR(36) NOT NULL REFERENCES");
                    s = s.replace("TEXT NULL REFERENCES", "VARCHAR(36) NULL REFERENCES");
                    s = s.replace("TEXT NULL", "VARCHAR(255) NULL");
                    s
                }
                sea_orm::DatabaseBackend::Sqlite => trimmed.to_string(),
            };

            db.execute(sea_orm::Statement::from_string(backend, stmt))
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();

        let drops = [
            "DROP TABLE IF EXISTS calendario_evento_recursos",
            "DROP TABLE IF EXISTS calendario_eventos",
            "DROP TABLE IF EXISTS calendario_recursos",
            "DROP TABLE IF EXISTS calendario_grupos_recurso",
        ];

        for stmt in drops {
            db.execute(sea_orm::Statement::from_string(backend, stmt))
                .await?;
        }

        Ok(())
    }
}
