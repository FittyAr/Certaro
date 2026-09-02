use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS usuarios (
    id                TEXT    NOT NULL PRIMARY KEY,
    email             TEXT    NOT NULL,
    nombre_completo   TEXT    NOT NULL,
    password_hash     TEXT        NULL,
    activo            INTEGER NOT NULL DEFAULT 1 CHECK (activo IN (0,1)),
    requiere_2fa      INTEGER NOT NULL DEFAULT 0 CHECK (requiere_2fa IN (0,1)),
    totp_secret       TEXT        NULL,
    ultimo_login      TEXT        NULL,
    intentos_fallidos INTEGER NOT NULL DEFAULT 0,
    bloqueado_hasta   TEXT        NULL,
    created_at        TEXT    NOT NULL,
    updated_at        TEXT        NULL,
    row_version       BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted        INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at        TEXT        NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_usuarios_email ON usuarios (email) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS ix_usuarios_is_deleted ON usuarios (is_deleted);

CREATE TABLE IF NOT EXISTS roles (
    id           TEXT    NOT NULL PRIMARY KEY,
    nombre       TEXT    NOT NULL,
    descripcion  TEXT        NULL,
    es_sistema   INTEGER NOT NULL DEFAULT 0 CHECK (es_sistema IN (0,1)),
    prioridad    INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT    NOT NULL,
    updated_at   TEXT        NULL,
    row_version  BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted   INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at   TEXT        NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_roles_nombre ON roles (nombre) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS ix_roles_is_deleted ON roles (is_deleted);

CREATE TABLE IF NOT EXISTS permisos (
    id           TEXT    NOT NULL PRIMARY KEY,
    modulo       TEXT    NOT NULL,
    accion       TEXT    NOT NULL,
    recurso      TEXT        NULL,
    clave        TEXT    NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_permisos_clave ON permisos (clave);
CREATE INDEX IF NOT EXISTS ix_permisos_modulo ON permisos (modulo);

CREATE TABLE IF NOT EXISTS usuario_roles (
    id           TEXT    NOT NULL PRIMARY KEY,
    usuario_id   TEXT    NOT NULL,
    rol_id       TEXT    NOT NULL,
    created_at   TEXT    NOT NULL,
    updated_at   TEXT        NULL,
    row_version  BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted   INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at   TEXT        NULL,
    CONSTRAINT fk_usuario_roles_usuario FOREIGN KEY (usuario_id) REFERENCES usuarios (id) ON DELETE CASCADE,
    CONSTRAINT fk_usuario_roles_rol FOREIGN KEY (rol_id) REFERENCES roles (id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_usuario_roles_usuario_rol ON usuario_roles (usuario_id, rol_id) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS ix_usuario_roles_usuario_id ON usuario_roles (usuario_id);
CREATE INDEX IF NOT EXISTS ix_usuario_roles_rol_id ON usuario_roles (rol_id);

CREATE TABLE IF NOT EXISTS rol_permisos (
    id           TEXT    NOT NULL PRIMARY KEY,
    rol_id       TEXT    NOT NULL,
    permiso_id   TEXT    NOT NULL,
    CONSTRAINT fk_rol_permisos_rol FOREIGN KEY (rol_id) REFERENCES roles (id) ON DELETE CASCADE,
    CONSTRAINT fk_rol_permisos_permiso FOREIGN KEY (permiso_id) REFERENCES permisos (id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_rol_permisos_rol_permiso ON rol_permisos (rol_id, permiso_id);

CREATE TABLE IF NOT EXISTS sesiones (
    id           TEXT    NOT NULL PRIMARY KEY,
    usuario_id   TEXT    NOT NULL,
    token_hash   TEXT    NOT NULL,
    expira_en    TEXT    NOT NULL,
    ip           TEXT        NULL,
    user_agent   TEXT        NULL,
    created_at   TEXT    NOT NULL,
    CONSTRAINT fk_sesiones_usuario FOREIGN KEY (usuario_id) REFERENCES usuarios (id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_sesiones_token_hash ON sesiones (token_hash);
CREATE INDEX IF NOT EXISTS ix_sesiones_usuario_id ON sesiones (usuario_id);
CREATE INDEX IF NOT EXISTS ix_sesiones_expira_en ON sesiones (expira_en);

CREATE TABLE IF NOT EXISTS auth_externo (
    id                 TEXT    NOT NULL PRIMARY KEY,
    usuario_id         TEXT    NOT NULL,
    proveedor          TEXT    NOT NULL,
    proveedor_user_id  TEXT    NOT NULL,
    email              TEXT        NULL,
    vinculado_en       TEXT    NOT NULL,
    CONSTRAINT fk_auth_externo_usuario FOREIGN KEY (usuario_id) REFERENCES usuarios (id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_auth_externo_prov_user ON auth_externo (proveedor, proveedor_user_id);
CREATE INDEX IF NOT EXISTS ix_auth_externo_usuario_id ON auth_externo (usuario_id);
"#;

const SEED: &str = r#"
-- 1. Roles del sistema
INSERT OR IGNORE INTO roles (id, nombre, descripcion, es_sistema, prioridad, created_at, row_version, is_deleted)
VALUES
('00000000-0000-0000-0000-000000000010', 'Administrador', 'Control total de la plataforma y administración de usuarios', 1, 100, '2026-01-01T00:00:00.000Z', X'0000000000000001', 0),
('00000000-0000-0000-0000-000000000020', 'Operador', 'Gestión operativa (movimientos, facturación, obras y personal)', 1, 50, '2026-01-01T00:00:00.000Z', X'0000000000000001', 0),
('00000000-0000-0000-0000-000000000030', 'Visualizador', 'Acceso de sólo lectura para reportes y tableros', 1, 10, '2026-01-01T00:00:00.000Z', X'0000000000000001', 0);

-- 2. Permisos del sistema
INSERT OR IGNORE INTO permisos (id, modulo, accion, recurso, clave)
VALUES
('00000000-0000-0000-0001-000000000001', 'movimientos', 'ver', NULL, 'movimientos:ver'),
('00000000-0000-0000-0001-000000000002', 'movimientos', 'crear', NULL, 'movimientos:crear'),
('00000000-0000-0000-0001-000000000003', 'movimientos', 'editar', NULL, 'movimientos:editar'),
('00000000-0000-0000-0001-000000000004', 'movimientos', 'borrar', NULL, 'movimientos:borrar'),

('00000000-0000-0000-0001-000000000005', 'facturas', 'ver', NULL, 'facturas:ver'),
('00000000-0000-0000-0001-000000000006', 'facturas', 'crear', NULL, 'facturas:crear'),
('00000000-0000-0000-0001-000000000007', 'facturas', 'editar', NULL, 'facturas:editar'),
('00000000-0000-0000-0001-000000000008', 'facturas', 'borrar', NULL, 'facturas:borrar'),

('00000000-0000-0000-0001-000000000009', 'empleados', 'ver', NULL, 'empleados:ver'),
('00000000-0000-0000-0001-000000000010', 'empleados', 'crear', NULL, 'empleados:crear'),
('00000000-0000-0000-0001-000000000011', 'empleados', 'editar', NULL, 'empleados:editar'),
('00000000-0000-0000-0001-000000000012', 'empleados', 'borrar', NULL, 'empleados:borrar'),

('00000000-0000-0000-0001-000000000013', 'asistencias', 'ver', NULL, 'asistencias:ver'),
('00000000-0000-0000-0001-000000000014', 'asistencias', 'registrar', NULL, 'asistencias:registrar'),
('00000000-0000-0000-0001-000000000015', 'asistencias', 'editar', NULL, 'asistencias:editar'),

('00000000-0000-0000-0001-000000000016', 'liquidaciones', 'ver', NULL, 'liquidaciones:ver'),
('00000000-0000-0000-0001-000000000017', 'liquidaciones', 'generar', NULL, 'liquidaciones:generar'),
('00000000-0000-0000-0001-000000000018', 'liquidaciones', 'pagar', NULL, 'liquidaciones:pagar'),

('00000000-0000-0000-0001-000000000019', 'proyectos', 'ver', NULL, 'proyectos:ver'),
('00000000-0000-0000-0001-000000000020', 'proyectos', 'crear', NULL, 'proyectos:crear'),
('00000000-0000-0000-0001-000000000021', 'proyectos', 'editar', NULL, 'proyectos:editar'),

('00000000-0000-0000-0001-000000000022', 'trabajos', 'ver', NULL, 'trabajos:ver'),
('00000000-0000-0000-0001-000000000023', 'trabajos', 'crear', NULL, 'trabajos:crear'),
('00000000-0000-0000-0001-000000000024', 'trabajos', 'editar', NULL, 'trabajos:editar'),

('00000000-0000-0000-0001-000000000025', 'kanban', 'ver', NULL, 'kanban:ver'),
('00000000-0000-0000-0001-000000000026', 'kanban', 'crear_tarjeta', NULL, 'kanban:crear_tarjeta'),
('00000000-0000-0000-0001-000000000027', 'kanban', 'mover_tarjeta', NULL, 'kanban:mover_tarjeta'),
('00000000-0000-0000-0001-000000000028', 'kanban', 'gestionar_tablero', NULL, 'kanban:gestionar_tablero'),

('00000000-0000-0000-0001-000000000029', 'calendario', 'ver', NULL, 'calendario:ver'),
('00000000-0000-0000-0001-000000000030', 'calendario', 'crear_evento', NULL, 'calendario:crear_evento'),
('00000000-0000-0000-0001-000000000031', 'calendario', 'editar_evento', NULL, 'calendario:editar_evento'),
('00000000-0000-0000-0001-000000000032', 'calendario', 'gestionar_recursos', NULL, 'calendario:gestionar_recursos'),

('00000000-0000-0000-0001-000000000033', 'usuarios', 'ver', NULL, 'usuarios:ver'),
('00000000-0000-0000-0001-000000000034', 'usuarios', 'crear', NULL, 'usuarios:crear'),
('00000000-0000-0000-0001-000000000035', 'usuarios', 'editar', NULL, 'usuarios:editar'),
('00000000-0000-0000-0001-000000000036', 'usuarios', 'gestionar_roles', NULL, 'usuarios:gestionar_roles'),

('00000000-0000-0000-0001-000000000037', 'sistema', 'configuracion', NULL, 'sistema:configuracion'),
('00000000-0000-0000-0001-000000000038', 'sistema', 'backup', NULL, 'sistema:backup'),
('00000000-0000-0000-0001-000000000039', 'sistema', 'restore', NULL, 'sistema:restore');

-- 3. Asignar todos los permisos al rol Administrador
INSERT OR IGNORE INTO rol_permisos (id, rol_id, permiso_id)
SELECT ('00000000-0000-0002-' || SUBSTR(id, 20)), '00000000-0000-0000-0000-000000000010', id
FROM permisos;

-- 4. Usuario inicial Super Admin (password: admin123)
INSERT OR IGNORE INTO usuarios
(id, email, nombre_completo, password_hash, activo, requiere_2fa, created_at, row_version, is_deleted)
VALUES
('00000000-0000-0000-0000-000000000999', 'admin@certaro.local', 'Super Administrador', '$argon2id$v=19$m=19456,t=2,p=1$eDR2cW54aGNrZDRzcnkzNQ$e0sJ903f7vI5oIff01f+UqL6rZf4iKqA8a7dI23qQ3g', 1, 0, '2026-01-01T00:00:00.000Z', X'0000000000000001', 0);

-- 5. Vincular Super Admin con Rol Administrador
INSERT OR IGNORE INTO usuario_roles (id, usuario_id, rol_id, created_at, row_version, is_deleted)
VALUES
('00000000-0000-0000-0000-000000000998', '00000000-0000-0000-0000-000000000999', '00000000-0000-0000-0000-000000000010', '2026-01-01T00:00:00.000Z', X'0000000000000001', 0);
"#;

const DROP: &str = r#"
DROP TABLE IF EXISTS auth_externo;
DROP TABLE IF EXISTS sesiones;
DROP TABLE IF EXISTS rol_permisos;
DROP TABLE IF EXISTS usuario_roles;
DROP TABLE IF EXISTS permisos;
DROP TABLE IF EXISTS roles;
DROP TABLE IF EXISTS usuarios;
"#;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        match backend {
            sea_orm::DatabaseBackend::Sqlite => {
                manager.get_connection().execute_unprepared(SCHEMA).await?;
                manager.get_connection().execute_unprepared(SEED).await?;
            }
            sea_orm::DatabaseBackend::Postgres => {
                let schema_pg = SCHEMA
                    .replace("BLOB NOT NULL DEFAULT X'0000000000000001'", "BYTEA NOT NULL DEFAULT '\\x0000000000000001'::bytea")
                    .replace("BLOB", "BYTEA");
                let seed_pg = SEED
                    .replace("X'0000000000000001'", "'\\x0000000000000001'::bytea")
                    .replace("INSERT OR IGNORE INTO", "INSERT INTO")
                    .replace(";\n", " ON CONFLICT DO NOTHING;\n");
                manager.get_connection().execute_unprepared(&schema_pg).await?;
                manager.get_connection().execute_unprepared(&seed_pg).await?;
            }
            sea_orm::DatabaseBackend::MySql => {
                let schema_mysql = SCHEMA
                    .replace("TEXT    NOT NULL PRIMARY KEY", "VARCHAR(36) NOT NULL PRIMARY KEY")
                    .replace("BLOB NOT NULL DEFAULT X'0000000000000001'", "BINARY(8) NOT NULL DEFAULT 0x0000000000000001")
                    .replace("WHERE is_deleted = 0", "")
                    .replace("usuario_id   TEXT    NOT NULL", "usuario_id VARCHAR(36) NOT NULL")
                    .replace("rol_id       TEXT    NOT NULL", "rol_id VARCHAR(36) NOT NULL")
                    .replace("permiso_id   TEXT    NOT NULL", "permiso_id VARCHAR(36) NOT NULL")
                    .replace("email             TEXT    NOT NULL", "email VARCHAR(255) NOT NULL")
                    .replace("nombre       TEXT    NOT NULL", "nombre VARCHAR(100) NOT NULL")
                    .replace("clave        TEXT    NOT NULL", "clave VARCHAR(255) NOT NULL")
                    .replace("token_hash   TEXT    NOT NULL", "token_hash VARCHAR(64) NOT NULL")
                    .replace("proveedor          TEXT    NOT NULL", "proveedor VARCHAR(50) NOT NULL")
                    .replace("proveedor_user_id  TEXT    NOT NULL", "proveedor_user_id VARCHAR(255) NOT NULL");
                let seed_mysql = SEED
                    .replace("X'0000000000000001'", "0x0000000000000001")
                    .replace("INSERT OR IGNORE INTO", "INSERT IGNORE INTO");
                manager.get_connection().execute_unprepared(&schema_mysql).await?;
                manager.get_connection().execute_unprepared(&seed_mysql).await?;
            }
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(DROP).await?;
        Ok(())
    }
}
