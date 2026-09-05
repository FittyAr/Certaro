# Fase 14: Anticipos de Clientes, Trazabilidad en Base de Datos, Ergonomía de Órdenes y Coherencia Financiera

## 1. Contexto y Justificación

A partir de la auditoría integral de los flujos de trabajo, cálculos y diseño de Certaro desde la perspectiva de un usuario real de pyme (obras, instalaciones eléctricas y montajes industriales), se identificaron problemas estructurales en la gestión de anticipos, dependencias precarias de almacenamiento en navegador y trabas de navegación:

1. **La Trampa de Doble Imputación de Anticipos / Señas de Obra:**
   - En obras, los clientes pagan señas o anticipos financieros (ej. 30%) antes de la emisión de la factura o certificado. Al ingresar el dinero al banco, el usuario lo asienta en el Libro de Caja como un `Ingreso`.
   - Cuando semanas después se emite la factura, al saldarla en el modal de cobros (`FacturaPagosModal.vue`), la opción *"Registrar movimiento en caja"* viene marcada por defecto. Si el usuario la mantiene, **se genera un segundo ingreso duplicando el saldo de caja**. Si la desmarca, el cobro original queda desvinculado de la factura.
   - En la `CuentaCorrienteView`, los anticipos no facturados son invisibles: el cliente aparece con saldo \$0,00 a pesar de haber entregado millones de pesos como seña.

2. **Fragilidad de Trazabilidad por `localStorage` (Factura $\leftrightarrow$ Obra y Certificado $\leftrightarrow$ Factura):**
   - La vinculación entre una factura y su obra, y la marca de si un certificado ya fue facturado, se guardan en el almacenamiento local del navegador (`localStorage`).
   - Al cambiar de PC, restaurar un backup `.db` o limpiar datos del navegador, se pierde la trazabilidad: certificados facturados vuelven a aparecer como pendientes y las facturas pierden la referencia a la obra.

3. **Inaccesibilidad para Crear Órdenes desde la Vista Global:**
   - En la ruta global `/ordenes` (accesible desde el menú principal), el botón `+ Nuevo` está condicionado a `v-if="trabajoId"`. El usuario no puede presupuestar una orden desde el listado consolidado sin navegar obligatoriamente por múltiples pantallas previas.
   - En la ficha de detalle de trabajo (`TrabajoDetalleView.vue`), el botón `+ Nuevo` bajo la tabla de órdenes redirige a la vista intermedia en vez de abrir directamente el modal de creación.

4. **Orfandad Contable al Anular Liquidaciones de Sueldos:**
   - Al anular un recibo de liquidación en `LiquidacionDetalleView.vue`, la liquidación se da de baja pero el egreso generado en caja no se elimina ni se advierte al usuario, produciendo descuadres en los saldos de tesorería.

5. **Ambigüedad Visual Bimonetaria (USD vs. ARS) en Movimientos:**
   - En la tabla y en la barra de resumen inferior de `MovimientosView.vue`, los importes en USD se muestran con el signo local `$` sin identificar con claridad en los totales que la suma corresponde a moneda extranjera.

6. **Riesgo de Gastos de Obra Fantasma en Proyectos sin Trabajos:**
   - En `MovimientoDrawer.vue`, si un usuario selecciona un proyecto que aún no tiene trabajos creados, el `trabajoId` queda nulo silenciosamente. El movimiento se guarda pero no impacta en la caja de la obra ni en la rentabilidad, sin que el usuario reciba advertencia alguna.

---

## 2. Especificación de Tareas

### Tarea 14.1: Creación de Órdenes Globales y Alta Directa en Detalle de Trabajo
* **Archivos:**
  - `src/views/ordenes/OrdenesView.vue`
  - `src/views/ordenes/components/OrdenFormModal.vue`
  - `src/views/trabajos/TrabajoDetalleView.vue`
* **Detalle:**
  - En `OrdenesView.vue`, habilitar el botón `+ Nuevo` permanentemente. Si `trabajoId` no está en la URL, abrir `OrdenFormModal` en modo de selección de obra.
  - En `OrdenFormModal.vue`, cuando no se reciba un `trabajoId` fijo por prop, incorporar selectores dependientes de Proyecto y Trabajo en el encabezado del modal.
  - En `TrabajoDetalleView.vue`, hacer que el botón `+ Nuevo` abra directamente el `OrdenFormModal` precargando el `trabajoId` actual.
