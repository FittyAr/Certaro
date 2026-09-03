# Checklist de Avance — Plan de Mejoras del Flujo de Trabajo y Diseño

Este documento sirve como **tablero de control y seguimiento** del plan integral de mejoras de Certaro. Cada tarea cuenta con su alcance técnico, archivos involucrados y criterios de aceptación.

---

## Estado Global del Plan

| Fase | Descripción | Total Tareas | Completadas | Estado |
| :--- | :--- | :---: | :---: | :---: |
| **Fase 1** | Correcciones Críticas de Lógica y Bloqueos de Flujo | 4 | 4 | 🟢 Completada |
| **Fase 2** | Integración del Libro de Caja y Rentabilidad de Obras | 4 | 4 | 🟢 Completada |
| **Fase 3** | Experiencia de Documentos y Acciones Contextuales | 3 | 3 | 🟢 Completada |
| **Fase 4** | Estandarización de UX/UI, Calendario y Onboarding | 4 | 4 | 🟢 Completada |
| **Fase 5** | Pruebas Integrales y Verificación de Flujo End-to-End | 3 | 3 | 🟢 Completada |

---

## Fase 1: Correcciones Críticas de Lógica y Bloqueos de Flujo

- [x] **1.1. Selector de Empleado para Adelantos en Movimientos**
  - **Archivos:** `src/views/movimientos/MovimientosView.vue`, `src/stores/useCatalogStore.ts` o `src/stores/useEmpleadosStore.ts`.
  - **Detalle:** Incorporar selector de empleado en el drawer de movimientos. Condicionar su visibilidad y obligatoriedad visual cuando el tipo de movimiento seleccionado sea `ADELANTO` (o coincida con `tipos_movimiento::ADELANTO`).
  - **Criterio de aceptación:** El usuario puede registrar un adelanto de sueldo sin recibir el error de validación `Validation.Movimiento.EmpleadoRequeridoAdelanto`. El adelanto queda correctamente imputado al empleado y disponible para descontarse en el asistente de liquidaciones.

- [x] **1.2. Corrección del Bug de Recargos en Liquidaciones por Lote**
  - **Archivos:** `src/views/liquidaciones/LiquidacionesView.vue`.
  - **Detalle:** Reemplazar la variable computada global `huboCambioDeBase` por una función de verificación individual `empleadoCambioDeBase(empleadoId: string): boolean`. En la generación del DTO `dtoDe(s)`, aplicar el cálculo plano `dias * tarifa` **únicamente** al empleado que fue modificado; mantener `s.totalBruto` (con sus recargos de sábados, domingos y feriados calculados por backend) para todos los demás empleados del lote no editados.
  - **Criterio de aceptación:** Al liquidar a un lote de empleados y modificar los días de uno solo, los demás empleados conservan íntegros sus recargos de fin de semana y feriados.

- [x] **1.3. Corrección de Mapeo de Columnas en Árbol de Obras (`ProyectosTreeTable`)**
  - **Archivos:** `src/components/domain/ProyectosTreeTable.vue`.
  - **Detalle:** Corregir los campos del nodo hijo `isTrabajo`. No asignar `trab.presupuesto` en la columna de `rentabilidad`, ni `trab.fechaInicio` en la columna de `localidad`. Mostrar en `nombre` la descripción del trabajo, en `estado` su pill de estado, en `presupuesto` su valor (o dejar un guión en rentabilidad), y no sobrecargar la columna localidad con fechas.
  - **Criterio de aceptación:** En la tabla de proyectos, los trabajos hijos no muestran fechas en la columna "Localidad" ni su presupuesto disfrazado de ganancia neta en "Rentabilidad".

- [x] **1.4. Corrección del Desfasaje Horario (Timezone Bug) en Calendario**
  - **Archivos:** `src/views/calendario/CalendarioView.vue`.
  - **Detalle:** Corregir la función `formatearFechaHoraIso` para que no añada el carácter `Z` a una hora obtenida con `d.getHours()` / `d.getMinutes()`. Generar la cadena ISO preservando el huso horario local o convirtiendo explícitamente con `toISOString()` desde el timestamp UTC real.
  - **Criterio de aceptación:** Los eventos agendados a las 09:00 hs se guardan y se visualizan a las 09:00 hs en la cuadrícula, sin sufrir desfasajes de 3 horas.

---

## Fase 2: Integración del Libro de Caja y Rentabilidad de Obras

