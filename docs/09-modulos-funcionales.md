# 09 — Módulos funcionales

> Qué hace cada pantalla: filtros, paginación, acciones, precondiciones y estados. El **cómo** visual
> está en [`16-frontend.md`](./16-frontend.md); las **rutas y atajos** en
> [`10-navegacion-y-atajos.md`](./10-navegacion-y-atajos.md); los **comandos IPC** en
> [`11-contratos-tauri.md`](./11-contratos-tauri.md).

## 1. Contrato común de listado

Todas las pantallas de listado se comportan igual. Esto es un cambio grande respecto del sistema
anterior y hay que respetarlo sin excepciones.

### 1.1 Filtrado y paginación: siempre en servidor

**[FIX]** En el sistema anterior **sólo Movimientos** filtraba y paginaba en el servidor. Los otros
ocho listados hacían `GetAllAsync()` y filtraban con LINQ en memoria, y varios tenían un `CurrentPage`
que nunca cambiaba porque no había botones de navegación. Con 50 registros funciona; con 20.000
movimientos de tres años, no.

En el sistema nuevo **todo listado** usa el mismo contrato:

```rust
pub struct ListQuery<F> {
    pub filtro: F,
    pub page: u32,          // 1-based
    pub page_size: u32,     // 0 = todos
    pub sort_by: Option<String>,
    pub sort_dir: SortDir,  // Asc | Desc
}

pub struct PagedResult<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,   // 0 si page_size == 0
}
```

### 1.2 Tamaños de página

| Valor | Etiqueta i18n |
| --- | --- |
| `10` | `"10"` |
| `30` | `"30"` |
| `50` | `"50"` |
| `100` | `"100"` |
| `0` | `General.PageSizeAll` |

Default: **30**. Se persiste en configuración como `Application.LastPageSize` (doc 14) y aplica a
todos los listados: es una preferencia global, no por pantalla.

`page_size == 0` significa «todos»: no se aplica `LIMIT`, `total_pages` vale `0` y el control de
paginación se oculta.

**[FIX]** La etiqueta del valor `0` era el literal `"Todos"` hardcodeado en `PageSizeConverter`.
Ahora es la clave `General.PageSizeAll`.

### 1.3 Debounce

Todo filtro de texto o numérico dispara la recarga con **300 ms** de debounce; las peticiones
anteriores se cancelan. Los filtros de selección (desplegable, fecha, checkbox) recargan
**inmediatamente**, sin debounce.

**[FIX]** Sólo Movimientos tenía debounce. El resto disparaba una consulta por cada tecla.

Implementación única en el composable `useDebouncedQuery` (doc 16), no repetida por pantalla.

### 1.4 Cambio de filtro

Cambiar **cualquier** filtro resetea `page = 1`. Cambiar `page_size` también. Cambiar el orden
también.

### 1.5 Ordenamiento

**[FIX]** En el sistema anterior el orden era fijo y no configurable en ninguna pantalla. Ahora todas
las columnas marcadas como ordenables en las tablas de §3 permiten ordenar clickeando el
encabezado, con ciclo `asc → desc → orden por defecto`. El orden viaja al servidor; nunca se ordena
en el cliente sobre una página parcial.

Cada pantalla declara su orden por defecto en §3. El backend valida `sort_by` contra una
**allowlist** de nombres de columna por entidad; un valor desconocido devuelve
`AppError::Validation` y no se interpola nunca en el SQL.

### 1.6 Estados de la lista

Cuatro estados mutuamente excluyentes, resueltos en este orden:

| Estado | Condición | Qué se muestra |
| --- | --- | --- |
| `loading` | petición en vuelo y sin datos previos | skeleton de filas |
| `error` | la petición falló | mensaje + botón `General.Retry` que repite la última consulta |
| `empty` | `total_count == 0` y ningún filtro activo | mensaje `<Modulo>.EmptyState` + botón de alta |
| `no-results` | `total_count == 0` con algún filtro activo | mensaje `General.NoResults` + botón `General.ClearFilters` |
| `ready` | hay ítems | la tabla |

**[FIX]** El sistema anterior no distinguía `empty` de `no-results`: mostraba «No hay datos» tanto
cuando la base estaba vacía como cuando el filtro no encontró nada, lo cual hace creer que se
perdieron los datos.

Una recarga con datos ya en pantalla **no** vuelve al estado `loading`: mantiene la tabla y muestra
un indicador de progreso fino en el borde superior, para que la lista no parpadee al tipear.

### 1.7 Borrado

Todo borrado es lógico (INV-12) y pasa por confirmación:

```
título:  General.Delete
cuerpo:  <Modulo>.DeleteConfirm  con params { nombre }
botones: General.Cancel (default, foco inicial) / General.Delete (destructivo)
```

