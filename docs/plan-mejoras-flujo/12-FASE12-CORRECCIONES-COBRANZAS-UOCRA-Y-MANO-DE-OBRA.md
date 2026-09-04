# Fase 12: Correcciones Críticas de Cobranzas en Caja, Ajuste UOCRA, Mano de Obra y Ergonomía

## 1. Contexto y Justificación

A partir de una auditoría exhaustiva desde la perspectiva de un usuario operativo (dueño o encargado de pyme de obras, instalaciones y servicios por proyecto), se identificaron inconsistencias aritméticas en pantalla, fallos silenciosos en la integración de cobranzas con la tesorería real, rigideces en la imputación salarial y trampas de navegación que degradan la experiencia de usuario:

1. **Fallo Silencioso de Cobranza en el Libro de Caja:**
   - Al registrar el pago de una factura en `FacturaPagosModal.vue` o `CuentaCorrienteView.vue` con la opción *"Registrar movimiento en caja"*, el frontend envía `categoriaId: null`.
   - La regla de validación de backend `validation/movimientos.rs` (regla INV-03) exige obligatoriamente una categoría no nula (`Validation.Movimiento.CategoriaRequired`).
   - El movimiento es rechazado y el error es capturado silenciosamente en `console.warn`, dejando la factura como pagada pero sin ingresar un solo peso a la caja real de la empresa ni a la rentabilidad de la obra.

2. **Contradicción Aritmética en Ajuste UOCRA (`OrdenDetalleView.vue`):**
   - En el pie de la planilla de cómputo de la orden de trabajo, la fila de Ajuste UOCRA muestra un signo menos literal (`− <MoneyText :value="orden.ajusteUocra" />`).
   - El ajuste UOCRA por regla de negocio suma al neto (`total_neto = total_certificado + ajuste_uocra - otros_descuentos`), generando una contradicción visual donde parece que la aplicación no sabe sumar.

3. **Imputación Unificada de Mano de Obra en Liquidaciones por Lote:**
   - En `LiquidacionWizardModal.vue`, en el paso 3 solo se ofrece un único selector de Proyecto / Trabajo para todo el lote de empleados.
   - En obras múltiples, el usuario se ve obligado a cargarle el 100% del sueldo de toda la cuadrilla a una sola obra (falseando sus costos) o dejarlo vacío (con lo cual ninguna obra absorbe su mano de obra real en su "Caja de Proyecto").

4. **Bloqueo Innecesario en Facturas en Borrador (`FacturasTable.vue`):**
   - El botón del monedero (abrir pagos) está siempre activo, incluso para facturas en estado `Borrador`. Si el usuario intenta cobrar una factura recién creada sin haber presionado el avión de papel de "Emitir", el sistema arroja el error `ESTADO_NO_ADMITE_PAGOS`.

5. **Falta de Previsualización del Total en Alta de Movimientos y Campo Empleado Redundante:**
   - En `MovimientoDrawer.vue`, el usuario ingresa Monto y Cantidad pero no ve el total estimado (`monto × cantidad`) antes de guardar.
   - Además, la condición `v-if` del selector de empleado incluye `drawer.open.value`, haciéndolo visible siempre y saturando formularios de compras cotidianas.

6. **Visualización de "-$ 0,00" en Caja de Obra:**
   - En `ProyectoCajaTab.vue` y `ProyectoCajaView.vue`, se concatena un `-` fijo sobre `totalGastos`, mostrando `-$ 0,00` en rojo en proyectos nuevos sin gastos.

7. **Marcado Prematuro de Certificados como Facturados:**
   - En `CertificadoDetalleView.vue`, el certificado se marca en `localStorage` apenas se pulsa "Facturar", antes de que la factura se guarde. Si el usuario cancela el formulario, el certificado queda advertido erróneamente como ya facturado.

---

## 2. Especificación de Tareas

