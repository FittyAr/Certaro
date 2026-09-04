# Fase 9: Correcciones Críticas de Lógica Numérica, Imputación Financiera y Flujo de Obra

## 1. Contexto y Justificación

A partir de la exhaustiva auditoría del flujo de trabajo de Certaro realizada desde la perspectiva de un contratista / usuario de pyme (instalaciones eléctricas, climatización, construcción y servicios a campo), se identificaron inconsistencias matemáticas de alto impacto, desconexiones de imputación de costos y limitaciones en la ergonomía de uso:

1. **La "Trampa" de Cálculo en Ajuste UOCRA:** En el dominio (`OrdenTrabajo::total_neto` y `certificados.rs`), el ajuste UOCRA se deduce (`total_certificado - ajuste_uocra`). En la práctica comercial argentina, un ajuste de escala salarial UOCRA es una **redeterminación por inflación / incremento de costo**, lo que representa un **adicional positivo** para el contratista. Restarlo genera una pérdida directa del monto compensatorio facturado al cliente.
2. **Mezcla Heterogénea de Monedas en Caja:** La consulta SQL de resumen (`SUM(monto * cantidad)`) totaliza conjuntamente transacciones en Pesos (ARS) y Dólares (USD) 1 a 1 sin aplicar la cotización registrada, arrojando balances irreales en tesorería y en el Dashboard.
3. **Pérdida de Imputación de Proyectos en Movimientos:** En `MovimientosView.vue`, si un usuario selecciona un Proyecto que tiene múltiples trabajos (o ninguno) y deja el selector de Trabajo vacío, el movimiento se almacena con `trabajo_id = NULL`. Dado que la tabla `movimientos` carece de `proyecto_id`, el gasto o ingreso se "desvanece" de la caja del proyecto (`ProyectoCajaView`) y no se descuenta de su rentabilidad.
4. **Falta de Edición en Planilla de Obra (`OrdenDetalleView`):** La vista de trabajo de una Orden de Trabajo no cuenta con botones para agregar o modificar líneas de cómputo; el usuario se ve obligado a retroceder y navegar al listado de órdenes para abrir el diálogo de edición.
5. **Efecto "Huevo y Gallina" en Asistencia:** El filtro por Proyecto en `AsistenciaView.vue` oculta a los operarios que no tienen marcas previas en la obra seleccionada, impidiendo asignar asistencia inicial a la cuadrilla.
6. **Cálculo Financiero Truncado en Ficha de Proyecto:** En `ProyectoDetalleView.vue`, los totales de ingresos y gastos de la obra se calculan iterando sobre la primera página de 100 movimientos en vez de utilizar el `resumen` de base de datos, falseando los números en obras grandes.

---

## 2. Especificación de Tareas

### Tarea 9.1: Corrección de Signo y Claridad en Ajuste UOCRA de Certificados
* **Problema:** La fórmula `total_neto = total_certificado - ajuste_uocra - otros_descuentos` descuenta el ajuste salarial en lugar de adicionarlo.
* **Solución Técnica:**
  - En `crates/certaro-domain/src/entities/orden_trabajo.rs`: Modificar `total_neto` para que adicione el `ajuste_uocra` cuando sea positivo:
    $$\text{Total Neto} = (\text{Total Certificado} + \text{Ajuste UOCRA}) - \text{Otros Descuentos}$$
  - En `crates/certaro-application/src/use_cases/certificados.rs`: Actualizar el cálculo de emisión:
    ```rust
    let neto_previo = total_certificado.checked_add(ajuste_uocra)?;
    let otros_descuentos = descuento_a_aplicar.min(neto_previo);
    let total_neto = neto_previo.checked_sub(otros_descuentos)?;
    ```
  - En `src/views/ordenes/OrdenDetalleView.vue`: Actualizar la previsualización del modal de emisión para reflejar la suma del adicional UOCRA y etiquetar el campo como *"Adicional / Ajuste UOCRA (+)"*.
  - Actualizar tests unitarios de dominio y casos de uso en Rust.
* **Criterio de Aceptación:** Al certificar una orden con 10% de ajuste UOCRA sobre $100.000, el monto neto resultante es $110.000 (menos otros descuentos), protegiendo los ingresos del contratista.