El foco arranca en Cancelar y `Escape` cancela. Tras borrar: toast `General.DeleteSuccess` y recarga
de la página actual. Si la página queda vacía y `page > 1`, se retrocede una página.

Si el borrado choca con una dependencia (regla 5.7 del doc 07), el toast es de error con la clave y
el conteo que devolvió el backend.

**[FIX]** El botón de borrar existía sólo en 4 de los 9 listados aunque el comando estuviera
implementado en todos. En el nuevo sistema **cada fila** tiene su acción de borrado en la columna de
acciones.

### 1.8 Alta y edición

Patrón único: **panel lateral (drawer) sobre la lista**, no ruta, no ventana.

**[FIX]** El sistema anterior mezclaba tres mecanismos: overlay modal a pantalla completa
(Movimientos, Categorías, Tipos), master-detail lateral (Clientes, Obras, Facturas, Empleados) y
overlay sin fondo (Trabajos). Se unifica en uno.

Reglas:

- El drawer se abre con el registro cargado completo (incluidas colecciones hijas).
- Doble click en una fila abre la edición. También la tecla `Enter` sobre la fila enfocada.
- `Escape` cierra el drawer. Si hay cambios sin guardar, pregunta
  `General.DiscardChangesConfirm` antes de cerrar.
- `Ctrl+S` guarda. Al guardar con éxito: toast `General.SaveSuccess`, cierre del drawer y recarga
  de la lista preservando página y filtros.
- Los errores de validación se pintan campo por campo usando el `field` del `FieldError` (doc 07
  §1.1). El primer campo con error recibe el foco.
- El título del drawer es una clave i18n: `<Modulo>.NewTitle` / `<Modulo>.EditTitle`.
  **[FIX]** varios títulos estaban hardcodeados en español (`"Nuevo Movimiento"`,
  `"Editar Cliente"`, `"Editar Trabajo"`).

### 1.9 Columna de acciones

Última columna de toda tabla, fija a la derecha, con iconos y `aria-label` traducido. El orden es
siempre el mismo: acciones específicas del módulo, luego editar, luego borrar.

## 2. Mapa de módulos

| # | Módulo | Ruta | Filtrado | Paginación | Notas |
| --- | --- | --- | --- | --- | --- |
| 1 | Dashboard | `/` | selector de período | — | KPIs y gráficos |
| 2 | Movimientos | `/movimientos` | 8 filtros | sí | el más usado |
| 3 | Clientes | `/clientes` | 3 filtros | sí | + pestaña Cuenta corriente |
| 4 | Obras | `/obras` | 4 filtros | sí | |
| 5 | Trabajos | `/obras/:obraId/trabajos` | 3 filtros | sí | **[FIX]** ahora accesible |
| 6 | Órdenes de trabajo | `/trabajos/:trabajoId/ordenes` | — | no | maestro de ítems |
| 7 | Certificados | `/certificados` | 4 filtros | sí | |
| 8 | Facturas | `/facturas` | 5 filtros | sí | + pagos |
| 9 | Empleados | `/empleados` | 3 filtros | sí | |
| 10 | Asistencia | `/asistencia` | período + búsqueda | no | grilla mes × empleado |
| 11 | Liquidaciones | `/liquidaciones` | 3 filtros | sí | + asistente de 3 pasos |
| 12 | Reportes | `/reportes` | según reporte | no | centro de exportación |
| 13 | Categorías | `/admin/categorias` | 1 filtro | sí | árbol |
| 14 | Tipos de movimiento | `/admin/tipos-movimiento` | 1 filtro | sí | |
| 15 | Configuración | `/configuracion` | — | — | 5 secciones |

## 3. Detalle por módulo

### 3.1 Dashboard

Fuente de los números: doc 06 §9.

**Selector de período:** `Mensual` (default) / `Anual` / `Total`. Se persiste en
`Dashboard.LastPeriod`. Las etiquetas son claves: `Dashboard.Period.Mensual`, `.Anual`, `.Total`.

**[FIX]** Los tres valores estaban como literales `"Mensual"`, `"Anual"`, `"Total"` y viajaban como
string mágico al servicio. Ahora es un enum `PeriodoDashboard` con `TryFrom<&str>`.

**Bloques, en orden:**

1. Cuatro KPI principales: ingresos, gastos, balance, cantidad de movimientos del período, cada uno
   con su variación porcentual contra el período anterior y flecha de tendencia.
2. Cotizaciones del dólar (doc 13 §2). Si la API falló, el bloque no se muestra; **no** se muestra
   un error: es información accesoria.
3. Alertas accionables. Cada una navega a su módulo con el filtro ya aplicado:

