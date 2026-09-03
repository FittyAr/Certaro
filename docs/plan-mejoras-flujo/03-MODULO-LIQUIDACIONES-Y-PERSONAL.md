# Especificación Técnica: Módulo de Liquidaciones y Personal

## 1. Diagnóstico Actual

El módulo de liquidación de haberes y jornales ([LiquidacionesView.vue](../../src/views/liquidaciones/LiquidacionesView.vue)) permite calcular el sueldo de cuadrillas completas mediante un asistente de tres pasos. Sin embargo, se detectaron fallas de cálculo y de flujo de trabajo:

1. **Bug Crítico de Recargos en Lotes:**
   - En [LiquidacionesView.vue:175-199](../../src/views/liquidaciones/LiquidacionesView.vue#L175-L199), existe una variable computada:
     ```typescript
     const huboCambioDeBase = computed(() =>
       store.sugerencias.some((s) => {
         const ajuste = ajusteDe(s.empleadoId)
         return ajuste.diasTrabajados !== s.diasTrabajados || ajuste.tarifaAplicada !== s.tarifaAplicada
       }),
     )
     ```
   - Al construir el DTO para el backend (`dtoDe(s)`):
     ```typescript
     totalBruto: huboCambioDeBase.value ? totalBruto : s.totalBruto
     ```
   - **El efecto nocivo:** Si se procesa un lote de 10 trabajadores y se modifica los días o tarifa de **un solo operario**, la condición `huboCambioDeBase` se vuelve verdadera para **todos los demás trabajadores del lote**. En consecuencia, para los 9 operarios no tocados, su `totalBruto` se recalcula como una multiplicación básica `diasTrabajados * tarifaAplicada`, **borrando de un plumazo todos los recargos por sábados trabajados, domingos y feriados** que el backend había calculado en `s.totalBruto`.
2. **Ausencia de Exportación Directa de Recibos de Sueldo:**
   - Tras confirmar una liquidación, ni en la fila del listado ([LiquidacionesView.vue](../../src/views/liquidaciones/LiquidacionesView.vue)) ni en el detalle de la liquidación ([LiquidacionDetalleView.vue](../../src/views/liquidaciones/LiquidacionDetalleView.vue)) existe un botón para generar o descargar el PDF del recibo. El usuario debe ir al módulo de Reportes, buscar en un combo de 100 registros y exportarlo allí.

---

## 2. Solución Propuesta

### 2.1. Corrección Individual del Cálculo de Bruto por Empleado
Se modifica la lógica en [LiquidacionesView.vue](../../src/views/liquidaciones/LiquidacionesView.vue) para evaluar la modificación de base **de forma estrictamente aislada por empleado**:

```typescript
/** Determina si un empleado particular sufrió modificaciones manuales en días o tarifa. */
function empleadoCambioDeBase(s: LiquidacionSugerencia): boolean {
  const ajuste = ajusteDe(s.empleadoId)
  return ajuste.diasTrabajados !== s.diasTrabajados || ajuste.tarifaAplicada !== s.tarifaAplicada
}

function dtoDe(s: LiquidacionSugerencia): LiquidacionInput {
  const ajuste = ajusteDe(s.empleadoId)
  const fueModificado = empleadoCambioDeBase(s)
  
  // Si fue modificado manualmente, se usa la base plana multiplicada (o se solicita recálculo).
  // Si NO fue modificado, se preserva religiosamente el totalBruto original del backend con todos sus recargos.
  const totalBrutoCalculado = (Number(ajuste.diasTrabajados) * Number(ajuste.tarifaAplicada)).toFixed(4)

  return {
    empleadoId: s.empleadoId,
    fechaInicio: s.desde,
    fechaFin: s.hasta,
    diasTrabajados: ajuste.diasTrabajados,
    tarifaAplicada: ajuste.tarifaAplicada,
    incluirSabados: s.incluirSabados,
    incluirDomingos: s.incluirDomingos,
    incluirFeriados: s.incluirFeriados,
    multiplicadorSabado: s.desglose.multiplicadorSabado,
    multiplicadorDomingo: s.desglose.multiplicadorDomingo,
    multiplicadorFeriado: s.desglose.multiplicadorFeriado,
    totalBruto: fueModificado ? totalBrutoCalculado : s.totalBruto,
    totalAdelantos: totalAdelantosDe(s),
    observaciones: ajuste.observaciones,
    adelantos: s.adelantos
      .filter((a) => ajuste.adelantosIncluidos.has(a.movimientoId))
      .map((a) => ({
        movimientoId: a.movimientoId,
        fecha: a.fecha,
        concepto: a.concepto,
        monto: a.monto,
      })),
  }
}
```

*Mejora complementaria:* En el Paso 2 del asistente, si el usuario modifica los días trabajados, mostrar un botón inline "Recalcular con Recargos" para que el backend vuelva a correr las reglas de asistencia/feriados con la nueva cantidad de días antes de confirmar.

### 2.2. Botón de Exportación Directa del Recibo de Sueldo
En [LiquidacionDetalleView.vue](../../src/views/liquidaciones/LiquidacionDetalleView.vue):
- En la ranura `#actions` del `PageHeader`, incorporar:
  ```html
  <Button variant="outline" :disabled="!liquidacion" @click="exportarRecibo()">
    <AppIcon name="file-down" :size="16" />
    {{ $t('Reportes.ExportarPdf') }}
  </Button>
  ```
- Al hacer clic, se abre el diálogo nativo de Tauri para guardar el archivo (`@tauri-apps/plugin-dialog: save`) con el nombre sugerido `Recibo_{Apellido}_{Periodo}.pdf` y se invoca directamente `reportesStore.exportLiquidacion(liquidacion.id, rutaDestino)`.
- En la tabla de [LiquidacionesView.vue](../../src/views/liquidaciones/LiquidacionesView.vue), agregar el mismo ícono en la columna de acciones de cada fila.

---

## 3. Modificaciones de Archivos y Componentes

### Frontend
- **[src/views/liquidaciones/LiquidacionesView.vue](../../src/views/liquidaciones/LiquidacionesView.vue):**
  - Reemplazo de `huboCambioDeBase` por `empleadoCambioDeBase`.
  - Acción de exportación rápida en la columna `#actions` de la tabla.
- **[src/views/liquidaciones/LiquidacionDetalleView.vue](../../src/views/liquidaciones/LiquidacionDetalleView.vue):**
  - Incorporación de `useReportesStore` y del botón de exportación directa a PDF.
- **[src/locales/es.json](../../src/locales/es.json):**
  - Nuevas claves de i18n para exportación y avisos de liquidación.
