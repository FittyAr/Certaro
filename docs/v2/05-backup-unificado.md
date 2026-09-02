# Certaro v2 - Sistema de Backup Unificado

## 1. Visión General

Certaro v2 cuenta con un sistema de respaldo atómico y portable que soporta las 38 tablas de negocio a través de cualquier dialecto de base de datos (SQLite, PostgreSQL y MySQL).

## 2. Formato JSON Portable con Orden Topológico

El dump JSON (`Documento`) almacena las filas en un orden topológico estricto para garantizar que nunca se violen claves foráneas durante la restauración:
1. `roles`, `permisos`, `usuarios`, `usuario_roles`, `rol_permisos`, `sesiones`, `auth_externo`
2. `tipos_movimiento`, `categorias`, `tipos_concepto_pago`
3. `clientes`, `cliente_contactos`, `proyectos`, `trabajos`, `facturas`, `pagos_factura`, `ordenes_trabajo`, `orden_trabajo_items`
4. `certificados`, `certificado_items`, `empleados`, `asistencias_empleado`, `liquidaciones`, `liquidacion_adelantos`
5. `movimientos`, `adjuntos`, `feriados`
6. `kanban_tableros`, `kanban_columnas`, `kanban_tarjetas`, `kanban_etiquetas`, `kanban_tarjeta_etiquetas`, `kanban_tarjeta_checklist`, `kanban_tarjeta_asignados`
7. `calendario_grupos_recurso`, `calendario_recursos`, `calendario_eventos`, `calendario_evento_recursos`

## 3. Integridad y Restauración

- **Validación de Integridad**: Cada backup incluye versión de formato (`FORMAT_VERSION = 2`), fecha de exportación, versión de la aplicación y la última migración aplicada.
- **Transaccionalidad**: La restauración vacía e inserta dentro de una transacción atómica; si una tabla falla, se cancela y se revierte el estado completo.
- **Rotación de Backups Locales**: La política de retención conserva automáticamente los backups más recientes y descarta los antiguos según la configuración del sistema.