### Tarea 12.1: Resolución de Categoría Obligatoria en Cobros Automáticos de Factura
* **Archivos:** `src/views/facturas/components/FacturaPagosModal.vue`, `src/views/comercial/CuentaCorrienteView.vue`.
* **Detalle:** Cargar las categorías con `useCatalogStore.loadCategorias()`. Asignar automáticamente una categoría de ingresos al registrar el pago (ej. buscando por palabras clave como "cobranza", "ingreso" o la primera categoría de ingreso disponible). En caso de error, notificar al usuario con `notify(err)` en lugar de silenciarlo.
* **Criterio de Aceptación:** Al cobrar una factura con "Registrar movimiento en caja", el movimiento se inserta correctamente en la base de datos y se refleja de inmediato en el balance de caja y rentabilidad de obra.

### Tarea 12.2: Corrección del Signo de Ajuste UOCRA en Orden de Trabajo
* **Archivos:** `src/views/ordenes/OrdenDetalleView.vue`.
* **Detalle:** Reemplazar el signo `−` por `+` en la fila de `Ajuste UOCRA`, indicando visualmente que es un adicional sobre el valor certificado y armonizando con `OrdenEmisionCertificadoModal.vue`.
* **Criterio de Aceptación:** En la pantalla de orden de trabajo, la suma visual `Total Certificado + Ajuste UOCRA - Otros Descuentos = Total Neto` es matemáticamente coherente.

### Tarea 12.3: Imputación de Obra por Operario en Wizard de Liquidaciones
* **Archivos:** `src/views/liquidaciones/components/LiquidacionWizardModal.vue`, `src/views/liquidaciones/components/LiquidacionItemSugerido.vue`.
* **Detalle:** Permitir seleccionar Proyecto y Trabajo individualmente por operario en el paso 2 del wizard. Al confirmar en el paso 3, si hay empleados con obras distintas, generar los asientos de egreso correspondientes imputados a cada obra y empleado, o respetar la imputación individual en la tesorería.
* **Criterio de Aceptación:** Al liquidar una cuadrilla donde los obreros trabajaron en distintos proyectos, la mano de obra de cada uno impacta exactamente en la "Caja de Proyecto" de la obra asignada.

### Tarea 12.4: Control de Estado y Ergonomía del Botón Cobrar en Facturas
* **Archivos:** `src/views/facturas/components/FacturasTable.vue`.
* **Detalle:** Deshabilitar el botón de cobro o destacar la acción de emisión si la factura está en estado `Borrador`. Mostrar tooltip explicativo *"Debe emitir la factura para registrar pagos"* si no admite pagos.
* **Criterio de Aceptación:** El usuario no tropieza con errores `ESTADO_NO_ADMITE_PAGOS` y es guiado a emitir la factura antes de cobrarla.

### Tarea 12.5: Previsualización de Subtotal y Limpieza de Selector de Empleado en Movimientos
* **Archivos:** `src/views/movimientos/components/MovimientoDrawer.vue`.
* **Detalle:** Incorporar un indicador reactivo que calcule y muestre `Total estimado: Monto × Cantidad` en tiempo real. Corregir la condición `v-if` para que el selector de empleado se muestre solo cuando el tipo es `Adelanto` o ya tiene un empleado asignado.
* **Criterio de Aceptación:** El usuario tiene confirmación inmediata del total antes de guardar y los formularios de gastos estándar no solicitan empleado innecesariamente.

### Tarea 12.6: Corrección de Visualización "-$ 0,00" en Caja de Obra
* **Archivos:** `src/views/proyectos/components/ProyectoCajaTab.vue`, `src/views/proyectos/ProyectoCajaView.vue`.
* **Detalle:** Condicionar el prefijo `-` para que cuando `totalGastos === '0.0000'` o el número sea 0, se muestre `$ 0,00` con estilo neutral.
* **Criterio de Aceptación:** Una obra sin gastos muestra `$ 0,00` neutral sin signo negativo ni color rojo de alerta.

### Tarea 12.7: Trazabilidad de Certificado Facturado Post-Confirmación
* **Archivos:** `src/views/certificados/CertificadoDetalleView.vue`, `src/views/facturas/FacturasView.vue`.
* **Detalle:** No escribir en `localStorage` al hacer clic en "Facturar Certificado". Pasar el `certificadoId` en la URL y registrar la marca de facturado únicamente cuando la factura es persistida con éxito en el método `create` de `useCrudDrawer`.
* **Criterio de Aceptación:** Cancelar o cerrar el formulario de factura no deja el certificado marcado falsamente como facturado.