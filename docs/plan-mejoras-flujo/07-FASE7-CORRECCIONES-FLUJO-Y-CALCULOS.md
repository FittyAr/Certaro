# Fase 7: Correcciones Críticas de Flujo de Trabajo, Cálculos Numéricos y Ergonomía

## 1. Contexto y Justificación

Durante la auditoría integral de la aplicación desde la perspectiva de un usuario final y administrador de obra, se detectaron inconsistencias críticas en el flujo operativo, distorsiones en los balances de caja de obras, errores matemáticos acumulativos en la emisión de certificados de avance, penalizaciones involuntarias de recargos en liquidaciones de jornales y desconexiones de navegación.

Esta fase formaliza las especificaciones técnicas y criterios de aceptación para subsanar cada una de estas fallas y consolidar la robustez operativa de Certaro.

---

## 2. Especificación de Tareas

### Tarea 7.1: Filtrado de Movimientos por Proyecto en Caja de Obra
* **Problema:** `ProyectoCajaView.vue` y la pestaña de Caja en `ProyectoDetalleView.vue` mostraban movimientos de toda la empresa porque `MovimientoFiltroDto` en backend carecía del campo `proyectoId`.
* **Solución Técnica:**
  - Agregar `proyecto_id: Option<Uuid>` a `MovimientoFiltro` (`ports/repositories.rs`) y `MovimientoFiltroDto` (`dtos/movimientos.rs`).
  - En `movimiento.rs` (repositorio SeaORM), condicionar la consulta para filtrar movimientos imputados a cualquier trabajo perteneciente al proyecto indicado (`m.trabajo_id IN (SELECT id FROM trabajos WHERE proyecto_id = ? AND is_deleted = 0)`).
  - Tipar `proyectoId?: string` en `useMovimientosStore.ts`.
* **Criterio de Aceptación:** La Caja de Proyecto y la pestaña de Caja del Detalle de Proyecto muestran única y exclusivamente los ingresos y egresos imputados a los trabajos de esa obra en particular, reflejando su balance neto real.

### Tarea 7.2: Corrección del Descuento Recurrente en Certificados de Avance
* **Problema:** En `certificados.rs`, cada emisión parcial de certificado descontaba el valor total de `orden.otros_descuentos` en lugar de prorratearlo o limitar la deducción al saldo restante, cobrando el descuento múltiples veces. Además, el modal de emisión en frontend no previsualizaba las deducciones.
* **Solución Técnica:**
  - En `certificados.rs` (`create`): calcular la deducción proporcional de otros descuentos según la fracción del avance actual sobre el total de la orden, o deducir hasta el tope del monto total pactado sin duplicación.
  - En `OrdenDetalleView.vue`: incorporar en el modal de emisión el desglose previo con el subtotal bruto por ítems certificados, el descuento del Ajuste UOCRA (%), el descuento aplicado de la orden y el Total Neto final que tendrá el certificado.
* **Criterio de Aceptación:** La emisión de certificados parciales nunca deduce más del total de `otros_descuentos` pactado en la orden, y el usuario previsualiza con total exactitud el monto neto antes de confirmar la emisión.

### Tarea 7.3: Preservación de Recargos de Fin de Semana en Liquidaciones
* **Problema:** En `LiquidacionesView.vue`, al modificar manualmente los días trabajados o la tarifa en el paso 2 del wizard, la fórmula de `dtoDe` aplicaba un cálculo plano `dias * tarifa`, eliminando silenciosamente los recargos calculados por sábados (1.5x), domingos (2x) y feriados (2x).
* **Solución Técnica:**
  - En `dtoDe`, al recalcular la base modificada (`dias * tarifa`), preservar y sumar los recargos calculados (`s.desglose.recargos`), de modo que el ajuste de días ordinarios o de tarifa no prive al operario de sus bonificaciones devengadas.
* **Criterio de Aceptación:** Un empleado modificado en días o tarifa dentro del lote conserva íntegramente sus adicionales por sábados, domingos y feriados.

### Tarea 7.4: Reversión de Asientos en Caja al Eliminar Pagos de Factura
* **Problema:** Al cobrar una factura con *"Registrar movimiento en caja"*, se generaba un ingreso. Si el pago de la factura era posteriormente eliminado, el movimiento en caja permanecía como un saldo fantasma.
* **Solución Técnica:**
  - En `facturas.rs` (`borrar_pago`), verificar si existen movimientos activos en el libro de caja vinculados a la factura (`factura_id`) correspondientes al pago eliminado y marcarlos como eliminados (`soft_delete`).
* **Criterio de Aceptación:** Al anular o borrar un pago de factura, el movimiento de caja automático generado se anula coherentemente, manteniendo conciliado el saldo de tesorería.

### Tarea 7.5: Corrección de Zona Horaria en Asientos Automáticos
* **Problema:** En `FacturasView.vue` y `CuentaCorrienteView.vue`, la fecha del pago se convertía a ISO como medianoche UTC (`00:00:00Z`), lo que provocaba que en husos horarios occidentales (UTC-3) el movimiento apareciera fechado en el día anterior a las 21:00 hs.
* **Solución Técnica:**
  - Registrar el movimiento automático utilizando la estampa de tiempo actual del sistema local (`new Date().toISOString()`), evitando que la conversión a UTC desplace la fecha civil a la víspera.
* **Criterio de Aceptación:** Los cobros de facturas se registran en el libro de caja con la fecha del día en que se efectuaron.

### Tarea 7.6: Filtrado Activo de Cuadrilla en Asistencia
* **Problema:** El selector de proyecto en `AsistenciaView.vue` no filtraba las filas de la cuadrilla debido a una computada no operativa (`if (!filtroProyectoId.value) return g; return g;`).
* **Solución Técnica:**
  - Filtrar las filas de operarios en la vista en base al proyecto asignado o frentes de obra activos.
* **Criterio de Aceptación:** Al seleccionar un proyecto en la barra de asistencia, la grilla segmenta la visualización únicamente a los operarios pertinentes.

### Tarea 7.7: Mejoras de Ergonomía, Navegación y Consistencia UI
* **Problemas:**
  - Imposibilidad de navegar a la ficha del cliente (`ClienteDetalleView`) desde el listado principal de clientes.
  - Gastos en "Últimos Movimientos" del Dashboard mostrados sin signo negativo ni color rojo.
  - Onboarding de bienvenida inaccesible si no se dispone de una base de datos antigua.
* **Solución Técnica:**
  - En `ClientesView.vue`: añadir acción y opción de menú contextual para navegar a `ClienteDetalleView.vue`. En `ClienteDetalleView.vue`, permitir editar el cliente.
  - En `DashboardView.vue`: aplicar color rojo (`text-money-negative`) y signo `-` para egresos en la lista de últimos movimientos.
  - En `App.vue`: redirigir a `/welcome` en primer inicio cuando `eo:welcomed` no exista, permitiendo tanto migrar como comenzar desde cero de manera guiada.
* **Criterio de Aceptación:** Navegación fluida y sin pantallas huérfanas, consistencia en la colorimetría de ingresos y egresos, y bienvenida amigable a nuevos usuarios.