| Alerta | Condición | Destino con filtro |
| --- | --- | --- |
| Facturas vencidas | facturas impagas con `dias_mora > 0` | `/facturas?estado=vencida` |
| Obras pausadas | obras en `Pausada` | `/obras?estado=pausada` |
| Liquidaciones pendientes | empleados activos sin liquidación en el período | `/liquidaciones` |

**[FIX]** Las alertas navegaban al módulo sin aplicar ningún filtro; el usuario tenía que filtrar a
mano para encontrar de qué le hablaba la alerta.

4. Gráfico de ingresos vs gastos por mes.
5. Top 5 clientes por facturación del período.
6. Últimos 10 movimientos: fecha (`dd/MM`), concepto, total.
7. Rentabilidad por obra: las 5 con mayor margen y las 5 con menor.

**Modo privacidad:** oculta todos los importes reemplazándolos por `•••••`. Se persiste en
`Dashboard.PrivacyMode`. Atajo `Ctrl+Shift+P`. Los gráficos también se ofuscan (se quitan las
etiquetas del eje de valores).

**Estado del sistema:** versión, cantidad de migraciones aplicadas, fecha del último backup, tamaño
de la base. **[FIX]** El texto `"Saludable"` estaba hardcodeado como valor inicial y a veces quedaba
visible aunque el estado real fuera otro.

### 3.2 Movimientos

La pantalla más usada: es el registro diario de caja (RC-01).

**Filtros:**

| Filtro | Tipo | Default | Debounce | Nota |
| --- | --- | --- | --- | --- |
| `concepto` | texto | vacío | 300 ms | `LIKE %texto%` case-insensitive |
| `tipo_movimiento_id` | select | todos | no | |
| `categoria_id` | select | todos | no | **[NUEVO]** |
| `fecha_desde` | fecha | vacío | no | comparación por fecha civil |
| `fecha_hasta` | fecha | vacío | no | inclusivo |
| `monto_min` | moneda | vacío | 300 ms | compara contra `monto`, no contra el total |
| `monto_max` | moneda | vacío | 300 ms | |
| `obra_id` | select | todos | no | **[NUEVO]** para ver la caja de una obra |
| `cliente_id` | select | todos | no | **[NUEVO]** |
| `moneda` | select | todas | no | **[NUEVO]** |

Botón `General.ClearFilters` que resetea los diez. La cantidad de filtros activos se muestra en un
badge sobre el botón de filtros.

**[NUEVO]** Presets de fecha rápidos: `Hoy`, `Esta semana`, `Este mes`, `Mes anterior`, `Este año`.
Son botones que setean `fecha_desde`/`fecha_hasta`. Claves `General.DateRange.*`.

**Columnas:**

| Columna | Formato | Ordenable | Alineación |
| --- | --- | --- | --- |
| Fecha | `dd/MM/yyyy` | sí | izquierda |
| Concepto | texto, con ellipsis | sí | izquierda |
| Tipo | chip con el color del tipo | sí | izquierda |
| Categoría | chip con el color de la categoría | sí | izquierda |
| Obra | nombre, vacío si no tiene | no | izquierda |
| Monto | moneda | sí | derecha |
| Cantidad | número, `1` se muestra vacío | no | derecha |
| Total | moneda, negrita | sí | derecha |
| Acciones | adjuntos, editar, borrar | — | derecha |

Orden por defecto: `fecha` descendente, y como desempate `created_at` descendente.

**[FIX]** El orden anterior era sólo `Fecha DESC`, sin desempate, así que dos movimientos del mismo
día aparecían en orden arbitrario y cambiaban de posición entre recargas.

**Fila de totales** al pie de la tabla: suma de ingresos, suma de gastos y balance **del filtro
completo, no de la página visible**. Viene calculada del servidor en el mismo comando, en un campo
`resumen` del resultado. **[NUEVO]**

**Acciones de la barra:** nuevo movimiento, exportar (PDF / XLSX / DOCX / CSV / JSON).

**[FIX] Alcance de la exportación:** el sistema anterior exportaba desde Movimientos sólo la
**página visible**, mientras el centro de reportes exportaba **todo sin filtrar**. Ninguna de las dos
es lo que el usuario espera. Ahora la exportación siempre respeta **el filtro activo, ignorando la
paginación**, y el diálogo lo dice explícitamente: `Export.ScopeNotice` con el conteo de registros a
exportar.

**Formulario:** concepto, monto, cantidad, unidad, fecha, tipo, categoría, moneda + cotización,
cliente, obra, trabajo, factura, observaciones, adjuntos. Reglas de dependencia entre selectores:

- Elegir un cliente filtra las obras a las de ese cliente.
- Elegir una obra setea el cliente automáticamente y filtra los trabajos.
- Elegir un trabajo setea la obra y el cliente.
- Cambiar el cliente limpia obra, trabajo y factura si dejaron de ser coherentes.
- El campo de cotización sólo aparece si `moneda == Usd`, precargado con la cotización del día
  (venta del dólar blue) si está disponible.

### 3.3 Clientes

**Filtros:** `texto` (busca en nombre, CUIT y email), `condicion_iva` (select), `solo_con_deuda`
(checkbox, **[NUEVO]**).

**Columnas:** Nombre (ordenable), CUIT, Teléfono, Email, Condición IVA, Obras (conteo),
Deuda (moneda, ordenable, **[NUEVO]**), Acciones (email, editar, borrar).

Orden por defecto: `nombre` ascendente.

**Formulario:** datos del cliente + **grilla editable de contactos** (RC-13): etiqueta, nombre,
email, teléfono, principal. Se agregan y quitan filas sin salir del formulario. A lo sumo un
principal (V-04).

**[FIX]** El sistema anterior tenía la tabla `ClienteContactos` creada y el validador escrito, pero
la UI sólo editaba el `email` único del cliente. El requerimiento RC-13 («varios mails por cliente»)
estaba a medio implementar.

**Pestaña Cuenta corriente:** selector de cliente + KPIs de deuda total y buckets de antigüedad
`0-30 / 31-60 / 61-90 / 90+` (doc 06 §4.6), y la tabla de facturas impagas con número, fecha,
vencimiento, total, pagado, saldo y días de mora. Los días de mora se pintan según el bucket.

**Acción de email:** abre el cliente de correo del sistema con los destinatarios precargados. Si el
cliente tiene contactos, se ofrece elegir a cuáles enviar; el principal viene marcado.

### 3.4 Obras

**Filtros:** `texto` (nombre, número, dirección, localidad), `cliente_id`, `estado` (select con
**los cuatro** estados + Todos), `solo_activas` (checkbox, atajo de `estado in (Activa, Pausada)`).

**Columnas:** Número (ordenable, default), Nombre (ordenable), Cliente, Dirección, Localidad,
Estado (chip), Trabajos (conteo), Rentabilidad (moneda con signo, **[NUEVO]**), Acciones.

Orden por defecto: `numero` descendente (la obra más nueva primero).

**Alta:** el número se precarga con `MAX(numero) + 1` sobre **todas** las obras, incluidas las
borradas lógicamente, para no reutilizar números. **[FIX]** El cálculo anterior tomaba el máximo de
la colección cargada en memoria, que estaba paginada y sin borrados: podía proponer un número ya
usado.

**Acciones de estado:** los botones vienen de `transicionesPermitidas` (doc 08 §7). Finalizar y
cancelar disparan el diálogo de cascada de doc 08 §3.3.

**Navegación:** click en la fila abre el detalle de la obra, que lista sus trabajos
(`/obras/:id/trabajos`) y su caja (movimientos filtrados por esa obra).

### 3.5 Trabajos

**[FIX]** `TrabajosViewModel` y `TrabajosView` existían, estaban registrados en el contenedor de
dependencias, tenían atajos de teclado asignados… y **no** estaban registrados en la navegación: la
pantalla era inalcanzable. El menú mostraba «Certificados», que es otra cosa. En el sistema nuevo
Trabajos vive como sección del detalle de obra y también como listado propio en
`/trabajos` (sin `obraId`).

**Filtros:** `texto` (descripción), `estado` (select con **los cinco** estados + Todos),
`fecha_desde` / `fecha_hasta` sobre `fecha_inicio`, `cliente_id`, `obra_id`.

**[FIX]** El desplegable de estado ofrecía sólo 3 de los 5 estados (`Todos`, `En Curso`,
`Finalizados`): `Presupuestado` y `Cancelado` no se podían filtrar, y el índice 3 (`Pausado`) existía
en el ViewModel pero no en la vista.

**Columnas:** Descripción (ordenable), Obra, Cliente, Fecha inicio (ordenable), Fecha fin,
Presupuesto (moneda), Certificado % (barra de progreso, **[NUEVO]**), Estado (chip), Acciones.

Orden por defecto: `fecha_inicio` descendente.

**[FIX]** El filtro por cliente se aplicaba sobre `ClienteId` del trabajo, que es un campo
desnormalizado. Ahora se resuelve por la obra: `trabajo → obra → cliente`.

### 3.6 Órdenes de trabajo

Maestro de ítems presupuestados de un trabajo. No es un listado independiente: se llega desde el
detalle del trabajo.

