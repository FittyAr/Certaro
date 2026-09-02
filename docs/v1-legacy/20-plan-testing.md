# 20 · Plan sistemático de testing y corrección

> Complementa `17-testing.md`. Este documento es operativo: cada botón, cada caja de texto y cada flujo debe ser verificable. Si el código y este plan discrepan, este documento manda hasta que se corrija el código.

## 1. Objetivo

Eliminar errores tontos (no poder editar valores, registros que no persisten, validaciones que no se muestran) asegurando que **cada control sea funcional** en modo Tauri real y en preview web (`src/api/client.ts` mock).

Reglas no negociables (AGENTS.md):
- Cero hardcoding, i18n vía `t()`.
- `Money` es `Money(i64)` escala 4, nunca `f64`.
- Fechas en UTC, solo render local.
- Cada fórmula y validación con test.

## 2. Estrategia

Pirámide en 3 capas:
1. **Unitario** (`cargo test`, `vitest`): dominio, `parseMoneyInput`, validators mock.
2. **Componente** (`vue/test-utils` + `jsdom`): `MoneyInput`, `DataGrid`, `CrudDrawer`, `ListState`.
3. **Manual sistemático con navegador** (MCP Docker `host.docker.internal:1420`): cada ruta, cada botón, cada input.

Cada fix sigue TDD: test que falla → fix mínimo → `pnpm typecheck && pnpm lint && pnpm test && cargo test`.

## 3. Matriz por módulo (qué se prueba en cada control)

| Módulo | Ruta | Controles clave | Prueba por control |
|---|---|---|---|
| Movimientos | `/movimientos` | `Concepto` InputText, `Monto` MoneyInput, `Cantidad` InputNumber, `Tipo` Select, `Categoría` Select+filter, `Moneda` Select, `Cotización` MoneyInput condicional, filtros `concepto/tipo/categoria/fechaDesde/fechaHasta`, `Exportar`, `Nuevo`/`Editar`/`Borrar` | Alta vacía → validación por campo; edición `12345,67` y `12345.67` → persiste `12.345,67`; validación `monto>0`, `categoriaRequired`; paginador `1 a 9 de 9`; resumen ingresos/gastos/balance |
| Clientes | `/clientes` | `nombre`, `cuit`, `condicionIva`, `direccion`, `telefono`, `email`, grilla contactos (`etiqueta`, `email`, `nombre`, `telefono`, `esPrincipal`), filtros `texto/condicionIva/soloConDeuda` | Alta sin nombre → `VALIDATION nombre`; email inválido → `email`; edición persiste tras reload; borrado con obras/facturas → `DEPENDENCY_IN_USE` |
| Obras | `/obras` | `numero` autoincremental, `nombre`, `clienteId` Select, `direccion`, `localidad`, `estado` Select, filtros | Nombre vacío / cliente vacío → validación; edición persiste; borrado con trabajos → bloqueado |
| Trabajos | `/trabajos` | `obraId` Select filtrado por cliente, `descripcion` Textarea, `fechaInicio`/`fechaFin` DateInput, `presupuesto` MoneyInput, filtros | Descripción/obra requeridos; presupuesto acepta ambos separadores; cambio cliente limpia obra incoherente |
| Facturas | `/facturas` | `numero`, `fecha`, `clienteId`, `fechaVencimiento`, `subtotal`/`iva`/`total` MoneyInput (total = subtotal+iva), `observaciones`, pagos `fecha/monto/medioPago` | Total recalculado; vencimiento < emisión → validación; pago parcial cambia estado |
| Empleados | `/empleados` | `nombre`, `dni`, `cargo`, `sueldoBase`, `tarifaDiaria`, `pagoFrecuencia`, multiplicadores, `activo` Switch | Nombre requerido; tarifa >0; edición persiste |
| Categorías | `/admin/categorias` | `nombre`, `categoriaPadreId`, `colorHex`, `descripcion`, filtro `soloRaiz` | Nombre requerido; padre no puede ser sí mismo |
| Tipos movimiento | `/admin/tipos-movimiento` | `nombre`, `descripcion`, `esIngreso` Switch | Nombre requerido |
| Feriados | `/admin/feriados` | `anio` InputNumber, `fecha` DateInput, `nombre` InputText, `Sincronizar`, `Agregar`, `Borrar` por fila | Fecha y nombre requeridos; `feriados_add` mock con `validateMockFeriado`; `feriados_delete` por fecha |
| Asistencia | `/asistencia` | `desde`/`hasta` DateInput, grilla celdas clicables, diálogo `empleadoId`/`desde`/`hasta`/`tipoJornada`/`soloDiasHabiles` | Clic cicla jornada; carga masiva persiste |
| Liquidaciones | `/liquidaciones` | Filtros `empleadoId/fechaDesde/fechaHasta/soloSinPdf`, wizard 3 pasos | Paso1 requiere selección; paso3 confirma lote |
| Reportes | `/reportes` | `fechaDesde`/`fechaHasta`, `Exportar` con formatos PDF/XLSX/DOCX/CSV/JSON | Exportar respeta filtro activo, no solo página |
| Configuración | `/configuracion` | 6 tabs, `Aplicar` por sección | Guardado persiste en `mockConfig` / `config_set` |

