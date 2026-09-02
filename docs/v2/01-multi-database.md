# Certaro v2 - Arquitectura Multi-Base de Datos

## 1. Visión General

Certaro v2 introduce soporte nativo y unificado para tres motores de base de datos relacionales:
- **SQLite** (por defecto para despliegue desktop autónomo sin servidor).
- **PostgreSQL** (para servidores corporativos, despliegues multiusuario y contenedores Docker).
- **MySQL / MariaDB** (para empresas que estandarizan sobre infraestructura LAMP/LEMP).

## 2. Abstracción y Dialectos

La capa de persistencia está construida sobre **SeaORM** con compatibilidad multi-dialecto en todas las consultas y migraciones:

- `DatabaseConnection`: Conexión asíncrona abstracta que resuelve `DatabaseBackend::Sqlite`, `DatabaseBackend::Postgres` o `DatabaseBackend::MySql`.
- `DbPool`: Pool de conexiones con timeout configurable, reconexión automática y tamaño máximo adaptado al motor.
- Tipos de datos portables:
  - Identificadores primarios: `Uuid` normalizado a texto/char(36) o UUID nativo según el motor.
  - Concurrencia optimista: `row_version` almacenado de forma consistente (`blob` / `varbinary` / `bytea`).
  - Fechas e Instantes: Formato ISO 8601 UTC en almacenamiento con conversores bidireccionales en mappers.
  - Dinero y Números: Enteros escalados fijos a 4 decimales (`Money` y `Decimal4`) para evitar imprecisiones de coma flotante.

## 3. Configuración y Despliegue

La selección del motor se realiza mediante variables de entorno o archivo de configuración `config.toml`:

```toml
[database]
# "sqlite" | "postgres" | "mysql"
engine = "sqlite"

# Para SQLite:
path = "certaro.db"

# Para PostgreSQL / MySQL:
url = "postgres://certaro_user:secret@localhost:5432/certaro"
# url = "mysql://certaro_user:secret@localhost:3306/certaro"
max_connections = 10
timeout_seconds = 30
```

## 4. Migraciones Automáticas

Al inicializar la aplicación, `certaro-migration` ejecuta las 4 migraciones automáticas respetando las diferencias de sintaxis DDL entre SQLite, PostgreSQL y MySQL:
1. `m20260830_000001_create_initial_tables`: 28 tablas base de gestión operativa y comercial.
2. `m20260902_000002_create_auth_tables`: 7 tablas de seguridad y RBAC.
3. `m20260902_000003_create_kanban_tables`: 7 tablas del módulo Kanban y presets.
4. `m20260902_000004_create_calendar_tables`: 4 tablas del módulo Calendario y grupos iniciales.
