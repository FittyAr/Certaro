# Especificación Técnica: Módulo de Movimientos y Caja Real

## 1. Diagnóstico Actual

El libro de caja y movimientos es el corazón financiero de la pyme. En la auditoría se identificaron tres fallas estructurales:
1. **Imposibilidad de asociar gastos a obras:** El DTO backend de Movimientos (`MovimientoInput`) cuenta con las propiedades opcionales `cliente_id`, `trabajo_id`, `empleado_id` y `factura_id`. Sin embargo, en el formulario [MovimientosView.vue](../../src/views/movimientos/MovimientosView.vue), estos campos no existen visualmente en la plantilla.
2. **Bloqueo del tipo "Adelanto":** El validador backend exige obligatoriamente un `empleado_id` cuando `tipo_movimiento_id == ADELANTO`. Al no haber selector de empleado, el usuario no puede guardar ningún adelanto.
3. **Desconexión con Facturación:** Al registrar pagos en [FacturasView.vue](../../src/views/facturas/FacturasView.vue), se actualiza el saldo de la factura pero no se genera un movimiento en caja, desvirtuando el balance de caja real reportado.
4. **Caja de Proyecto empobrecida:** [ProyectoCajaView.vue](../../src/views/proyectos/ProyectoCajaView.vue) no diferencia ingresos de egresos visualmente, no muestra un balance de obra ni permite asentar gastos en el contexto de la obra.

---

## 2. Solución Propuesta

### 2.1. Selector Jerárquico e Imputación en Movimientos
En el CrudDrawer de [MovimientosView.vue](../../src/views/movimientos/MovimientosView.vue):

1. **Selector de Empleado:**
   - Componente `Select` con filtro, cargado desde `useEmpleadosStore.fetchLookup()`.
   - Se muestra siempre como opcional, pero si el usuario selecciona como tipo `Adelanto`, el campo se marca como obligatorio (`required`), mostrando un aviso y validación inline (`Validation.Movimiento.EmpleadoRequeridoAdelanto`).
2. **Selector Jerárquico de Imputación a Obra:**
   - **Cliente:** Selector opcional de cliente.
   - **Proyecto:** Selector opcional de proyecto (filtrado automáticamente por el cliente si éste fue seleccionado).
   - **Trabajo:** Selector opcional de trabajo (filtrado automáticamente por el proyecto).
   - *Nota de negocio:* La imputación a nivel de `trabajo_id` es la que activa la suma de gastos en la consulta SQL de `rentabilidad_proyectos` del Dashboard. Por lo tanto, si el usuario selecciona un proyecto que tiene un único trabajo general, el sistema puede autoseleccionar dicho trabajo por conveniencia.

### 2.2. Enriquecimiento de la Caja de Proyecto ([ProyectoCajaView.vue](../../src/views/proyectos/ProyectoCajaView.vue))
1. **Encabezado Informativo:**
   - Mostrar el Nombre del Proyecto y su Número en lugar del texto genérico.
   - Botón de retorno `<Button variant="outline" @click="router.back()">`.
2. **Tarjetas de Balance de la Obra (KPIs):**
   - **Total Ingresos Imputados:** Suma de movimientos positivos asociados a trabajos de la obra.
   - **Total Gastos Imputados:** Suma de egresos (materiales, subcontratos, gastos directos).
   - **Resultado Neto de Caja:** Balance coloreado (verde si es superávit, rojo si es déficit).
3. **Tabla con Signos y Colores:**
   - Utilizar `<MoneyText :value="data.esIngreso ? data.total : `-${data.total}`" colored />` para que los gastos se distingan claramente de los ingresos.
4. **Acción de Alta Rápida:**
   - Botón `+ Registrar Gasto en esta Obra` que abre el drawer de movimientos con el proyecto y cliente ya preconfigurados.

### 2.3. Sincronización Automática de Cobranzas de Facturas con Caja
En el modal de cobro de facturas ([FacturasView.vue:abrirPagos](../../src/views/facturas/FacturasView.vue#L136)):
- Agregar un toggle switch o checkbox: *"Asentar ingreso en Libro de Caja"* (activo por defecto).
- Si está activo, el formulario solicita opcionalmente la Categoría de Ingreso (por defecto la primera de tipo cobro/comercial) y la cuenta/caja de destino.
- En el backend ([crates/certaro-application/src/use_cases/facturas.rs](../../crates/certaro-application/src/use_cases/facturas.rs)), al ejecutarse `crear_pago`, si se solicita el asiento en caja, se inserta atómicamente el `Movimiento` correspondiente vinculado a `factura_id` y `cliente_id`.

---

## 3. Modificaciones de Archivos y Componentes

### Frontend
- **[src/views/movimientos/MovimientosView.vue](../../src/views/movimientos/MovimientosView.vue):**
  - Añadir llamadas a lookup de empleados (`useEmpleadosStore`) y clientes/proyectos.
  - Agregar campos en el CrudDrawer:
    - `Select` para `empleadoId` con validación condicional.
    - `Select` para `clienteId`.
    - `Select` para `trabajoId` (dependiente del proyecto o cliente).
- **[src/views/proyectos/ProyectoCajaView.vue](../../src/views/proyectos/ProyectoCajaView.vue):**
  - Incorporar métricas de resumen (ingresos, egresos, balance).
  - Integrar botón de regreso y botón de alta rápida de movimiento.
  - Formatear columna de total con signo y color.
- **[src/views/facturas/FacturasView.vue](../../src/views/facturas/FacturasView.vue):**
  - Incorporar en el diálogo de nuevo pago la opción de generar movimiento automático de caja.

### Backend
- **[crates/certaro-application/src/dtos/facturas.rs](../../crates/certaro-application/src/dtos/facturas.rs):**
  - Añadir flags `asentar_en_caja: bool` y `categoria_id: Option<Uuid>` al DTO `PagoFacturaInput`.
- **[crates/certaro-application/src/use_cases/facturas.rs](../../crates/certaro-application/src/use_cases/facturas.rs):**
  - Si `asentar_en_caja` es verdadero, instanciar e insertar un registro en `movimientos` dentro de la misma transacción SeaORM (`tx.movimientos().insert(...)`).
