# Fase 6: Consolidación de Flujos, Hub de Obras y Ergonomía de Usuario

Este documento detalla el alcance, especificación técnica y criterios de aceptación para la **Fase 6 del Plan de Mejoras del Flujo de Trabajo**. Esta fase resuelve las desconexiones contextuales identificadas durante la auditoría de usuario real en el ciclo de vida de obras, cuentas corrientes, órdenes de trabajo, calendario y registro de personal.

---

## 1. Motivación y Diagnóstico Operativo

Tras auditar el recorrido del usuario de una pyme de proyectos/obras, se detectaron 5 puntos donde el usuario se ve forzado a salir de su contexto de trabajo actual para realizar operaciones elementales:

1. **En la Orden de Trabajo (`OrdenDetalleView`):** Se puede emitir un certificado de avance, pero no se pueden ver ni descargar los certificados previamente emitidos para esa misma orden.
2. **En la Ficha de Proyecto (`ProyectoDetalleView`):** La vista es una ficha estática; para ver los trabajos o la caja de la obra el usuario debe cambiar de pantalla a rutas separadas, perdiendo la visión integral de la obra.
3. **En el Calendario (`CalendarioView`):** Se utilizan llamadas bloqueantes del navegador (`confirm()` y `alert()`), rompiendo la fluidez y el sistema de diseño PrimeVue/Tauri.
4. **En el Registro de Asistencia (`AsistenciaView`):** Todos los empleados aparecen mezclados en la grilla mensual sin posibilidad de filtrar por cuadrilla o por proyecto/obra asignada.
5. **En la Cuenta Corriente del Cliente (`CuentaCorrienteView`):** Se visualizan las facturas adeudadas y su antigüedad (0-30, 31-60, etc.), pero no hay forma de registrar la cobranza directamente sin navegar a Facturas y buscar el comprobante manualmente.

---

## 2. Especificación Técnica de las Tareas

### Tarea 6.1: Sección de Certificados Emitidos en `OrdenDetalleView`
- **Archivos involucrados:** `src/views/ordenes/OrdenDetalleView.vue`, `src/stores/useCertificadosStore.ts`, `src/locales/es.json`.
- **Diseño del flujo:**
  - En `OrdenDetalleView.vue`, luego de la tabla de ítems presupuestados y del bloque de totales, incorporar una sección o tarjeta: **"Certificados de Avance Emitidos"**.
  - Cargar los certificados vinculados a la orden (`store.fetchPaged` filtrando por `ordenTrabajoId: ordenId`).
  - Columnas: Número de certificado, Fecha, Total Neto, Observaciones y Acciones (Ver detalle con router-link, Exportar PDF directo).
  - Si no hay certificados emitidos aún, mostrar un estado vacío con invitación a emitir el primer certificado.
- **Criterio de aceptación:** Al ingresar a una orden de trabajo, el usuario ve la lista histórica de todos los certificados generados para esa orden y puede descargarlos en PDF con un solo clic.

---

### Tarea 6.2: Hub Integral de Obra con Pestañas en `ProyectoDetalleView`
- **Archivos involucrados:** `src/views/proyectos/ProyectoDetalleView.vue`, `src/views/proyectos/components/NuevoTrabajoModal.vue`.
- **Diseño del flujo:**
  - Transformar `ProyectoDetalleView.vue` en una vista unificada utilizando `Tabs / TabList / Tab / TabPanels / TabPanel` de PrimeVue:
    1. **Pestaña "Información General":** Datos del cliente, estado, dirección, localidad, fechas y accesos rápidos.
    2. **Pestaña "Trabajos":** Grilla completa de trabajos de la obra con su estado y presupuesto, con botón "+ Nuevo Trabajo" que abre `NuevoTrabajoModal.vue` y enlace a las órdenes de trabajo.
    3. **Pestaña "Caja y Rentabilidad":** Tarjetas KPI (Ingresos, Gastos, Balance Neto del Proyecto) y listado de movimientos financieros imputados al proyecto, con botón de registro rápido de gasto.
- **Criterio de aceptación:** El encargado de obra o director comercial puede gestionar completamente el proyecto (datos, trabajos, caja) sin cambiar de ruta ni perder el contexto de la obra.

---

### Tarea 6.3: Estandarización de Diálogos y Notificaciones en `CalendarioView`
- **Archivos involucrados:** `src/views/calendario/CalendarioView.vue`.
- **Diseño del flujo:**
  - Eliminar todas las ocurrencias de `window.alert(...)` y `window.confirm(...)`.
  - Utilizar `useConfirmDelete` o `useConfirm` de PrimeVue para la confirmación de eliminación de eventos.
  - Utilizar el composable `useApiError()` con `notify(err)` y toasts para notificar errores de guardado o confirmaciones de sincronización de recursos.
- **Criterio de aceptación:** Ninguna acción del calendario dispara ventanas modales nativas del sistema operativo ni bloquea el hilo de renderizado; todas las interacciones respetan el sistema de diseño.

---

### Tarea 6.4: Filtro por Frente de Obra / Proyecto en `AsistenciaView`
- **Archivos involucrados:** `src/views/asistencia/AsistenciaView.vue`, `src/stores/useProyectosStore.ts`.
- **Diseño del flujo:**
  - Añadir en la barra superior de filtros un selector de **Proyecto / Frente de Obra** (opcional, con opción "Todos").
  - Si se selecciona un proyecto, filtrar las filas de empleados a aquellos asignados a tareas/trabajos de dicho proyecto o filtrar la carga masiva asociando automáticamente el `trabajoId`.
- **Criterio de aceptación:** El usuario puede acotar la grilla mensual para visualizar únicamente a los operarios asignados al proyecto de interés.

---

### Tarea 6.5: Cobro Rápido de Facturas desde `CuentaCorrienteView`
- **Archivos involucrados:** `src/views/comercial/CuentaCorrienteView.vue`, `src/views/facturas/FacturasView.vue`.
- **Diseño del flujo:**
  - En la tabla de facturas impagas de `CuentaCorrienteView.vue`, agregar un botón de acción en cada fila: **"Registrar Cobro"** (icono `wallet` o `check-circle`).
  - Abrir un diálogo de cobro (monto pendiente sugerido, fecha, medio de pago y toggle para asentar en caja).
  - Al confirmar el cobro, recargar tanto el extracto de cuenta corriente como la antigüedad de deuda y el saldo global del cliente.
- **Criterio de aceptación:** El responsable de cobranzas puede cancelar o amortizar una factura vencida directamente desde el informe de cuenta corriente con un solo clic.