**Formulario:** título, número, fecha, observaciones + **grilla de ítems**: descripción, unidad,
cantidad, precio unitario, subtotal (calculado, sólo lectura). Se agregan, reordenan y quitan filas.
Al pie: total del presupuesto.

Sobre la orden se emiten certificados (§3.7). Los ítems no se pueden borrar si ya tienen
certificación: `Conflict` con `State.OrdenTrabajo.ItemCertificado`.

### 3.7 Certificados

Historial de certificaciones de avance (RC-10). Entidad nueva, así que la pantalla es nueva.

**[FIX]** El sistema anterior no tenía tabla de certificados: guardaba
`porcentaje_anterior` / `porcentaje_actual` en el ítem y los sobreescribía en cada certificación. La
pantalla «Certificados» mostraba en realidad las **órdenes de trabajo**, no certificados, y no había
forma de reconstruir la historia. El PDF de la certificación anterior era la única copia.

**Filtros:** `obra_id`, `trabajo_id`, `fecha_desde` / `fecha_hasta`, `cliente_id`.

**Columnas:** N.º (ordenable), Fecha (ordenable, default desc), Obra, Trabajo, Orden, Total actual
(moneda), Avance acumulado (barra), Acciones (ver PDF, exportar, borrar si es el último).

**Vista de detalle:** encabezado con obra, trabajo, cliente y fecha; la grilla de los 9 campos por
ítem (doc 12 §4); y el bloque de totales con ajuste UOCRA y otros descuentos.

**Alta:** se elige la orden de trabajo y el sistema precarga cada ítem con su porcentaje acumulado
histórico y un campo editable para el porcentaje de esta certificación, con el subtotal
recalculándose en vivo. El acumulado que superaría 100 % se marca en rojo antes de guardar (V-09,
doc 07 §5.3).

### 3.8 Facturas

**Filtros:** `texto` (número y nombre de cliente), `cliente_id`, `estado` (multi-select),
`fecha_desde` / `fecha_hasta`, `solo_impagas` (checkbox), `solo_vencidas` (checkbox).

**Columnas:** Número (ordenable), Fecha (ordenable), Vencimiento (**[NUEVO]**), Cliente, Estado
(chip), Subtotal, IVA, Total, Pagado (**[NUEVO]**), Saldo (**[NUEVO]**), Mora en días
(**[NUEVO]**), Acciones.

Orden por defecto: `fecha` descendente.

**Sub-panel de pagos:** dentro del drawer de la factura, la lista de pagos con fecha, monto, medio y
observaciones, más el saldo pendiente destacado. Alta de pago con validación contra el saldo
(doc 07 §5.4). Cada alta o baja de pago recalcula el estado en la misma transacción (doc 08 §2.4).

**Acciones de estado:** emitir, anular, volver a borrador, según `transicionesPermitidas`.

### 3.9 Empleados

**Filtros:** `texto` (nombre, DNI, cargo), `activo` (select Todos / Activos / Inactivos, default
**Activos**), `cargo` (select).

**[FIX]** El default anterior era «Todos», así que la lista se llenaba de empleados que ya no
trabajan y había que filtrar cada vez.

**Columnas:** Nombre (ordenable), DNI, Cargo, Tarifa diaria (moneda), Sueldo base (moneda),
Frecuencia de pago, Fecha ingreso (ordenable), Activo (switch), Acciones (email, WhatsApp, editar,
borrar).

Orden por defecto: `nombre` ascendente.

**Formulario:** datos personales, laborales, tarifa diaria, sueldo base, frecuencia de pago y los
tres multiplicadores (sábado, domingo, feriado). Los multiplicadores se precargan con los defaults de
configuración (doc 14) y se puede sobreescribir por empleado.

**[FIX] Mensaje de WhatsApp:** el texto
`"Hola {Nombre}, me pongo en contacto contigo desde ElectroObraApp."` estaba hardcodeado en el
ViewModel. Pasa a ser una plantilla configurable (`Communication.WhatsAppTemplate`) con
la clave i18n `Communication.WhatsAppDefault` como valor inicial, y usa `{nombre}` como marcador.

### 3.10 Asistencia

Grilla de asistencia mensual: una fila por empleado activo, una columna por día del mes (RC-06).

**Encabezados de columna:** número de día **y** la inicial del día de la semana (`L M M J V S D`).
Los sábados y domingos tienen fondo diferenciado; los feriados del calendario (doc 13 §3) llevan un
punto y el nombre del feriado en el tooltip.

**[FIX]** El encabezado anterior mostraba sólo el número, así que no se veía dónde caían los fines de
semana — justo la información que hace falta para aplicar los multiplicadores.