## 4. Correcciones ya aplicadas (referencia exacta)

- `src/components/domain/MoneyInput.vue:1-129` → `InputText` + `src/lib/moneyInput.ts:1-35` `parseMoneyInput` (soporta `,` y `.`, `focused` guard).
- `src/api/client.ts:528-610` validators `validateMock*` + `mockAudit()` y casos `feriados_add`/`feriados_delete` (`:768-790`).
- `src/components/domain/DataGrid.vue:34-41` plantilla `t('\''General.PageReport'\'', {first:'\''{first}'\''...})`.
- `src/components/ui/button/Button.vue:56` `v-bind="$attrs"`.
- `src/views/movimientos/MovimientosView.vue:224,249,261` y otras 6 vistas con `aria-label`.

## 5. Checklist manual (antes de cada release)

1. Arranque frío sin DB → migraciones y dashboard.
2. Sin conexión → cotización no disponible, sin error.
3. Recorrer 15 rutas → ninguna en blanco.
4. Alta/edición/borrado en cada CRUD (incluye validación por campo y recarga).
5. Temas claro/oscuro sin texto ilegible.
6. Inglés sin etiquetas en español.
7. 1024×768 sin scroll horizontal.
8. Exportar cada reporte y abrir archivo.
9. Backup crear/restaurar.
10. Atajos `Ctrl+N`, `Ctrl+K`, `Escape`.
11. Liquidación completa vs cálculo manual.
12. Certificado avance parcial vs acumulado.

## 6. Criterio de terminado por módulo

Un módulo no está terminado hasta: validación por campo visible, persistencia tras reload, borrado lógico con `rowVersion`, y al menos 1 test de regresión que falle sin el fix.

## 7. Próximos pasos

- Añadir `src/composables/__tests__/useCrudDrawer.spec.ts` y `DataGrid.spec.ts`.
- Ampliar `src/api/__tests__/client.spec.ts` con casos `feriados` y `liquidaciones`.
- Reemplazar 5 `PlaceholderView` (`/clientes/:id`, `/obras/:id`, etc.) por vistas reales.
## 8. Hallazgos E2E 2026-09-01 (verificado vía MCP Docker)

- **Obra → Trabajo**: `src/views/trabajos/TrabajosView.vue:393-416` creación con `Descripción` + `Obra` + `Presupuesto $` cierra dialog y persiste fila `Trabajo QA E2E` (1).
- **Trabajo → Orden**: `src/views/ordenes/OrdenesView.vue:157-235` dialog `Nueva orden` con `Título`/`Fecha`/`Ajuste UOCRA`/`Otros descuentos`/`Ítems` (`Descripción`/`Unidad`/`Cantidad`/`Precio unitario`). Creación con `Cantidad=10` y `Precio=10000` no persiste en mock actual (0 filas) — deuda: mock `ordenes_add` requiere `trabajoId` y validación de `Cantidad>0` no propagada.
- **Orden → Certificado**: ruta `/trabajos/:trabajoId/ordenes` y `/ordenes/:ordenId` accesibles, tablas vacías correctamente muestran *Este trabajo no tiene órdenes*.
- **Liquidaciones wizard**: `src/views/liquidaciones/LiquidacionesView.vue:270-308` paso 1 `Calcular` permanece `disabled` sin empleados seleccionados (comportamiento esperado por `store.suggest` que requiere `empleadoIds`).
- **Factura → Pago**: `src/views/facturas/FacturasView.vue:134-149` botón `Pagos` abre dialog, mock `pagos_factura_registrar` operativo.