* **Criterio de Aceptación:** El usuario puede crear una orden de trabajo tanto desde el menú global como desde la ficha del trabajo con un solo clic.

### Tarea 14.2: Imputación de Anticipos Preexistentes en Cobranzas de Facturas
* **Archivos:**
  - `src/views/facturas/components/FacturaPagosModal.vue`
* **Detalle:**
  - En `FacturaPagosModal.vue`, consultar los movimientos de ingreso del cliente con `factura_id IS NULL`.
  - Si existen anticipos disponibles, ofrecer la opción de *"Aplicar anticipo o ingreso existente en caja"*.
  - Al seleccionar un anticipo: registrar el pago en la factura y actualizar el movimiento de caja vinculando su `facturaId`, sin generar un nuevo movimiento de ingreso duplicado.
* **Criterio de Aceptación:** Cobrar una factura utilizando un anticipo previo no incrementa el saldo de caja y mantiene la trazabilidad del cobro.

### Tarea 14.3: Visualización de Anticipos y Crédito Disponible en Cuenta Corriente
* **Archivos:**
  - `src/views/comercial/CuentaCorrienteView.vue`
* **Detalle:**
  - Cargar los ingresos registrados a nombre del cliente que no tienen factura imputada (`facturaId == null`).
  - Mostrar una tarjeta/bloque de *"Anticipos y Saldo a Favor"* y computar el Saldo Neto Real (`Saldo Facturado - Anticipos Disponibles`).
* **Criterio de Aceptación:** Un cliente que entregó una seña ve reflejado su crédito a favor en la cuenta corriente antes de que se emita la factura.

### Tarea 14.4: Persistencia Confiable de Trazabilidad en Base de Datos
* **Archivos:**
  - `src/views/certificados/CertificadoDetalleView.vue`
  - `src/views/facturas/FacturasView.vue`
* **Detalle:**
  - Al generar la factura desde un certificado, incluir en `observaciones` la etiqueta estructurada `[cert:${certificadoId}]` y `[proy:${proyectoId}]`.
  - En `verificarFacturado(id)` de certificados, consultar a la base de datos si existe una factura que contenga dicha etiqueta como respaldo robusto frente a la pérdida de `localStorage`.
* **Criterio de Aceptación:** La relación entre certificado y factura persiste aun si se borran los datos temporales del navegador o se consulta en otra máquina.

### Tarea 14.5: Anulación Coordinada de Asientos de Sueldo en Liquidaciones
* **Archivos:**
  - `src/views/liquidaciones/LiquidacionDetalleView.vue`
* **Detalle:**
  - Al pulsar *"Anular Liquidación"*, buscar si existen movimientos de gasto vinculados al pago de haberes del empleado en ese período.
  - En el diálogo de confirmación, consultar al usuario si desea dar de baja también el egreso en caja, procediendo a su eliminación sincronizada.
* **Criterio de Aceptación:** Anular una liquidación permite limpiar el egreso de sueldo en caja en la misma operación, evitando descuadres contables.

### Tarea 14.6: Claridad Bimonetaria en Tabla y Resúmenes de Movimientos
* **Archivos:**
  - `src/views/movimientos/components/MovimientosTable.vue`
  - `src/views/movimientos/MovimientosView.vue`
* **Detalle:**
  - En la tabla de movimientos, señalar explícitamente el distintivo `USD` tanto en monto unitario como en total.
  - En la barra de resumen inferior, si el filtro activo es USD, rotular los indicadores con `(USD)` explícito para evitar confusiones con la moneda local.
* **Criterio de Aceptación:** Un usuario que filtra por USD sabe inequívocamente que los totales mostrados corresponden a dólares estadounidenses.

### Tarea 14.7: Validación y Advertencia en Proyectos sin Trabajos
* **Archivos:**
  - `src/views/movimientos/components/MovimientoDrawer.vue`
* **Detalle:**
  - Cuando se seleccione un Proyecto que no posea ningún Trabajo creado, mostrar un aviso contextual advirtiendo que para imputar costos al proyecto debe existir al menos un trabajo, evitando que el gasto se guarde huérfano.
* **Criterio de Aceptación:** El usuario no puede imputar accidentalmente un gasto a una obra sin trabajos sin ser advertido.

### Tarea 14.8: Verificación y Pruebas Unitarias
* **Archivos:**
  - Backend and frontend test suites.
* **Detalle:**
  - Ejecutar `cargo test --workspace` y `pnpm test` validando 100% de éxito y cero regresiones.
