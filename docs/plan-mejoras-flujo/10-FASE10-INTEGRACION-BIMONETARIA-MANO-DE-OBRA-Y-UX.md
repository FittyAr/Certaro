# Fase 10: Integración Bimonetaria, Imputación de Mano de Obra y Ergonomía de Flujo

## 1. Contexto y Justificación

A partir de la auditoría exhaustiva del flujo de trabajo y cálculos de Certaro desde la perspectiva de un usuario real de pyme (instalaciones eléctricas, climatización, construcción y servicios técnicos), se detectaron inconsistencias críticas en el tratamiento de monedas, omisiones en el costeo de obras y áreas de fricción de interfaz:

1. **Distorsión Bimonetaria en Caja y Dashboard:** Al no aplicar la cotización de cambio registrada (`cotizacion_aplicada`), las transacciones en Pesos (ARS) y Dólares (USD) se totalizan linealmente 1 a 1 en las consultas de base de datos. Un gasto de USD 1.500 se computa como $1.500 pesos, desfigurando el saldo real de tesorería, el balance patrimonial y los gráficos anuales.
2. **Mano de Obra Omitida en la Rentabilidad de Obra:** Al liquidar jornales y registrar el egreso correspondiente en caja, el movimiento se guarda sin `trabajo_id` ni `proyecto_id`. En consecuencia, la mano de obra (que constituye el 40-60% del costo operativo real de cualquier obra) nunca se imputa al proyecto, mostrando márgenes ficticios e inflados en la ficha de obra y en el Dashboard.
3. **Desconexión del IVA Sugerido al Facturar Certificados:** Al pulsar *"Facturar este Certificado"*, el sistema precarga `iva = 0.0000` ignorando la alícuota configurada (`ivaSugerido`). Además, el formulario de facturas no ofrece botones de selección rápida de alícuotas (0%, 10.5%, 21%, 27%), obligando al usuario a recurrir a cálculos manuales con calculadora externa.
4. **Fuga de Información en Modo Privacidad:** En el Dashboard, los últimos movimientos fuerzan el signo `+` o `-` y colores verde/rojo por fuera del componente `MoneyText`, revelando la naturaleza y sentido de las transacciones aun con el modo confidencial activado.
5. **Caracteres Corruptos (`\uFFFD`) y Diálogos Residuales:** Presencia de glifos rotos en pantallas de reportes y certificados por codificación incorrecta, junto con el uso residual de ventanas nativas `window.confirm()`.
6. **Información Incompleta en Ficha de Cliente:** `ClienteDetalleView` no exhibe el saldo de cuenta corriente ni el listado de proyectos activos, exigiendo navegar hacia otras pantallas para conocer el estado comercial del cliente.
7. **Paginación Truncada en Obra:** La pestaña de caja de la ficha de proyecto carece de controles de paginación en pantalla para obras con más de 100 movimientos.

---

## 2. Especificación de Tareas

### Tarea 10.1: Conversión y Consolidación Bimonetaria en Caja y Dashboard
* **Problema:** Las consultas SQL agregan importes en ARS y USD sin conversión, falseando los totales consolidados.
* **Solución Técnica:**
  - En `crates/certaro-infrastructure/src/persistence/repositories/movimiento.rs`: En el método `resumen()`, cuando el filtro no especifique una moneda concreta (`filtro.moneda IS NULL`), convertir las transacciones en USD a la moneda base multiplicando por `COALESCE(m.cotizacion_aplicada, 1.0)`.
  - En `crates/certaro-infrastructure/src/persistence/repositories/dashboard.rs`: En `resumen_rango()`, aplicar la misma regla de conversión monetaria para que los KPIs de Ingresos, Gastos, Balance y el gráfico de serie mensual reflejen magnitudes económicas homogéneas.
  - En `src/views/movimientos/MovimientosView.vue`: Usar tokens semánticos en las etiquetas de moneda USD (`bg-warning/10 text-warning border-warning/30`).
* **Criterio de Aceptación:** Al totalizar gastos de $100.000 ARS y USD 1.000 (cotización 1.200), el total consolidado refleja $1.300.000 ARS en lugar de $101.000 ARS.

