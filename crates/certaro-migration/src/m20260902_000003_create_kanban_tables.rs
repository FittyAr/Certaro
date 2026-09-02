use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS kanban_tableros (
    id           TEXT    NOT NULL PRIMARY KEY,
    nombre       TEXT    NOT NULL,
    descripcion  TEXT        NULL,
    color        TEXT        NULL,
    es_preset    INTEGER NOT NULL DEFAULT 0 CHECK (es_preset IN (0,1)),
    tipo_preset  TEXT        NULL,
    activo       INTEGER NOT NULL DEFAULT 1 CHECK (activo IN (0,1)),
    created_at   TEXT    NOT NULL,
    updated_at   TEXT        NULL,
    row_version  BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted   INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at   TEXT        NULL
);
CREATE INDEX IF NOT EXISTS ix_kanban_tableros_is_deleted ON kanban_tableros (is_deleted);

CREATE TABLE IF NOT EXISTS kanban_columnas (
    id              TEXT    NOT NULL PRIMARY KEY,
    tablero_id      TEXT    NOT NULL REFERENCES kanban_tableros (id) ON DELETE CASCADE,
    nombre          TEXT    NOT NULL,
    color           TEXT        NULL,
    orden           INTEGER NOT NULL DEFAULT 0,
    limite_wip      INTEGER     NULL,
    estado_mapeado  INTEGER     NULL,
    created_at      TEXT    NOT NULL,
    updated_at      TEXT        NULL,
    row_version     BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted      INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at      TEXT        NULL
);
CREATE INDEX IF NOT EXISTS ix_kanban_columnas_tablero ON kanban_columnas (tablero_id);
CREATE INDEX IF NOT EXISTS ix_kanban_columnas_orden ON kanban_columnas (tablero_id, orden);

CREATE TABLE IF NOT EXISTS kanban_tarjetas (
    id                 TEXT    NOT NULL PRIMARY KEY,
    columna_id         TEXT    NOT NULL REFERENCES kanban_columnas (id) ON DELETE CASCADE,
    titulo             TEXT    NOT NULL,
    descripcion        TEXT        NULL,
    prioridad          INTEGER NOT NULL DEFAULT 1,
    fecha_vencimiento  TEXT        NULL,
    orden              INTEGER NOT NULL DEFAULT 0,
    trabajo_id         TEXT        NULL REFERENCES trabajos (id) ON DELETE SET NULL,
    orden_trabajo_id   TEXT        NULL REFERENCES ordenes_trabajo (id) ON DELETE SET NULL,
    archivada          INTEGER NOT NULL DEFAULT 0 CHECK (archivada IN (0,1)),
    created_at         TEXT    NOT NULL,
    updated_at         TEXT        NULL,
    row_version        BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted         INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at         TEXT        NULL
);
CREATE INDEX IF NOT EXISTS ix_kanban_tarjetas_columna ON kanban_tarjetas (columna_id);
CREATE INDEX IF NOT EXISTS ix_kanban_tarjetas_trabajo ON kanban_tarjetas (trabajo_id);
CREATE INDEX IF NOT EXISTS ix_kanban_tarjetas_orden_trabajo ON kanban_tarjetas (orden_trabajo_id);
CREATE INDEX IF NOT EXISTS ix_kanban_tarjetas_is_deleted ON kanban_tarjetas (is_deleted);

CREATE TABLE IF NOT EXISTS kanban_etiquetas (
    id           TEXT    NOT NULL PRIMARY KEY,
    nombre       TEXT    NOT NULL,
    color        TEXT    NOT NULL,
    created_at   TEXT    NOT NULL,
    updated_at   TEXT        NULL,
    row_version  BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted   INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at   TEXT        NULL
);
CREATE INDEX IF NOT EXISTS ix_kanban_etiquetas_is_deleted ON kanban_etiquetas (is_deleted);

CREATE TABLE IF NOT EXISTS kanban_tarjeta_etiquetas (
    tarjeta_id   TEXT NOT NULL REFERENCES kanban_tarjetas (id) ON DELETE CASCADE,
    etiqueta_id  TEXT NOT NULL REFERENCES kanban_etiquetas (id) ON DELETE CASCADE,
    PRIMARY KEY (tarjeta_id, etiqueta_id)
);