**Navegación de período:** mes anterior / mes siguiente / selector de mes y año / botón «Hoy».
**[NUEVO]** También vista quincenal (1–15 y 16–fin), porque las liquidaciones suelen ser quincenales
(RC-05) y 31 columnas no entran en pantalla.

**Búsqueda de empleado:** filtro de texto sobre las filas, con debounce. Con 40 peones la grilla no
se navega sin buscador.

**Ciclo de click.** Un click en una celda avanza al siguiente valor:

```
(vacío) → Completa → Media → Falta → FaltaJustificada → Feriado → (vacío) → …
```

**[FIX]** El ciclo anterior era `(vacío) → Completa → Media → Falta → FaltaJustificada → Feriado →
Completa`: **una vez creado el registro no había forma de volver a vacío**. Un click de más en la
celda equivocada dejaba una asistencia falsa que sólo se podía arreglar desde la base. Volver a
vacío ahora borra el registro (borrado lógico).

**Guardado:** inmediato por celda (`upsert`), sin botón de guardar. La celda muestra un estado
optimista y revierte si el servidor falla, con un toast de error. Se conserva del sistema anterior
porque cargar la asistencia de 40 personas por 30 días es un trabajo de clicks y un guardado por
lotes se pierde.

**Selección múltiple** **[NUEVO]**: se puede arrastrar sobre varias celdas o seleccionar una fila
completa y aplicar un valor de una vez, desde un menú contextual. Marcar 26 días de jornada completa
click por click es la queja más obvia del flujo actual.

**Colores por tipo de jornada:** vienen de tokens del tema, no de literales.
**[FIX]** Estaban hardcodeados en el ViewModel: `#2ecc71`, `#f1c40f`, `#e74c3c`, `#e67e22`,
`#3498db`, `#3a3a3a`. Pasan a `--eo-attendance-completa`, `-media`, `-falta`, `-falta-justificada`,
`-feriado`, `-vacio`, definidos por tema claro y oscuro.

**Símbolos y etiquetas:** claves `Attendance.TipoJornada.<Tipo>.Symbol` y `.Label`. Se conservan.

**Columna de resumen** a la derecha de cada fila **[NUEVO]**: días completos, medios, faltas y el
total de jornadas equivalentes del período. Es el número que después usa la liquidación, así que verlo
antes evita sorpresas.

**Guarda:** no se puede cargar asistencia posterior a la `fecha_egreso` del empleado
(doc 08 §5.3).

### 3.11 Liquidaciones

**Listado histórico.** Filtros: `empleado_id`, `fecha_desde` / `fecha_hasta` sobre el período,
`solo_sin_pdf` (checkbox, **[NUEVO]**).

Columnas: Empleado (ordenable), Desde, Hasta (ordenable, default desc), Días, Total bruto,
Adelantos, Total neto (negrita, rojo si es negativo), Acciones (PDF, email, WhatsApp, editar,
borrar).

**[FIX]** El listado histórico no tenía **ningún** filtro ni paginación: se cargaba completo. A dos
liquidaciones por mes por empleado, con 20 empleados, son 480 filas por año.

**Asistente de 3 pasos.** Se conserva la estructura del sistema anterior, que estaba bien resuelta.

**Paso 1 — Período y empleados**

- Fechas desde/hasta. Default: `hoy - 15 días` a `hoy`. **[NUEVO]** Presets `Quincena actual`,
  `Quincena anterior`, `Mes actual`, `Mes anterior`, calculados sobre el calendario real.
- Grilla de empleados activos con casilla de selección, nombre, cargo y tarifa diaria.
- Botones `Seleccionar todos` / `Limpiar selección`.
- **[NUEVO]** Columna con las jornadas registradas en el período, para no incluir a alguien que no
  tiene asistencia cargada.
- Guardas para avanzar: `fecha_inicio <= fecha_fin` (`Settlements.Wizard.InvalidPeriod`) y al menos
  un empleado seleccionado (`Settlements.Wizard.NoEmployeesSelected`).

**Paso 2 — Revisión**

- Se llama a la sugerencia de liquidación (doc 06 §6) para cada empleado seleccionado y se arma la
  vista previa.
- Grilla editable. Columnas: Empleado (sólo lectura), **Días trabajados (editable)**,
  **Tarifa aplicada (editable, [NUEVO])**, Total bruto, Adelantos, Total neto.
- **[NUEVO]** Cada fila se puede expandir para ver el desglose: jornadas por tipo, recargos de
  sábado/domingo/feriado y la lista de adelantos detectados con fecha, concepto e importe, cada uno
  con su casilla para excluirlo de esta liquidación. Sin esto el usuario ve un número de adelantos y
  no puede saber de dónde sale — que es justo lo que pide RC-02.