- [x] **2.1. Selector de Cliente, Proyecto y Trabajo en Formulario de Movimientos**
  - **Archivos:** `src/views/movimientos/MovimientosView.vue`, `src/stores/useMovimientosStore.ts`.
  - **Detalle:** Agregar en el CrudDrawer los selectores dependientes:
    - Selector opcional de **Cliente**.
    - Selector opcional de **Proyecto / Obra** (filtrado por el cliente si se seleccionó uno).
    - Selector opcional de **Trabajo** (filtrado por el proyecto si se seleccionó uno).
  - **Criterio de aceptación:** Al guardar un gasto de materiales o compra, el movimiento guarda su `trabajo_id` y `cliente_id`. 

- [x] **2.2. Verificación de Rentabilidad y Caja de Proyecto**
  - **Archivos:** `crates/certaro-infrastructure/src/persistence/repositories/dashboard.rs`, `src/views/proyectos/ProyectoCajaView.vue`.
  - **Detalle:** 
    - Al ingresar compras imputadas a un trabajo de un proyecto, verificar que la consulta SQL de `rentabilidad_proyectos` refleje inmediatamente los gastos en el Dashboard ("Mejores / Peores Proyectos").
    - En [ProyectoCajaView.vue], implementar encabezado con totales (Ingresos, Egresos, Balance del Proyecto) y formatear la tabla con `MoneyText` coloreado (verde ingresos, rojo egresos con signo `-`).
  - **Criterio de aceptación:** El usuario abre "Caja de Proyecto" y ve claramente cuánto dinero ingresó, cuánto se gastó y cuál es el resultado neto de esa obra en particular.

- [x] **2.3. Alta Rápida de Movimiento desde Caja de Proyecto**
  - **Archivos:** `src/views/proyectos/ProyectoCajaView.vue`.
  - **Detalle:** Agregar botón "+ Registrar Movimiento / Gasto" en `ProyectoCajaView.vue` que abra el drawer de movimientos con el `proyectoId` (y cliente) preseleccionados.
  - **Criterio de aceptación:** El encargado de obra puede registrar un gasto directo sin salir del contexto del proyecto.

- [x] **2.4. Automatización / Vinculación de Cobranzas de Facturas con Caja**
  - **Archivos:** `crates/certaro-application/src/use_cases/facturas.rs`, `src/views/facturas/FacturasView.vue`.
  - **Detalle:** Al registrar un pago en `FacturasView.vue`, permitir marcar la opción "Registrar movimiento en caja" (por defecto activa), indicando la cuenta/medio de pago y categoría de ingreso.
  - **Criterio de aceptación:** Cuando un cliente paga una factura de $100.000, el saldo de la factura se cancela Y se inserta el movimiento de ingreso correspondiente en el Libro de Caja, manteniendo el saldo bancario/caja al día.

---

## Fase 3: Experiencia de Documentos y Acciones Contextuales

- [x] **3.1. Botón de Descarga / Exportación Directa de Recibo en Liquidación**
  - **Archivos:** `src/views/liquidaciones/LiquidacionDetalleView.vue`, `src/views/liquidaciones/LiquidacionesView.vue`.
  - **Detalle:** Agregar botón con ícono de descarga / PDF en el PageHeader de `LiquidacionDetalleView.vue` y como acción en la tabla de `LiquidacionesView.vue` para invocar directamente `reportes.exportLiquidacion(id, destino)`.
  - **Criterio de aceptación:** El usuario no necesita ir al módulo general de Reportes para obtener el recibo de sueldo de un empleado.

- [x] **3.2. Botón de Exportación Directa en Certificado de Avance**
  - **Archivos:** `src/views/certificados/CertificadoDetalleView.vue`, `src/views/certificados/CertificadosView.vue`.
  - **Detalle:** Incorporar acción directa de exportar a PDF (`reportes.exportCertificado`) en el encabezado de `CertificadoDetalleView.vue`.
  - **Criterio de aceptación:** Al emitir un certificado, el usuario puede guardarlo o imprimirlo de inmediato desde esa misma pantalla.

- [x] **3.3. Trazabilidad de Certificado a Factura**
  - **Archivos:** `src/views/certificados/CertificadoDetalleView.vue`, `src/views/facturas/FacturasView.vue`.
  - **Detalle:** Añadir botón "Facturar este Certificado" en el Certificado Detalle que abra el alta de Factura con cliente, subtotal, IVA y notas precargadas con el número de certificado y proyecto.
  - **Criterio de aceptación:** El flujo comercial de emisión de certificado a emisión de comprobante de cobro queda unificado y sin re-tipeo manual de montos.