CREATE TABLE IF NOT EXISTS kanban_tarjeta_checklist (
    id           TEXT    NOT NULL PRIMARY KEY,
    tarjeta_id   TEXT    NOT NULL REFERENCES kanban_tarjetas (id) ON DELETE CASCADE,
    titulo       TEXT    NOT NULL,
    completada   INTEGER NOT NULL DEFAULT 0 CHECK (completada IN (0,1)),
    orden        INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT    NOT NULL,
    updated_at   TEXT        NULL,
    row_version  BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted   INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at   TEXT        NULL
);
CREATE INDEX IF NOT EXISTS ix_kanban_checklist_tarjeta ON kanban_tarjeta_checklist (tarjeta_id);

CREATE TABLE IF NOT EXISTS kanban_tarjeta_asignados (
    id           TEXT NOT NULL PRIMARY KEY,
    tarjeta_id   TEXT NOT NULL REFERENCES kanban_tarjetas (id) ON DELETE CASCADE,
    empleado_id  TEXT     NULL REFERENCES empleados (id) ON DELETE SET NULL,
    usuario_id   TEXT     NULL REFERENCES usuarios (id) ON DELETE SET NULL,
    asignado_en  TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS ix_kanban_asignados_tarjeta ON kanban_tarjeta_asignados (tarjeta_id);

-- Seeds iniciales para tableros preset y etiquetas
-- 1. Tablero preset Trabajos
INSERT OR IGNORE INTO kanban_tableros (id, nombre, descripcion, color, es_preset, tipo_preset, activo, created_at, row_version, is_deleted)
VALUES
('00000000-0000-0000-0002-000000000001', 'Trabajos', 'Tablero sincronizado con los trabajos contratados', '#3b82f6', 1, 'trabajos', 1, '2026-01-01T00:00:00.000Z', X'0000000000000001', 0);

-- Columnas preset Trabajos (0=Presupuestado, 1=EnProceso, 2=Pausado, 3=Finalizado, 4=Cancelado)
INSERT OR IGNORE INTO kanban_columnas (id, tablero_id, nombre, color, orden, limite_wip, estado_mapeado, created_at, row_version, is_deleted)
VALUES
('00000000-0000-0002-0001-000000000001', '00000000-0000-0000-0002-000000000001', 'Presupuestado', '#64748b', 0, NULL, 0, '2026-01-01T00:00:00.000Z', X'0000000000000001', 0),
('00000000-0000-0002-0001-000000000002', '00000000-0000-0000-0002-000000000001', 'En Proceso', '#3b82f6', 1, NULL, 1, '2026-01-01T00:00:00.000Z', X'0000000000000001', 0),
('00000000-0000-0002-0001-000000000003', '00000000-0000-0000-0002-000000000001', 'Pausado', '#f59e0b', 2, NULL, 2, '2026-01-01T00:00:00.000Z', X'0000000000000001', 0),
('00000000-0000-0002-0001-000000000004', '00000000-0000-0000-0002-000000000001', 'Finalizado', '#10b981', 3, NULL, 3, '2026-01-01T00:00:00.000Z', X'0000000000000001', 0),
('00000000-0000-0002-0001-000000000005', '00000000-0000-0000-0002-000000000001', 'Cancelado', '#ef4444', 4, NULL, 4, '2026-01-01T00:00:00.000Z', X'0000000000000001', 0);

-- 2. Tablero preset Órdenes de Trabajo
INSERT OR IGNORE INTO kanban_tableros (id, nombre, descripcion, color, es_preset, tipo_preset, activo, created_at, row_version, is_deleted)
VALUES
('00000000-0000-0000-0002-000000000002', 'Órdenes de Trabajo', 'Tablero sincronizado con órdenes de trabajo operativas', '#8b5cf6', 1, 'ordenes', 1, '2026-01-01T00:00:00.000Z', X'0000000000000001', 0);

INSERT OR IGNORE INTO kanban_columnas (id, tablero_id, nombre, color, orden, limite_wip, estado_mapeado, created_at, row_version, is_deleted)
VALUES
('00000000-0000-0002-0002-000000000001', '00000000-0000-0000-0002-000000000002', 'Borrador', '#64748b', 0, NULL, 0, '2026-01-01T00:00:00.000Z', X'0000000000000001', 0),
('00000000-0000-0002-0002-000000000002', '00000000-0000-0000-0002-000000000002', 'Emitida', '#3b82f6', 1, NULL, 1, '2026-01-01T00:00:00.000Z', X'0000000000000001', 0),
('00000000-0000-0002-0002-000000000003', '00000000-0000-0000-0002-000000000002', 'En Ejecución', '#8b5cf6', 2, NULL, 2, '2026-01-01T00:00:00.000Z', X'0000000000000001', 0),
('00000000-0000-0002-0002-000000000004', '00000000-0000-0000-0002-000000000002', 'Finalizada', '#10b981', 3, NULL, 3, '2026-01-01T00:00:00.000Z', X'0000000000000001', 0);

-- 3. Etiquetas iniciales
INSERT OR IGNORE INTO kanban_etiquetas (id, nombre, color, created_at, row_version, is_deleted)
VALUES
('00000000-0000-0000-0002-000000000010', 'Urgente', '#ef4444', '2026-01-01T00:00:00.000Z', X'0000000000000001', 0),
('00000000-0000-0000-0002-000000000011', 'Materiales', '#f59e0b', '2026-01-01T00:00:00.000Z', X'0000000000000001', 0),
('00000000-0000-0000-0002-000000000012', 'Mano de Obra', '#3b82f6', '2026-01-01T00:00:00.000Z', X'0000000000000001', 0),
('00000000-0000-0000-0002-000000000013', 'Revisión', '#8b5cf6', '2026-01-01T00:00:00.000Z', X'0000000000000001', 0);
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
                    s = s.replace("INTEGER NOT NULL DEFAULT 0 CHECK (activo IN (0,1))", "BOOLEAN NOT NULL DEFAULT FALSE");
                    s = s.replace("INTEGER NOT NULL DEFAULT 1 CHECK (activo IN (0,1))", "BOOLEAN NOT NULL DEFAULT TRUE");
                    s = s.replace("INTEGER NOT NULL DEFAULT 0 CHECK (es_preset IN (0,1))", "BOOLEAN NOT NULL DEFAULT FALSE");
                    s = s.replace("INTEGER NOT NULL DEFAULT 1 CHECK (es_preset IN (0,1))", "BOOLEAN NOT NULL DEFAULT TRUE");
                    s = s.replace("INTEGER NOT NULL DEFAULT 0 CHECK (archivada IN (0,1))", "BOOLEAN NOT NULL DEFAULT FALSE");
                    s = s.replace("INTEGER NOT NULL DEFAULT 0 CHECK (completada IN (0,1))", "BOOLEAN NOT NULL DEFAULT FALSE");
                    s = s.replace("INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1))", "BOOLEAN NOT NULL DEFAULT FALSE");
                    s = s.replace("INSERT OR IGNORE INTO", "INSERT INTO");
                    s = s.replace("WHERE is_deleted = 0", "WHERE is_deleted = FALSE");
                    s
                }
                sea_orm::DatabaseBackend::MySql => {
                    let mut s = trimmed.to_string();
                    s = s.replace("BLOB NOT NULL DEFAULT X'0000000000000001'", "BINARY(8) NOT NULL DEFAULT 0x0000000000000001");
                    s = s.replace("BLOB", "BLOB");
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
            "DROP TABLE IF EXISTS kanban_tarjeta_asignados",
            "DROP TABLE IF EXISTS kanban_tarjeta_checklist",
            "DROP TABLE IF EXISTS kanban_tarjeta_etiquetas",
            "DROP TABLE IF EXISTS kanban_etiquetas",
            "DROP TABLE IF EXISTS kanban_tarjetas",
            "DROP TABLE IF EXISTS kanban_columnas",
            "DROP TABLE IF EXISTS kanban_tableros",
        ];

        for stmt in drops {
            db.execute(sea_orm::Statement::from_string(backend, stmt))
                .await?;
        }

        Ok(())
    }
}
