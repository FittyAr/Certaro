# Especificación Técnica: Módulo Comercial, Obras y Certificaciones

## 1. Diagnóstico Actual

El módulo comercial agrupa la relación con Clientes, la presupuestación de Proyectos/Obras, la ejecución por Trabajos, la emisión de Certificados de Avance y las Facturas. Se identificaron las siguientes deficiencias:

1. **Mapeo Cruzado de Columnas en Árbol de Obras ([ProyectosTreeTable.vue](../../src/components/domain/ProyectosTreeTable.vue)):**
   - En las líneas 74-82, al formatear los nodos hijos de tipo Trabajo:
     - `localidad: trab.fechaInicio` $\rightarrow$ Muestra una fecha (ej: `2026-08-10`) en la columna con encabezado "Localidad".
     - `rentabilidad: trab.presupuesto` $\rightarrow$ Muestra el presupuesto presupuestado del trabajo en la columna con encabezado "Rentabilidad", formateado además con `MoneyText colored` (en verde como si fuera ganancia neta).
2. **Imposibilidad de acceder al Detalle del Proyecto:**
   - En [ProyectosTreeTable.vue](../../src/components/domain/ProyectosTreeTable.vue), ni el clic simple ni el doble clic abren la ficha [ProyectoDetalleView.vue](../../src/views/proyectos/ProyectoDetalleView.vue). El menú contextual tiene opciones para Trabajos, Caja y Kanban, pero ninguna para ver el detalle de la obra.
3. **Subtítulo con UUID Crudo en Trabajos de Proyecto:**
   - En [ProyectoTrabajosView.vue:40](../../src/views/proyectos/ProyectoTrabajosView.vue#L40), el subtítulo muestra literalmente `:subtitle="String(proyectoId)"`, exponiendo una cadena UUID técnica incomprensible para el usuario.
4. **Falta de Trazabilidad entre Certificados y Facturas:**
   - Emitir un Certificado de Avance en [OrdenDetalleView.vue](../../src/views/ordenes/OrdenDetalleView.vue) finaliza el flujo sin enlace a Facturación. No existe un botón de "Facturar Certificado".
5. **Certificados sin Descarga Directa:**
   - [CertificadoDetalleView.vue](../../src/views/certificados/CertificadoDetalleView.vue) no cuenta con botón para descargar o imprimir el certificado en PDF.

---

## 2. Solución Propuesta

### 2.1. Corrección de la Tabla Jerárquica de Proyectos y Trabajos
En [ProyectosTreeTable.vue](../../src/components/domain/ProyectosTreeTable.vue):
- Corregir el objeto `data` de los nodos hijos (`isTrabajo`):
  ```typescript
  trabajos.map((trab) => ({
    key: trab.id,
    data: {
      isTrabajo: true,
      trabajo: trab,
      numero: '—',
      nombre: trab.descripcion,
      clienteNombre: trab.clienteNombre,
      localidad: '—', // El trabajo no tiene localidad propia; la localidad pertenece al proyecto padre.
      estado: trab.estado,
      trabajosCount: null,
      presupuesto: trab.presupuesto,
      rentabilidad: null, // No confundir presupuesto con rentabilidad.
      proyecto: null,
    },
    leaf: true,
  }))
  ```
- En la plantilla:
  - En la columna "Localidad", los trabajos muestran un guión `—`.
  - En la columna "Rentabilidad", los trabajos muestran `—` (o si se desea mostrar el presupuesto, se crea una columna explícita o se indica con etiqueta `Presupuesto: $X`).
- **Navegación:**
  - Agregar evento `@row-dblclick` o enlace en el nombre del proyecto que navegue a `{ name: 'proyecto-detalle', params: { proyectoId: node.data.proyecto.id } }`.
  - En el menú contextual, agregar la opción `label: t('General.ViewDetails'), icon: 'pi pi-info-circle'`.

### 2.2. Corrección de `ProyectoTrabajosView.vue`
- Cargar los datos del proyecto al montar la vista (`await proyectosStore.fetchOne(proyectoId.value)`).
- Asignar en el encabezado:
  ```html
  <PageHeader 
    :title="$t('Menu.Trabajos')" 
    :subtitle="proyecto ? `${proyecto.numero} · ${proyecto.nombre}` : undefined"
  >
  ```
- Agregar botón de retorno y permitir hacer clic en cada trabajo de la tabla para abrir `trabajo-detalle`.

### 2.3. Emisión Rápida de Factura desde Certificado de Avance
En [CertificadoDetalleView.vue](../../src/views/certificados/CertificadoDetalleView.vue):
- Agregar en `#actions` del encabezado:
  ```html
  <Button variant="outline" :disabled="!certificado" @click="exportarPdf()">
    <AppIcon name="file-down" :size="16" />
    {{ $t('Reportes.ExportarPdf') }}
  </Button>
  <Button :disabled="!certificado" @click="facturarCertificado()">
    <AppIcon name="receipt" :size="16" />
    {{ $t('Certificados.Facturar') }}
  </Button>
  ```
- La función `facturarCertificado()` redirige a la ruta `/facturas` pasando por `query` o `state` los valores del certificado:
  - `clienteId`: ID del cliente de la obra.
  - `subtotal`: Monto neto certificado.
  - `observaciones`: "Correspondiente a Certificado Nº X de obra Y".
- En `FacturasView.vue`, si se detectan estos query params al abrir, se dispara automáticamente el drawer de creación con estos valores precargados.

### 2.4. Enriquecimiento de la Ficha del Cliente ([ClienteDetalleView.vue](../../src/views/clientes/ClienteDetalleView.vue))
- Añadir sección con el listado de proyectos activos del cliente.
- Mostrar una tarjeta con el Saldo Actual de Cuenta Corriente (obtenido de `useComercialStore`).
- Agregar botón `+ Nuevo Proyecto para este Cliente` que abra el alta de proyecto con el cliente seleccionado.
- Listar los contactos registrados del cliente con sus teléfonos y correos.

---

## 3. Modificaciones de Archivos y Componentes

### Frontend
- **[src/components/domain/ProyectosTreeTable.vue](../../src/components/domain/ProyectosTreeTable.vue):** Corrección de bindings de columnas, eventos de navegación y menú contextual.
- **[src/views/proyectos/ProyectoTrabajosView.vue](../../src/views/proyectos/ProyectoTrabajosView.vue):** Carga de datos del proyecto para subtítulo y tabla clickeable.
- **[src/views/certificados/CertificadoDetalleView.vue](../../src/views/certificados/CertificadoDetalleView.vue):** Botón directo de exportación PDF y acción "Facturar Certificado".
- **[src/views/clientes/ClienteDetalleView.vue](../../src/views/clientes/ClienteDetalleView.vue):** Proyectos vinculados, saldo de cuenta corriente y contactos visibles.