### Tarea 10.2: Imputación de Costos Laborales a Obras en Liquidaciones
* **Problema:** Los egresos de sueldos liquidados se registran sin asignación a obra, ocultando el costo de mano de obra en la caja de proyecto.
* **Solución Técnica:**
  - En `src/views/liquidaciones/LiquidacionesView.vue` (Paso 3 del wizard de liquidación):
    - Incorporar selectores dependientes de **Proyecto** y **Trabajo** para la imputación del pago de sueldos (con opción de imputación general si la liquidación abarca personal administrativo o de taller).
    - Al invocar `movimientosStore.create()`, enviar los identificadores `proyectoId` y `trabajoId` seleccionados.
* **Criterio de Aceptación:** El usuario puede imputar la liquidación de sueldos a una obra determinada, y el egreso impacta inmediatamente en `ProyectoCajaView` y en el cálculo de rentabilidad del proyecto.

### Tarea 10.3: Automatización de IVA Sugerido y Alícuotas Rápidas en Facturación
* **Problema:** La emisión de factura desde certificado deja el IVA en 0, y el alta manual de facturas carece de selectores de porcentaje de IVA.
* **Solución Técnica:**
  - En `src/views/certificados/CertificadoDetalleView.vue`:
    - Leer `sistema.config?.business.ivaSugerido` (por defecto 21%).
    - Calcular el importe sugerido: `iva = (totalNeto * ivaSugerido / 100).toFixed(4)` y `total = (totalNeto + iva).toFixed(4)`.
    - Reemplazar `window.confirm()` por `useConfirm` / `confirm.require`.
  - En `src/views/facturas/FacturasView.vue`:
    - Incorporar chips/botones de alícuota rápida (`0%`, `10.5%`, `21%`, `27%`) que calculen automáticamente el monto de IVA sobre el subtotal ingresado y actualicen el total.
* **Criterio de Aceptación:** Al facturar un certificado de $100.000, el formulario de facturas abre con IVA de $21.000 y Total de $121.000 precargados, y el usuario puede cambiar la alícuota con un solo clic.

### Tarea 10.4: Corrección de Fuga de Signo en Modo Privacidad del Dashboard
* **Problema:** En los últimos movimientos del Dashboard, los caracteres `+` y `-` y las clases de color verde/rojo revelan el flujo monetario con el modo confidencial activado.
* **Solución Técnica:**
  - En `src/views/dashboard/DashboardView.vue`:
    - Ocultar los caracteres de signo o usar estilo neutro cuando `ui.privacyMode` sea verdadero.
* **Criterio de Aceptación:** Con el modo privacidad activo, la lista de últimos movimientos no delata el sentido del flujo ni el color del importe.

### Tarea 10.5: Saneamiento de Caracteres Corruptos y Claves i18n
* **Problema:** Presencia de caracteres `\uFFFD` y falta de la clave `Ordenes.AjusteUocraPorcentaje` en los catálogos de idioma.
* **Solución Técnica:**
  - En `src/views/reportes/ReportesView.vue`, `src/views/certificados/CertificadoDetalleView.vue` y `src/views/ordenes/OrdenesView.vue`: Reemplazar los caracteres corruptos por el separador `·`.
  - En `src/locales/es.json` y `src/locales/en.json`: Añadir la clave `Ordenes.AjusteUocraPorcentaje` y las traducciones complementarias.
* **Criterio de Aceptación:** `architecture.spec.ts` pasa sin errores de claves faltantes ni colores no permitidos, y las pantallas no muestran signos de interrogación rotos.

### Tarea 10.6: Enriquecimiento de Ficha de Cliente (`ClienteDetalleView`)
* **Problema:** `ClienteDetalleView` oculta la deuda actual y no lista las obras activas del cliente.
* **Solución Técnica:**
  - En `src/views/clientes/ClienteDetalleView.vue`:
    - Agregar una tarjeta de indicador con el saldo en cuenta corriente (`deuda`).
    - Cargar y mostrar la lista de proyectos asociados al cliente con su estado, presupuesto y enlace directo a su detalle.
* **Criterio de Aceptación:** Al entrar a la ficha del cliente, el usuario visualiza de un vistazo cuánto dinero le debe y qué obras están en ejecución.

### Tarea 10.7: Paginación en Caja de Obra y Ergonomía en Reportes
* **Problema:** La pestaña de movimientos en la ficha de obra no incluye paginación en pantalla.
* **Solución Técnica:**
  - En `src/views/proyectos/ProyectoDetalleView.vue`: Habilitar paginador en la tabla de movimientos (`paginator :rows="20" :rows-per-page-options="[10, 20, 50]"`).
* **Criterio de Aceptación:** El usuario puede paginar y revisar el historial completo de movimientos de la obra sin límites artificiales.