- Botón `Recalcular` que vuelve a llamar la sugerencia preservando los días y la tarifa editados.
- **[NUEVO]** Advertencia por fila si un adelanto del período ya fue descontado en otra liquidación
  (doc 07 §5.5): se muestra tachado y no suma.

**Paso 3 — Confirmación**

- Grilla de sólo lectura: empleado, días, total neto.
- Totales del lote: bruto, adelantos, neto.
- **[NUEVO]** Casilla «Generar los PDF al confirmar».
- Botón `Confirmar`. Antes de persistir **recalcula todo** una última vez, para que un cambio hecho
  en otra pestaña no pase inadvertido.
- Se crea el lote en **una sola transacción**: si una liquidación falla, ninguna se guarda.
- Al confirmar se registran las filas de `liquidacion_adelantos` que consumen los adelantos
  (doc 03), lo que impide el doble descuento.
- Toast `Settlements.Wizard.BatchSuccess` con la cantidad, y se vuelve al listado.

**Indicador de paso:** `Settlements.Wizard.StepIndicator` con formato `{paso}/{total}`. Los pasos
se pueden revisitar hacia atrás sin perder lo editado.

**Guarda de edición:** una liquidación con PDF ya generado no admite cambios de importe
(doc 08 §5.2).

### 3.12 Reportes

Centro de exportación. Cada reporte es una tarjeta con su propio conjunto de parámetros y sus
formatos disponibles. Layouts exactos en [`12-reportes-y-exportaciones.md`](./12-reportes-y-exportaciones.md).

| Reporte | Parámetros | Formatos |
| --- | --- | --- |
| Movimientos | los mismos 10 filtros del módulo | PDF, XLSX, DOCX, CSV, JSON |
| Caja por período | desde, hasta, agrupación (día/semana/mes) | PDF, XLSX, CSV |
| Rentabilidad por obra | desde, hasta, cliente | PDF, XLSX |
| Cuenta corriente | cliente, incluir pagadas | PDF, XLSX |
| Antigüedad de deuda | fecha de corte | PDF, XLSX |
| Liquidación | liquidación | PDF |
| Certificado | certificado | PDF |
| Asistencia mensual | mes, año, empleados | PDF, XLSX |
| Base completa | — | JSON (respaldo, doc 13 §5) |

**[FIX]** El centro de reportes anterior exportaba **todos** los movimientos sin permitir filtrar, y
guardaba el archivo directamente en el Escritorio con un nombre generado. Ahora cada reporte pide sus
parámetros y abre el diálogo de guardado del sistema con un nombre propuesto.

Mientras exporta, la tarjeta muestra progreso y el resto queda deshabilitado. Al terminar: toast con
la ruta y un botón `General.OpenFolder`.

### 3.13 Categorías

Administración con jerarquía (RC-04).

**Filtro:** `texto` sobre el nombre.

**Vista:** árbol de dos niveles con la categoría padre y sus hijas, más el conteo de movimientos de
cada una. Columnas: Nombre, Descripción, Color (muestra), Icono, Movimientos (conteo), Acciones.

**Formulario:** nombre, descripción, categoría padre, color, icono. Ciclos y profundidad se validan
según doc 07 §5.2.

**[FIX]** La jerarquía no existía: todas las categorías eran planas. RC-04 pide agrupar
(«materiales» conteniendo «cables», «caños», …).

### 3.14 Tipos de movimiento

**Filtro:** `texto` sobre el nombre.

**Columnas:** Nombre, Descripción, Es ingreso (switch), Es sistema (candado), Movimientos (conteo),
Acciones.

Los cuatro tipos de sistema muestran un candado, no se pueden borrar y su marca `es_ingreso` es
inmutable (doc 08, doc 07 §5.6). El nombre sí se puede editar.

**[FIX]** El botón de borrar se deshabilitaba en la vista, pero la protección no existía en el
backend: una llamada directa al servicio borraba un tipo de sistema.

### 3.15 Configuración

Cinco secciones. El catálogo completo de claves está en
[`14-configuracion-e-i18n.md`](./14-configuracion-e-i18n.md).

| Sección | Contenido |
| --- | --- |
| General | idioma, tema, formato de fecha, moneda por defecto, tamaño de página por defecto |
| Negocio | **[NUEVO]** datos del contratista, logo, multiplicadores por defecto, días por frecuencia de pago, días de vencimiento de factura, IVA sugerido |
| Liquidaciones | feriados (sincronización y alta manual), multiplicadores, plantilla del PDF |
| Integraciones | URL y timeout de la API del dólar y de feriados, plantillas de email y WhatsApp |
| Sistema | migraciones aplicadas y pendientes, backups (crear, restaurar, verificar), exportar/importar JSON, ruta de datos, nivel de log |

