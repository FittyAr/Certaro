# Fase 11: Auditoría Integral de Flujo Operativo, Cálculos Financieros y Ergonomía UX/UI

## 1. Contexto y Justificación

A partir de una simulación completa de usuario en un escenario de pyme contratista (instalaciones eléctricas, climatización, reformas y servicios técnicos), se auditó el flujo operativo integral de Certaro: presupuestación de obras, avance métrico y emisión de certificados, facturación y cobranzas en cuenta corriente, libro diario de caja y rentabilidad de proyectos, y liquidación de jornales con control de adelantos y asistencia.

Durante esta auditoría se detectaron desalineaciones matemáticas, riesgos operativos directos y fricciones que impactan la productividad diaria del usuario:

1. **Discrepancia Matemática en Certificados Parciales (Frontend vs Backend):**
   - En `OrdenEmisionCertificadoModal.vue`, al calcular la deducción proporcional de `otros_descuentos`, el modal multiplica la proporción de avance por el saldo remanente (`(sub / totalOrden) * restante`), aplicando un descuento cuadrático menguante. El backend (`issuance.rs`), en cambio, aplica la proporción de avance sobre el descuento total original pactado en la orden (`orden.otros_descuentos.percent(pct)`).
   - *Impacto:* El usuario aprueba un certificado esperando un descuento (ej. $25.000) pero el sistema emite el comprobante aplicando un descuento diferente (ej. $50.000), alterando el total neto facturado.

2. **Riesgo Crítico de Pérdida de Datos: Borrado de Pagos sin Confirmación:**
   - En `FacturaPagosModal.vue`, el botón con icono de papelera invoca directamente `borrarPago(data.id, data.rowVersion)`.
   - *Impacto:* La baja del cobro desencadena no solo el recálculo del saldo de la factura sino la baja lógica (`soft_delete`) inmediata del movimiento de fondos en el Libro de Caja general. Un clic accidental de ratón borra registros contables y fondos de caja sin advertencia alguna.

3. **Pérdida de Imputación a Obra en Cobranzas desde Cuenta Corriente:**
   - Desde la ficha de un cliente o el listado de Cuenta Corriente (`CuentaCorrienteCobroModal.vue`), el usuario puede cobrar una factura adeudada y asentar el ingreso en caja, pero el modal carece de selectores de Proyecto y Trabajo (a diferencia de `FacturaPagosModal.vue`).
   - *Impacto:* Los fondos ingresan a la caja como recaudación general sin imputación a obra, ocultando los ingresos reales del proyecto en la ficha de obra (`ProyectoCajaView.vue`) y en el balance de rentabilidad.

4. **Fricción en Registro de Gastos desde Ficha de Obra:**
   - Al pulsar `+ Registrar Gasto` en la pestaña de Tesorería de un proyecto (`ProyectoDetalleView.vue`), la aplicación redirige a `/movimientos` con el proyecto precargado, pero el campo "Tipo de Movimiento" queda en blanco.
   - *Impacto:* El usuario hizo clic en "Registrar Gasto", pero se ve obligado a desplegar el combo y buscar "Gasto" manualmente, con riesgo de error o validación fallida.

5. **Inconsistencia Visual en Recargos y Falta de Alerta ante Salario Neto Negativo:**
   - En el asistente de liquidaciones (`LiquidacionItemSugerido.vue`), si el usuario modifica la tarifa diaria de un operario, la columna de recargos muestra el valor congelado de la sugerencia original, generando una incoherencia visual donde `(días * tarifa) + recargos != total_bruto`.
   - Asimismo, si los adelantos descontados superan el sueldo bruto devengado, el neto resultante se vuelve negativo sin emitir ninguna alerta o advertencia al usuario.

6. **Cuello de Botella en Carga de Asistencia de Cuadrilla:**
   - En obras a campo, cuadrillas completas (5 a 20 personas) concurren en el mismo horario. El modal `AsistenciaCargaMasivaModal.vue` solo permite cargar un rango de fechas empleado por empleado, obligando al usuario a repetir la carga tantas veces como operarios tenga.

7. **Higiene de Código y Glifos Corruptos Residuales:**
   - En `ReportesView.vue` persisten glifos corruptos en comentarios de cabecera y en `CuentaCorrienteView.vue` una declaración anidada redundante de `fechaPagoToIso`.

---

## 2. Especificación de Tareas

