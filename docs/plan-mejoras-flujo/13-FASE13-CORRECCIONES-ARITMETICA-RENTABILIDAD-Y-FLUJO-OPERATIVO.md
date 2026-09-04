# Fase 13: Correcciones Críticas de Aritmética de Cotización, Rentabilidad Bimonetaria y Flujo Operativo

## 1. Contexto y Justificación

A partir de la auditoría operativa integral de flujos de trabajo, cálculos matemáticos y diseño UX/UI en Certaro (escenario de pyme de obras, instalaciones y montajes), se identificaron fallas de cálculo, inconsistencias bimonetarias y trabas de navegación que afectan la toma de decisiones financieras y la productividad operativa:

1. **Aritmética Rota en Presupuestos / Cotizaciones ("Total Neto" Negativo):**
   - En `OrdenTrabajo`, el método de dominio `total_neto` resta `otros_descuentos` del `total_certificado`. En órdenes de trabajo y cotizaciones nuevas (donde el avance certificado es 0%), el neto resultante se calcula como `0 + 0 - otros_descuentos`, arrojando valores negativos (ej. `-$ 50.000,00`).
   - El sistema carecía de un cálculo formal para el **Total Presupuestado Neto** de la cotización completa (`presupuesto + ajuste_uocra - otros_descuentos`), confundiendo al cliente y al contratista al momento de presupuestar.

2. **Distorsión Aritmética Bimonetaria en Rentabilidad de Proyectos (USD vs. ARS):**
   - En `crates/certaro-infrastructure/src/persistence/repositories/proyecto/query.rs`, la consulta SQL de `suma_movimientos_expr` para calcular la rentabilidad de las obras sumaba directamente `monto * cantidad` sin verificar si la moneda era USD ni aplicar `cotizacion_aplicada`.
   - Compras en dólares (ej. 500 USD a cotización de $1.200 = $600.000 ARS) se restaban como $500 pesos, inflando falsamente las ganancias del proyecto en cientos de miles de pesos.

3. **Pérdida de Adelantos Preexistentes en la Liquidación de Sueldos:**
   - La consulta de adelantos candidatos en `crates/certaro-infrastructure/src/persistence/repositories/liquidacion/mod.rs` filtraba estrictamente entre `m.fecha >= desde AND m.fecha <= hasta` de la quincena.
   - Si un operario recibía un adelanto días antes del inicio de la quincena (ej. el día 14 o a fin de mes pasado) que aún no había sido descontado, el sistema lo ignoraba por completo, liquidando el sueldo íntegro sin deducir la deuda.

4. **Borrado Heurístico e Impreciso de Cobros en Caja (`borrar_pago`):**
   - Al eliminar un cobro de una factura, el backend buscaba movimientos únicamente por `factura_id` y mismo monto (`monto_min = monto_max`), procediendo a dar de baja el primer registro retornado por la consulta.
   - En facturas con múltiples pagos de igual monto o cuando un pago no generó movimiento contable, existía el riesgo de borrar el asiento contable de otro pago legítimo.

5. **Acceso Global a Órdenes de Trabajo / Presupuestos en Navegación:**
   - En el menú principal de la aplicación no existía acceso directo a las Órdenes de Trabajo. El usuario debía navegar obligatoriamente por Proyectos $\rightarrow$ Detalle $\rightarrow$ Trabajos $\rightarrow$ Órdenes. No existía un listado global consolidado para revisar todas las cotizaciones abiertas o en proceso de la empresa.

6. **Sobrecarga de Peticiones en Carga Masiva de Asistencia:**
   - En `AsistenciaCargaMasivaModal.vue`, al marcar "Toda la cuadrilla activa" (ej. 15 operarios), el frontend ejecutaba un bucle secuencial llamando a `store.cargarRango(...)`, recargando la grilla completa 15 veces por IPC/red y ralentizando la interfaz.

---

## 2. Especificación de Tareas

### Tarea 13.1: Incorporación de Total Presupuestado Neto y Corrección en Órdenes de Trabajo
* **Archivos:**
  - `crates/certaro-domain/src/entities/orden_trabajo.rs`
  - `crates/certaro-application/src/dtos/ordenes_trabajo.rs`
  - `crates/certaro-infrastructure/src/persistence/mappers/orden_trabajo.rs`
  - `src/views/ordenes/OrdenDetalleView.vue`
* **Detalle:**
  - En el dominio, incorporar `ajuste_uocra_presupuestado(&self)` y `total_presupuestado_neto(&self)`, calculando el valor neto final de la cotización pactada:
    $$\text{Total Presupuestado Neto} = \text{Total Presupuestado} + \text{Ajuste UOCRA Presupuestado} - \text{Otros Descuentos}$$
  - Mapear estos campos en el DTO `OrdenTrabajoDetalle`.
  - En `OrdenDetalleView.vue`, mostrar en el bloque de totales el desglose claro de la cotización completa (`Total Presupuestado`, `Ajuste UOCRA`, `Otros Descuentos` y `Total Presupuestado Neto`), diferenciándolo del bloque de avance certificado para evitar netos negativos.