---

## Fase 4: Estandarización de UX/UI, Calendario y Onboarding

- [x] **4.1. Navegación y Acceso al Detalle de Proyecto**
  - **Archivos:** `src/components/domain/ProyectosTreeTable.vue`, `src/views/proyectos/ProyectoTrabajosView.vue`.
  - **Detalle:**
    - Agregar acción "Ver Detalle" en el menú contextual del árbol de proyectos y soporte de doble clic en la fila.
    - Corregir el subtítulo de `ProyectoTrabajosView.vue` para que cargue el nombre y número del proyecto en lugar del UUID crudo.
  - **Criterio de aceptación:** El usuario puede ingresar a la ficha informativa del proyecto y las rutas secundarias muestran títulos legibles.

- [x] **4.2. Refactorización de Componentes en Módulo Calendario**
  - **Archivos:** `src/views/calendario/CalendarioView.vue`.
  - **Detalle:** Reemplazar elementos HTML crudos (`<input>`, `<select>`, modal flotante personalizado) por componentes del sistema de diseño (`Dialog`, `InputText`, `Select`, `PageHeader`, `HelpButton`). Añadir selector de Proyecto/Trabajo opcional para vincular eventos de agenda a trabajos específicos.
  - **Criterio de aceptación:** El Calendario tiene la misma consistencia visual, accesibilidad y paleta de colores que el resto del sistema.

- [x] **4.3. Flujo de Bienvenida y Onboarding para Usuarios Nuevos**
  - **Archivos:** `src/App.vue`, `src/views/WelcomeView.vue`.
  - **Detalle:** Modificar la condición de arranque para que si `localStorage.getItem('eo:welcomed')` no existe, se dirija al usuario a una pantalla de bienvenida amigable que permita:
    1. Importar base de datos legacy (si existe).
    2. O comenzar desde cero configurando los datos básicos de la empresa (Nombre, CUIT, moneda base).
    3. O cargar datos de demostración / seed si está en entorno de prueba.
  - **Criterio de aceptación:** Un usuario nuevo no cae en un panel vacío sin contexto; recibe orientación inicial sobre cómo dar de alta su primera obra.

- [x] **4.4. Acciones Rápidas en Ficha de Cliente**
  - **Archivos:** `src/views/clientes/ClienteDetalleView.vue`.
  - **Detalle:** Mostrar en el detalle del cliente: lista de proyectos activos, saldo de cuenta corriente visible, lista de contactos agregados y botón directo "+ Nuevo Proyecto para este Cliente".
  - **Criterio de aceptación:** La ficha del cliente deja de ser un formulario pasivo y se convierte en un centro de gestión comercial del cliente.

---

## Fase 5: Pruebas Integrales y Verificación de Flujo End-to-End

- [x] **5.1. Verificación de Pruebas Automatizadas Frontend**
  - **Comando:** `pnpm test`, `pnpm typecheck`, `pnpm lint`.
  - **Criterio:** 0 errores de TypeScript y 100% de tests unitarios existentes pasando.

- [x] **5.2. Verificación de Pruebas Automatizadas Backend**
  - **Comando:** `cargo test --workspace`, `cargo clippy --workspace --all-targets`.
  - **Criterio:** Compilación limpia sin warnings ni fallos en pruebas de dominio o casos de uso.

- [x] **5.3. Prueba de Humo de Flujo de Negocio Completo (Manual)**
  - **Paso 1:** Crear Cliente y Contacto.
  - **Paso 2:** Crear Proyecto con presupuesto y Trabajos con ítems de cómputo.
  - **Paso 3:** Emitir Certificado de Avance al 50% y descargarlo en PDF.
  - **Paso 4:** Facturar certificado y cobrar factura; verificar que el dinero ingresa al balance de caja.
  - **Paso 5:** Cargar gasto de materiales imputado al proyecto; verificar impacto en la Caja de Proyecto y en el cálculo de Rentabilidad del Dashboard.
  - **Paso 6:** Registrar asistencia de operarios, cargar adelanto a un empleado, liquidar el lote de sueldos con recargos de feriado y exportar el recibo en PDF.