### Tarea 9.2: Selector y Filtro de Moneda en Libro de Movimientos
* **Problema:** El balance general de tesorería y el resumen de movimientos totalizan indiscriminadamente importes en ARS y USD.
* **Solución Técnica:**
  - En `src/views/movimientos/MovimientosView.vue`: Incorporar un selector de `Moneda` en la `FilterBar` con opciones: *"Todas"*, *"Pesos (ARS)"* y *"Dólares (USD)"*.
  - Conectar el selector con `table.filter.value.moneda`.
  - Asegurar que al filtrar por una divisa específica, el banner de totales (`resumen`) represente con exactitud la suma de esa moneda.
* **Criterio de Aceptación:** El usuario puede aislar y totalizar sus movimientos en USD y en ARS de forma independiente, evitando la suma distorsionada de ambas monedas.

### Tarea 9.3: Prevención de Movimientos Huérfanos al Imputar Proyecto
* **Problema:** Al registrar un gasto o ingreso y seleccionar un Proyecto en el formulario, si no se selecciona explícitamente un Trabajo, se guarda `trabajo_id = null`, perdiéndose la imputación al proyecto en base de datos.
* **Solución Técnica:**
  - En `src/views/movimientos/MovimientosView.vue`:
    - Al dispararse `onProyectoChange()`, si el proyecto posee uno o más trabajos disponibles y el usuario no seleccionó uno, autoseleccionar por defecto el primer trabajo activo.
    - Si el proyecto no cuenta con trabajos aún, alertar visualmente al usuario indicando que el movimiento requiere un trabajo asociado para computarse en la caja y rentabilidad de la obra.
* **Criterio de Aceptación:** Ningún movimiento asignado a un proyecto queda con `trabajo_id = null` si el proyecto cuenta con trabajos activos; el gasto/ingreso se refleja inmediatamente en `ProyectoCajaView`.

### Tarea 9.4: Edición Directa de Planilla de Cómputo en `OrdenDetalleView`
* **Problema:** En `OrdenDetalleView.vue` no es posible modificar ítems, agregar tareas omitidas o ajustar precios sin abandonar la pantalla y navegar de regreso al listado general de órdenes.
* **Solución Técnica:**
  - Incorporar un botón *"Editar Planilla / Ítems"* en el `PageHeader` de `OrdenDetalleView.vue`.
  - Integrar el modal de edición de orden (`EditorOrdenModal` o diálogo análogo) permitiendo agregar nuevas líneas, corregir precios o descripciones, conservando la restricción de seguridad que impide eliminar o reducir líneas que ya cuenten con certificaciones aprobadas.
* **Criterio de Aceptación:** El usuario puede corregir el presupuesto o cómputo de la orden directamente desde su pantalla de detalle sin dar rodeos por la navegación secundaria.

### Tarea 9.5: Corrección del Filtro de Cuadrilla en Asistencia
* **Problema:** Al activar `filtroProyectoId` en `AsistenciaView.vue`, se filtran las filas ocultando a todos los operarios que no tengan celdas registradas previamente en dicho proyecto, impidiendo la asignación inicial de marcas a la cuadrilla.
* **Solución Técnica:**
  - En `AsistenciaView.vue`, agregar un control o toggle: *"Solo asignados"* vs *"Todos los operarios activos"*.
  - Cuando se selecciona un proyecto para cargar asistencia, permitir visualizar a toda la nómina activa para poder marcar la jornada imputada a los trabajos de ese proyecto mediante el diálogo de carga masiva o clic individual.
* **Criterio de Aceptación:** El usuario puede filtrar por obra y aun así visualizar y asignar marcas de jornada a cualquier operario activo de su cuadrilla.

### Tarea 9.6: Consistencia de Totales en Caja de Proyecto (`ProyectoDetalleView`)
* **Problema:** En `ProyectoDetalleView.vue`, los indicadores de *Ingresos*, *Gastos* y *Balance Neto* de la pestaña de tesorería se calculan reduciendo el array local de la primera página (máx. 100 movimientos), truncando los valores reales en obras con alto movimiento.
* **Solución Técnica:**
  - En `ProyectoDetalleView.vue`, al invocar `movimientosStore.fetchPaged()`, capturar y enlazar `res.resumen` (igual que en `ProyectoCajaView.vue`) para mostrar los totales agregados de base de datos sin importar la paginación.
* **Criterio de Aceptación:** La ficha integral de obra refleja los importes matemáticos exactos independientemente de la cantidad de movimientos registrados.
