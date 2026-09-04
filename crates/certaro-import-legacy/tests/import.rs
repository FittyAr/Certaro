//! Integration tests for eo-import-legacy. See `docs/15-migracion-de-datos.md` §8.

#[path = "import/schema.rs"]
mod schema;
#[path = "import/common.rs"]
mod common;
#[path = "import/ejecucion_base.rs"]
mod ejecucion_base;
#[path = "import/fechas_valores.rs"]
mod fechas_valores;
#[path = "import/derivaciones_control.rs"]
mod derivaciones_control;