* **Criterio de Aceptación:** Una orden recién creada con descuentos pactados muestra su precio total neto de cotización coherente y positivo en lugar de `-$ 50.000,00`.

### Tarea 13.2: Corrección Bimonetaria en Rentabilidad de Obras (SeaORM)
* **Archivos:**
  - `crates/certaro-infrastructure/src/persistence/repositories/proyecto/query.rs`
* **Detalle:**
  - Modificar `suma_movimientos_expr` para que en la suma de egresos e ingresos aplique la fórmula de monto consolidado a ARS:
    ```sql
    CASE WHEN movimiento.moneda = 1 AND movimiento.cotizacion_aplicada IS NOT NULL AND movimiento.cotizacion_aplicada > 0
         THEN (movimiento.monto * movimiento.cotizacion_aplicada / 10000) * movimiento.cantidad
         ELSE movimiento.monto * movimiento.cantidad END
    ```
* **Criterio de Aceptación:** Una compra de 500 USD a cotización 1.200 impacta en la rentabilidad del proyecto restando $600.000 ARS y no $500 ARS.

### Tarea 13.3: Inclusión de Adelantos Preexistentes No Descontados en Liquidación
* **Archivos:**
  - `crates/certaro-infrastructure/src/persistence/repositories/liquidacion/mod.rs`
* **Detalle:**
  - En `candidatos_adelantos`, ampliar la condición para incluir cualquier adelanto con fecha anterior o igual a `hasta_fecha` que no haya sido descontado en otra liquidación previa (`liquidacion_adelanto.id IS NULL`), permitiendo recuperar adelantos de fechas previas a la quincena actual.
* **Criterio de Aceptación:** Un adelanto entregado antes del inicio de la quincena aparece listado en el asistente de liquidación como sugerencia a descontar.

### Tarea 13.4: Precisión en Borrado de Pagos y Asientos de Caja
* **Archivos:**
  - `crates/certaro-application/src/use_cases/facturas/pagos.rs`
  - `src/views/facturas/components/FacturaPagosModal.vue`
* **Detalle:**
  - Al generar el movimiento de caja en `FacturaPagosModal.vue`, incluir en el concepto la fecha, medio de pago e importe específico del cobro.
  - En `borrar_pago`, refinar la búsqueda para validar que el movimiento a anular corresponda estrictamente al medio de pago y fecha del cobro borrado.
* **Criterio de Aceptación:** Borrar un cobro por transferencia no afecta un cobro en efectivo del mismo monto registrado previamente.

### Tarea 13.5: Acceso Global y Gestión de Órdenes de Trabajo en Menú Principal
* **Archivos:**
  - `src/router/menu.ts`
  - `src/router/routes.ts`
  - `src/views/ordenes/OrdenesView.vue`
* **Detalle:**
  - Registrar la ruta global `/ordenes` en el menú (grupo Comercial) y en el router.
  - Adaptar `OrdenesView.vue` para que, cuando no se pase `trabajoId`, cargue y permita filtrar todas las órdenes de trabajo por cliente, proyecto y texto de búsqueda.
* **Criterio de Aceptación:** El usuario puede hacer clic en "Órdenes de Trabajo" en la barra lateral y ver todas las cotizaciones de la empresa sin tener que entrar proyecto por proyecto.

### Tarea 13.6: Optimización Batch en Carga Masiva de Asistencia
* **Archivos:**
  - `src/stores/useAsistenciaStore.ts`
  - `src/views/asistencia/components/AsistenciaCargaMasivaModal.vue`
* **Detalle:**
  - Implementar método batch `cargarRangoCuadrilla` en `useAsistenciaStore.ts` que envíe las peticiones de los operarios en paralelo y realice una única recarga de la grilla (`fetchGrilla`) al finalizar.
* **Criterio de Aceptación:** La asignación de jornadas para toda la cuadrilla se procesa ágilmente y realiza una sola recarga visual.

### Tarea 13.7: Verificación Integral y Pruebas Unitarias
* **Archivos:**
  - `crates/certaro-domain/src/entities/orden_trabajo.rs` (tests)
  - `crates/certaro-infrastructure/tests/`
  - Frontend vitest specs
* **Detalle:**
  - Validar con `cargo test --workspace` y `pnpm test` que todas las fórmulas y componentes funcionen sin regresiones.
* **Criterio de Aceptación:** 100% de los tests pasando sin advertencias ni fallos.