**[FIX]** Los títulos de las secciones estaban hardcodeados en español dentro del XAML
(`"General"`, `"Personalización"`, `"Liquidaciones"`, `"Sistema"`). Ahora son claves
`Settings.Section.*`.

**[FIX]** El contratista `"PABLO BAEZ"` y el logo `"GENERCON"` estaban hardcodeados en el generador
de PDF. Pasan a la sección Negocio.

Los cambios se aplican con un botón explícito (`Settings.Apply`) y muestran confirmación durante
3 segundos. Salir con cambios sin aplicar pregunta antes.

## 4. Acciones globales

| Acción | Dónde | Comportamiento |
| --- | --- | --- |
| Búsqueda global | encabezado, `Ctrl+K` | paleta de comandos (doc 10 §4) |
| Notificaciones | encabezado | toasts apilados abajo a la derecha, 4 s los de éxito, persistentes los de error |
| Cambio de tema | encabezado y Configuración | claro / oscuro / sistema, persistido |
| Modo privacidad | encabezado, `Ctrl+Shift+P` | oculta importes en toda la app |
| Backup manual | Configuración | `VACUUM INTO` + verificación de integridad (doc 13 §4) |

## 5. Reglas transversales de UI

1. **Ningún importe se formatea a mano.** Siempre por el componente `MoneyText`, que aplica el
   formato de moneda de la configuración regional y el modo privacidad.
2. **Ninguna fecha se formatea a mano.** Siempre por `DateText`, que usa el formato configurado y
   convierte de UTC a hora local para mostrar (doc 04 §3).
3. **Ningún texto visible es un literal.** Todo pasa por `$t()`. Un literal en un `.vue` es un
   error de revisión.
4. **Ningún color es un literal.** Todo por tokens de Tailwind del tema (doc 16).
5. **Toda tabla es navegable por teclado:** flechas para moverse, `Enter` para editar, `Delete` para
   borrar con confirmación.
6. **Toda acción destructiva se confirma** y es reversible mientras el borrado sea lógico.
7. **Toda operación de más de 400 ms muestra progreso.**
8. **Todo error se muestra al usuario** con su mensaje traducido; ninguno se traga en silencio,
   salvo las degradaciones explícitamente documentadas (cotizaciones y feriados, doc 13).

## 6. Inventario de correcciones

Resumen de lo que este documento arregla del sistema anterior, para que el implementador no lo
reproduzca por imitar el código viejo.

| # | Problema | Sección |
| --- | --- | --- |
| 1 | Filtrado y paginación en memoria en 8 de 9 listados | §1.1 |
| 2 | `CurrentPage` sin controles de navegación | §1.1 |
| 3 | Debounce sólo en Movimientos | §1.3 |
| 4 | Sin ordenamiento configurable en ninguna pantalla | §1.5 |
| 5 | `empty` y `no-results` indistinguibles | §1.6 |
| 6 | Botón de borrar ausente en 5 listados | §1.7 |
| 7 | Tres mecanismos distintos de edición | §1.8 |
| 8 | Títulos de formulario hardcodeados en español | §1.8 |
| 9 | `"Todos"` del selector de tamaño de página sin i18n | §1.2 |
| 10 | Alertas del dashboard que navegan sin filtrar | §3.1 |
| 11 | Período del dashboard como string mágico | §3.1 |
| 12 | Exportación de Movimientos limitada a la página visible | §3.2 |
| 13 | Centro de reportes exportando todo sin filtros | §3.12 |
| 14 | Orden de movimientos sin desempate | §3.2 |
| 15 | Contactos múltiples de cliente sin UI | §3.3 |
| 16 | Numeración de obra calculada sobre datos paginados | §3.4 |
| 17 | Pantalla de Trabajos inalcanzable | §3.5 |
| 18 | Filtro de estado de trabajos con 3 de 5 estados | §3.5 |
| 19 | Certificados sin historial | §3.7 |
| 20 | Asistencia sin poder volver a vacío | §3.10 |
| 21 | Encabezados de asistencia sin día de la semana | §3.10 |
| 22 | Colores de asistencia hardcodeados | §3.10 |
| 23 | Sin carga masiva de asistencia | §3.10 |
| 24 | Liquidaciones sin filtros ni paginación | §3.11 |
| 25 | Adelantos sin desglose visible | §3.11 |
| 26 | Categorías sin jerarquía | §3.13 |
| 27 | Tipos de sistema protegidos sólo en la UI | §3.14 |
| 28 | Contratista y logo hardcodeados | §3.15 |
| 29 | Secciones de configuración hardcodeadas | §3.15 |
| 30 | Plantilla de WhatsApp hardcodeada | §3.9 |
