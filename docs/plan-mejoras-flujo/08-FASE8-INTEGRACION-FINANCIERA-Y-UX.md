# Fase 8: Integración Financiera, Consistencia de Cálculos y Ergonomía UX

## 1. Contexto y Justificación

A partir de la auditoría integral de flujos de trabajo y cálculos realizada sobre Certaro desde la perspectiva de un usuario real (dueño de pyme, jefe de obra y administrativo contable), se identificaron desconexiones operativas clave entre los módulos comercial, financiero, de obras y de personal:

1. **Rentabilidad de Obra Desconectada:** El cobro de facturas originadas en certificados de avance generaba movimientos de caja sin imputar el `trabajoId` de la obra, provocando que la rentabilidad de las obras en el Dashboard y en la Ficha de Proyecto mostrara ingresos en $0 y márgenes en -100%.
2. **Dinero Fantasma por Sueldos:** Al liquidar sueldos masivamente a través del asistente, no existía opción para asentar el egreso neto en el Libro de Caja, dejando los saldos de tesorería sobrestimados.
3. **Facturación Duplicada de Certificados:** Un certificado de avance podía ser facturado repetidas veces sin control de estado ni trazabilidad.
4. **Navegación y Ergonomía Rota:** Vínculos rotos entre Cuenta Corriente y Facturas (`query.id`), ceguera de imputación a obras en la grilla de Movimientos, pérdida de frente de obra al ciclar asistencias y falta de selector de obra en eventos de Calendario.
5. **Permisos Omitidos en Rutas:** Inconsistencia de nombre de propiedad en las guardas de Kanban y Calendario.

Esta fase detalla las tareas técnicas y criterios de aceptación para resolver íntegramente estas deficiencias.

---

## 2. Especificación de Tareas

### Tarea 8.1: Imputación de Cobranzas a Obra y Rentabilidad Real
* **Problema:** Al cobrar una factura con *"Registrar movimiento en caja"*, se guardaba `trabajoId: null`. La consulta SQL de rentabilidad de proyectos (`rentabilidad_proyectos`) y la Caja de Proyecto solo computan ingresos con `trabajo_id` asociado a un trabajo de la obra, arrojando $0 ingresos para proyectos facturados y cobrados.
* **Solución Técnica:**
  - En `FacturasView.vue`, al crear una factura a partir de un certificado de avance (o manual), almacenar o inferir la vinculación al primer trabajo o trabajo principal de la orden/certificado (`trabajoId` o referencia en metadata).
  - Al ejecutar `registrarPago()` con asiento en caja, asignar el `trabajoId` vinculado al nuevo movimiento generado.
  - Asegurar que `CuentaCorrienteView.vue` al cobrar una factura preserve la imputación al trabajo si está presente en la factura.
* **Criterio de Aceptación:** Al cobrar una factura vinculada a una obra, el movimiento de ingreso en el Libro de Caja queda imputado al trabajo/obra correspondiente, impactando de inmediato en el balance de `ProyectoCajaView` y en los rankings de rentabilidad del Dashboard.

### Tarea 8.2: Asiento Opcional de Egreso en Caja al Liquidar Sueldos
* **Problema:** El asistente de liquidaciones (`LiquidacionesView.vue`) crea los comprobantes de nómina y recibos PDF pero no genera el movimiento de egreso de fondos en caja, distorsionando el balance de tesorería real.
* **Solución Técnica:**
  - En el paso 3 (Confirmación) de `LiquidacionesView.vue`, incorporar un bloque opcional:
    - Checkbox *"Registrar egreso de sueldos en caja"* (marcado por defecto).
    - Selector de medio de pago / cuenta (Efectivo, Transferencia, Cheque).
    - Selector de categoría (preseleccionando "Sueldos y Jornales" si existe, o primera de egresos).
  - Al pulsar *"Confirmar y Liquidar"*, tras invocar `store.createBatch()`, emitir un movimiento de egreso en `movimientosStore.create()` por el monto total neto del lote liquidado, fechado en el día y con concepto descriptivo (ej: *"Liquidación de Sueldos - Lote N operarios - Período DD/MM al DD/MM"*).
* **Criterio de Aceptación:** Al finalizar el asistente de liquidación, el saldo de caja y el dashboard descuentan automáticamente los sueldos netos abonados.

### Tarea 8.3: Trazabilidad y Prevención de Duplicados en Facturación de Certificados
* **Problema:** `CertificadoDetalleView` permite pulsar indefinidamente *"Facturar este Certificado"*. No hay indicación visual en la lista de certificados de si ya fue facturado o está pendiente.
* **Solución Técnica:**
  - En `CertificadoDetalleView.vue`, registrar en el estado local de facturación (o vía prefijo en notas/metadata) el número o fecha de factura generada.
  - Mostrar badge de estado: *"Facturado"* o *"Pendiente de Facturar"*.
  - En el botón de acción, si ya fue facturado, mostrar advertencia antes de re-facturar para evitar duplicaciones involuntarias.
  - En `CertificadosView.vue`, añadir columna/filtro de estado de facturación.