### Tarea 11.1: Corrección y Unificación del Cálculo de Descuentos en Certificados
* **Problema:** El modal de emisión de certificados en frontend calcula los descuentos parciales con una fórmula divergente a la del backend.
* **Solución Técnica:**
  - En `src/views/ordenes/components/OrdenEmisionCertificadoModal.vue`:
    - En la propiedad computada `otrosDescuentosAEmitir`, calcular la deducción proporcional aplicando el avance sobre el total de la orden y el monto disponible (`Math.min(prop, restante)`), garantizando paridad exacta con `issuance.rs`.
  - En `src/views/ordenes/OrdenDetalleView.vue`:
    - Añadir acceso directo para facturar el certificado o ver su detalle en la tabla de certificados emitidos.
* **Criterio de Aceptación:** El monto total y neto previsualizado en el modal coincide exactamente con el total del certificado generado por el backend.

### Tarea 11.2: Confirmación Segura en Eliminación de Cobros
* **Problema:** Los pagos de facturas se eliminan sin confirmación en `FacturaPagosModal.vue`.
* **Solución Técnica:**
  - Integrar `useConfirmDelete` para solicitar confirmación explícita con fecha, medio de pago e importe antes de ejecutar `borrarPago`.
* **Criterio de Aceptación:** Al presionar la papelera de un pago, se despliega un diálogo de confirmación. Si el usuario cancela, no se altera ningún registro.

### Tarea 11.3: Imputación a Obra en Cobranzas desde Cuenta Corriente
* **Problema:** `CuentaCorrienteCobroModal.vue` no permite elegir proyecto ni frente de obra para el movimiento de caja.
* **Solución Técnica:**
  - Incorporar selectores dependientes de **Proyecto** y **Trabajo** en `CuentaCorrienteCobroModal.vue`.
  - Actualizar `CuentaCorrienteView.vue` para pasar `proyectoId` y `trabajoId` al crear el movimiento en `movimientosStore.create`.
* **Criterio de Aceptación:** Al cobrar una factura desde Cuenta Corriente asignándole una obra, el movimiento en el Libro de Caja queda vinculado al proyecto y se refleja de inmediato en su caja y rentabilidad.

### Tarea 11.4: Preselección de "Gasto" desde Caja de Obra
* **Problema:** Navegar con "Registrar Gasto" deja el tipo de movimiento vacío.
* **Solución Técnica:**
  - En `src/views/movimientos/MovimientosView.vue`: Cuando la ruta reciba `proyectoId` o `tipoMovimientoId`, preseleccionar automáticamente el identificador de `Gasto` (`00000000-0000-0000-0000-000000000002`).
  - En `src/views/movimientos/components/MovimientoDrawer.vue`: Soportar `preset.tipoMovimientoId` en `openCreate`.
* **Criterio de Aceptación:** Al presionar "+ Registrar Gasto" en la obra, el drawer abre con Cliente, Proyecto, Trabajo y Tipo "Gasto" preseleccionados.

### Tarea 11.5: Recálculo Dinámico de Recargos y Alerta de Neto Negativo en Liquidaciones
* **Problema:** Visualización desincronizada de recargos al cambiar tarifa y ausencia de alerta si los adelantos superan el sueldo.
* **Solución Técnica:**
  - En `src/views/liquidaciones/components/LiquidacionItemSugerido.vue`:
    - Recalcular dinámicamente los recargos con la tarifa actual editada por el usuario.
    - Renderizar un aviso visual `bg-warning/10 text-warning` cuando `totalNeto < 0`, indicando que los adelantos exceden el sueldo bruto generado.
* **Criterio de Aceptación:** Al cambiar la tarifa en el paso 2, los recargos se actualizan en pantalla y la suma es coherente; si el neto es negativo, se muestra una advertencia visible.

### Tarea 11.6: Carga Masiva de Asistencia para Toda la Cuadrilla
* **Problema:** La carga masiva solo se ejecuta para un trabajador a la vez.
* **Solución Técnica:**
  - En `src/views/asistencia/components/AsistenciaCargaMasivaModal.vue`:
    - Incorporar opción o toggle *"Aplicar a toda la cuadrilla activa"*.
    - Al guardar, iterar sobre todos los empleados activos seleccionados y cargar el rango para cada uno en paralelo.
* **Criterio de Aceptación:** El encargado puede marcar jornadas de lunes a viernes para toda la cuadrilla de una sola vez.

### Tarea 11.7: Higiene de Código y Saneamiento de Glifos
* **Problema:** Glifos rotos en comentarios de `ReportesView.vue` y función anidada en `CuentaCorrienteView.vue`.
* **Solución Técnica:**
  - Corregir `\uFFFD` por `§` en `ReportesView.vue`.
  - Extraer y refactorizar `fechaPagoToIso` en `CuentaCorrienteView.vue`.
* **Criterio de Aceptación:** 0 errores de tipado en TypeScript, 100% pruebas de linter y arquitectura aprobadas.