* **Criterio de Aceptación:** El usuario puede distinguir claramente qué certificados ya fueron facturados y cuáles están pendientes, evitando facturaciones duplicadas accidentales.

### Tarea 8.4: Navegación Directa de Facturas desde Cuenta Corriente (`query.id`)
* **Problema:** El botón *"Ver Factura"* en `CuentaCorrienteView.vue` dirige a `/facturas?id=UUID`, pero `FacturasView.vue` ignora el parámetro de consulta `id`, mostrando el listado general en lugar de la factura solicitada.
* **Solución Técnica:**
  - En `FacturasView.vue`, en el hook `onMounted`, inspeccionar `route.query.id`.
  - Si está presente, abrir directamente el diálogo de pagos/detalle de dicha factura (`abrirPagos({ id })`) o aplicar el filtro de texto con el ID/número de factura.
* **Criterio de Aceptación:** Hacer clic en *"Ver Factura"* desde la Cuenta Corriente abre inmediatamente la factura correspondiente.

### Tarea 8.5: Visibilidad y Filtrado de Obra/Cliente en Libro de Movimientos
* **Problema:** `MovimientosView.vue` no muestra a qué Proyecto o Cliente pertenece un gasto o ingreso en su tabla principal, y la barra de filtros no permite filtrar por Obra ni Cliente.
* **Solución Técnica:**
  - En `MovimientosView.vue`, agregar en la `FilterBar`:
    - Selector de **Cliente** (`filtro.clienteId`).
    - Selector de **Proyecto** (`filtro.proyectoId`).
  - En la tabla de datos (`DataGrid`), añadir columna opcional o detalle en concepto con el nombre del Proyecto/Cliente asignado cuando exista.
* **Criterio de Aceptación:** El usuario puede auditar rápidamente a qué obra pertenece cada movimiento y filtrar todos los gastos de una obra específica desde el libro de caja general.

### Tarea 8.6: Preservación de Obra y Resumen de Totales en Asistencia
* **Problema:** Al hacer clic para ciclar la jornada (`store.ciclar`) en una celda de la grilla de asistencia, `useAsistenciaStore.ts` forzaba `trabajoId = null`. Si el operario tenía una obra asignada por rango, un clic borraba su imputación al trabajo y lo hacía desaparecer de la grilla si estaba filtrada por proyecto. Además, no se mostraba el total de días trabajados por fila.
* **Solución Técnica:**
  - En `useAsistenciaStore.ts`, al ejecutar `ciclar(empleadoId, fecha)`, preservar el `trabajoId` preexistente en la celda (`fila.celdas[indice]?.trabajoId ?? null`).
  - En `AsistenciaView.vue`, añadir columnas fijas al final de cada fila con el conteo de días trabajados: días completos (C), medias jornadas (½) y total equivalente para facilitar el control de jornales.
* **Criterio de Aceptación:** Modificar una celda de asistencia no elimina el frente de obra asignado, y el usuario dispone de totales visuales de asistencia por operario.

### Tarea 8.7: Selector de Proyecto en Calendario y Corrección de Guardas de Permisos
* **Problema:**
  - `CalendarioView.vue` no permitía asociar eventos a un Proyecto o Trabajo.
  - En `routes.ts`, `kanban` y `calendario` definían `requiredPermission`, pero el guardia en `guards.ts` evalúa `to.meta.permission`, dejando las rutas desprotegidas.
* **Solución Técnica:**
  - En `routes.ts`: estandarizar `permission: 'kanban:ver'` y `permission: 'calendario:ver'` en lugar de `requiredPermission`.
  - En `CalendarioView.vue`: añadir selector de Proyecto y Trabajo en el formulario modal de *"Nuevo/Editar Evento"*, guardando `trabajoId`.
* **Criterio de Aceptación:** Las rutas de Kanban y Calendario respetan el sistema de permisos, y el usuario puede agendar eventos de obra vinculados a proyectos específicos.

### Tarea 8.8: Enriquecimiento de `TrabajoDetalleView` y Paginación en `ProyectoCajaView`
* **Problema:**
  - `TrabajoDetalleView.vue` era un formulario de solo lectura estático sin interacción.
  - `ProyectoCajaView.vue` limitaba la consulta a 100 movimientos sin paginador y calculaba los totales solo sobre la primera página.
* **Solución Técnica:**
  - En `TrabajoDetalleView.vue`: incorporar tabla integrada de sus Órdenes de Trabajo con botón de emisión rápida y avance.
  - En `ProyectoCajaView.vue`: añadir paginador `DataTable` y utilizar el total del resumen general para las tarjetas financieras.
* **Criterio de Aceptación:** El detalle de trabajo ofrece una gestión operativa real de sus órdenes y la caja del proyecto maneja grandes volúmenes con paginación y números exactos.
