# 10 — Navegación y atajos

> Define `src/router/`, `src/components/layout/` y `src/composables/useShortcuts.ts`. El contenido de
> cada pantalla está en [`09-modulos-funcionales.md`](./09-modulos-funcionales.md).

## 1. Estructura del shell

```
AppLayout
├── AppHeader          (fija, 56 px)
│   ├── botón de menú (colapsa/expande la barra lateral)
│   ├── título de la sección actual
│   ├── buscador / paleta de comandos (Ctrl+K)
│   └── acciones: tema, modo privacidad, notificaciones
├── AppSidebar         (260 px expandida / 56 px compacta)
│   └── menú agrupado en 4 secciones
├── <RouterView>       (área de contenido, con scroll propio)
└── AppStatusBar       (fija, 28 px)
    ├── botón volver (si hay historial)
    ├── ruta actual (migas)
    └── estado de conexión / última sincronización
```

El área de contenido es la **única** con scroll: encabezado, barra lateral y barra de estado quedan
fijas. Las tablas tienen su propio encabezado adherido.

## 2. Rutas

15 rutas de navegación más las anidadas de detalle. Las claves de ruta del sistema anterior se
conservan como `path` para que los enlaces y los atajos sigan siendo los mismos.

| # | `name` | `path` | Título (clave i18n) | Grupo | Icono | Ctrl+N |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | `dashboard` | `/` | `Menu.Dashboard` | Operación | `layout-dashboard` | — |
| 2 | `movimientos` | `/movimientos` | `Menu.Movimientos` | Operación | `arrow-left-right` | nuevo movimiento |
| 3 | `clientes` | `/clientes` | `Menu.Clientes` | Comercial | `users` | nuevo cliente |
| 4 | `obras` | `/obras` | `Menu.Obras` | Comercial | `building-2` | nueva obra |
| 5 | `trabajos` | `/trabajos` | `Menu.Trabajos` | Comercial | `hammer` | nuevo trabajo |
| 6 | `certificados` | `/certificados` | `Menu.Certificados` | Comercial | `file-badge` | nuevo certificado |
| 7 | `facturas` | `/facturas` | `Menu.Facturas` | Comercial | `receipt` | nueva factura |
| 8 | `empleados` | `/empleados` | `Menu.Empleados` | Personal | `id-card` | nuevo empleado |
| 9 | `asistencia` | `/asistencia` | `Menu.Asistencia` | Personal | `calendar-check` | — |
| 10 | `liquidaciones` | `/liquidaciones` | `Menu.Liquidaciones` | Personal | `banknote` | abrir asistente |
| 11 | `reportes` | `/reportes` | `Menu.Reports` | Sistema | `file-chart-column` | — |
| 12 | `categorias` | `/admin/categorias` | `Menu.Categories` | Sistema | `tags` | nueva categoría |
| 13 | `tipos-movimiento` | `/admin/tipos-movimiento` | `Menu.MovementTypes` | Sistema | `list-tree` | nuevo tipo |
| 14 | `configuracion` | `/configuracion` | `Menu.Settings` | Sistema | `settings` | — |
| 15 | `seed` | `/dev/seed` | `Menu.Seed` | Sistema | `database` | — |

`seed` sólo aparece si `Application.SeedEnabled` es `true`; en compilación de release la ruta no se
registra.

**[FIX]** `Menu.Trabajos` existía en `es.json` pero el menú no lo usaba porque la pantalla no estaba
registrada. Ahora la clave se usa.

### 2.1 Rutas anidadas de detalle

Se navega a ellas, no son drawers: son pantallas con su propio contexto.

| `name` | `path` | Padre en las migas |
| --- | --- | --- |
| `obra-detalle` | `/obras/:obraId` | Obras |
| `obra-trabajos` | `/obras/:obraId/trabajos` | Obras › {nombre de obra} |
| `obra-caja` | `/obras/:obraId/caja` | Obras › {nombre de obra} |
| `trabajo-detalle` | `/trabajos/:trabajoId` | Trabajos |
| `trabajo-ordenes` | `/trabajos/:trabajoId/ordenes` | Trabajos › {descripción} |
| `orden-detalle` | `/ordenes/:ordenId` | Trabajos › … › Orden {número} |
| `certificado-detalle` | `/certificados/:certificadoId` | Certificados |
| `cliente-detalle` | `/clientes/:clienteId` | Clientes |
| `cliente-cuenta` | `/clientes/:clienteId/cuenta-corriente` | Clientes › {nombre} |
| `liquidacion-detalle` | `/liquidaciones/:liquidacionId` | Liquidaciones |

**[FIX]** El sistema anterior tenía la ruta `liquidacion-edit` registrada y sin un solo llamador:
nadie navegaba nunca a ella. No se replica; la reemplaza `liquidacion-detalle`.

La cuenta corriente pasa de ser una pestaña dentro de Clientes a una ruta propia, para que se pueda
enlazar y compartir.

### 2.2 Regla: alta y edición no son rutas

El alta y la edición de un registro de listado abren un **drawer** sobre la lista (doc 09 §1.8), no
una ruta. Motivo: al cerrar hay que volver a la lista con la misma página, los mismos filtros y el
mismo scroll, y una ruta obliga a serializar todo ese estado.

Excepción: las entidades con hijos (Obra, Trabajo, Orden de trabajo, Certificado) **sí** tienen ruta
de detalle, porque son pantallas de trabajo, no formularios.

El estado del drawer se refleja en la query string (`?edit=<uuid>` o `?new=1`) para que `F5` no
pierda el formulario abierto. Los datos del formulario no se serializan; se recargan del backend.

### 2.3 Filtros en la URL

Los filtros de cada listado viven en la query string, con los mismos nombres que el DTO de filtro en
camelCase: `/movimientos?concepto=cable&tipoMovimientoId=…&fechaDesde=2026-01-01&page=2&pageSize=50`.

Esto habilita tres cosas que el sistema anterior no tenía: enlaces profundos desde las alertas del
dashboard (doc 09 §3.1), `F5` sin perder el filtro, y el botón de volver del navegador
funcionando como espera cualquiera.

Sólo se serializan los filtros con valor distinto del default, para no ensuciar la URL.

### 2.4 Guardas

| Guarda | Comportamiento |
| --- | --- |
| `beforeEach` global | setea `document.title` con `{sección} — {nombre de la app}` y actualiza las migas |
| `beforeEnter` de rutas con `:id` | valida que el parámetro sea un UUID; si no, redirige al listado padre |
| `beforeRouteLeave` con cambios sin guardar | pregunta `General.DiscardChangesConfirm` |
| ruta desconocida | redirige a `dashboard`; no hay pantalla 404 |
| `seed` con `SeedEnabled = false` | la ruta no existe |

### 2.5 Historial

Se usa el historial del navegador (`createWebHistory` sobre el protocolo de Tauri). El botón
«Volver» de la barra de estado es `router.back()`, deshabilitado cuando no hay entrada previa dentro
de la aplicación.

**[FIX]** El sistema anterior mantenía su propia pila de rutas en memoria, con la particularidad de
que `GoBack` no reapilaba, así que el historial se consumía y no se podía ir hacia adelante.

### 2.6 Estado persistido

| Qué | Clave de configuración | Default |
| --- | --- | --- |
| Última ruta visitada | `Application.LastRoute` | `dashboard` |
| Barra lateral expandida | `Application.SidebarExpanded` | `true` |
| Tamaño de página | `Application.LastPageSize` | `30` |
| Tema | `Application.Theme` | `system` |
| Modo privacidad | `Dashboard.PrivacyMode` | `false` |
| Período del dashboard | `Dashboard.LastPeriod` | `Mensual` |

**[NUEVO]** La última ruta no se guardaba: la aplicación siempre arrancaba en el dashboard. Ahora se
restaura, salvo que la ruta ya no exista o requiera un `:id` que fue borrado, en cuyo caso cae al
dashboard.

## 3. Menú

### 3.1 Grupos

| Grupo | Clave i18n | Rutas |
| --- | --- | --- |
| Operación | `Menu.Group.Operacion` | `dashboard`, `movimientos` |
| Comercial | `Menu.Group.Comercial` | `clientes`, `obras`, `trabajos`, `certificados`, `facturas` |
| Personal | `Menu.Group.Personal` | `empleados`, `asistencia`, `liquidaciones` |
| Sistema | `Menu.Group.Sistema` | `reportes`, `categorias`, `tipos-movimiento`, `configuracion`, `seed` |

Se conserva la agrupación del sistema anterior, que ya reflejaba bien cómo se usa la aplicación. El
único cambio es `trabajos`, que se agrega a Comercial.

### 3.2 Definición

El menú **no** se escribe en el template: se deriva de una única estructura de datos.

```ts
// src/router/menu.ts
export interface MenuItem {
  route: string;            // name de la ruta
  labelKey: string;         // clave i18n
  icon: string;             // nombre del icono
  shortcut?: string;        // "Ctrl+1"
  devOnly?: boolean;
}

export interface MenuGroup {
  labelKey: string;
  items: MenuItem[];
}

export const MENU: MenuGroup[] = [ /* la tabla de §3.1 */ ];
```

**[FIX]** El menú anterior estaba escrito ítem por ítem en el XAML, con el icono, el título y el
comando repetidos 14 veces. Agregar una pantalla implicaba tocar tres archivos y era exactamente el
motivo por el cual Trabajos quedó sin entrada de menú.

El ítem activo se resuelve comparando la ruta actual con `route`, considerando también las rutas
anidadas: estando en `/obras/:id/trabajos` se resalta **Obras**, no Trabajos, porque se llegó por
ahí. La regla es: se resalta el ítem del ancestro más cercano presente en el menú, calculado desde la
jerarquía de migas.

### 3.3 Barra lateral compacta

Con la barra colapsada se muestran sólo los iconos y los encabezados de grupo desaparecen (queda un
separador). Cada icono muestra el título en un tooltip con su atajo.

## 4. Atajos de teclado

### 4.1 Globales

| Atajo | Acción | Nota |
| --- | --- | --- |
| `Ctrl+K` | abre la paleta de comandos | también `Ctrl+P` y `/` cuando el foco no está en un input |
| `Ctrl+N` | nuevo registro en el módulo actual | ver la columna Ctrl+N de §2 |
| `Ctrl+S` | guarda el formulario o el drawer abierto | sin formulario abierto no hace nada |
| `F5` | recarga los datos del módulo actual | no recarga la ventana |
| `Escape` | cascada de cierre | §4.3 |
| `Ctrl+1` … `Ctrl+9` | navega a la n-ésima ruta del menú | §4.2 |
| `Ctrl+B` | colapsa/expande la barra lateral | **[NUEVO]** |
| `Ctrl+Shift+P` | modo privacidad | **[NUEVO]** |
| `Ctrl+,` | Configuración | **[NUEVO]**, convención habitual |
| `Alt+←` / `Alt+→` | atrás / adelante en el historial | **[NUEVO]** |
| `F1` | ayuda de atajos | **[NUEVO]**, abre el listado de esta sección |

Los atajos se registran en un único lugar (`useShortcuts`) y se desactivan cuando el foco está en un
campo de texto, salvo `Escape`, `Ctrl+S` y `Ctrl+K`.

**[FIX]** `F5` no tenía manejador en Obras, Asistencia, Categorías, Tipos de movimiento, Reportes ni
Seed: en 6 de 15 pantallas la tecla no hacía nada. `Ctrl+N` faltaba en Obras, Categorías, Tipos y
Certificados. `Ctrl+S` no cubría Obras ni los formularios en línea. En el sistema nuevo, cada módulo
declara sus manejadores en su propio composable y el shell no tiene un `switch` sobre el tipo del
ViewModel actual: si un módulo no declara un manejador, el atajo simplemente no aplica y eso es
explícito, no un olvido.

### 4.2 Atajos numéricos

`Ctrl+1` a `Ctrl+9` navegan a las primeras nueve rutas del menú, en el orden en que aparecen:

| Atajo | Ruta |
| --- | --- |
| `Ctrl+1` | `dashboard` |
| `Ctrl+2` | `movimientos` |
| `Ctrl+3` | `clientes` |
| `Ctrl+4` | `obras` |
| `Ctrl+5` | `trabajos` |
| `Ctrl+6` | `certificados` |
| `Ctrl+7` | `facturas` |
| `Ctrl+8` | `empleados` |
| `Ctrl+9` | `asistencia` |

La lista se **deriva** de `MENU`, no se escribe aparte. En el sistema anterior era un array
independiente que había que mantener sincronizado a mano con el menú del XAML.

`liquidaciones` pierde su `Ctrl+9` frente a `asistencia` porque `trabajos` entra al menú. Queda
accesible por la paleta y por `Ctrl+K`.

### 4.3 Cascada de Escape

Orden estricto; el primero que aplica consume la tecla:

1. Paleta de comandos abierta → la cierra.
2. Diálogo modal abierto (confirmación, selector de archivo) → lo cancela.
3. Drawer de edición abierto → lo cierra, preguntando si hay cambios sin guardar.
4. Menú contextual o desplegable abierto → lo cierra.
5. Filtros con algún valor → los limpia. **[NUEVO]**
6. Barra lateral en modo superpuesto y abierta → la cierra.
7. Nada de lo anterior → sin efecto. **[FIX]** El sistema anterior navegaba hacia atrás como último
   paso del `Escape`, lo que hacía que la tecla te sacara de la pantalla sin que lo pidieras.

**[FIX]** La cascada anterior no cubría el drawer de Obras, los formularios en línea de Categorías y
Tipos, ni el asistente de liquidaciones. Además convivía con `KeyBinding Escape` local en seis vistas
de edición, así que el comportamiento dependía de dónde estuviera el foco. Ahora hay **una sola**
implementación, basada en una pila de capas: cada componente que se abre se apila y `Escape` cierra
la de arriba.

### 4.4 Atajos contextuales

| Contexto | Atajo | Acción |
| --- | --- | --- |
| Tabla | `↑` `↓` | mueve la fila enfocada |
| Tabla | `Enter` | edita la fila enfocada |
| Tabla | `Delete` | borra la fila enfocada, con confirmación |
| Tabla | `Ctrl+A` | selecciona todas las filas de la página |
| Tabla | `PgUp` `PgDn` | página anterior / siguiente |
| Tabla | `Home` `End` | primera / última página |
| Formulario | `Ctrl+Enter` | guarda y cierra |
| Formulario | `Ctrl+Shift+Enter` | guarda y abre uno nuevo **[NUEVO]** |
| Asistente | `Alt+←` `Alt+→` | paso anterior / siguiente |
| Asistencia | `←` `→` `↑` `↓` | mueve la celda enfocada |
| Asistencia | `Espacio` | avanza el tipo de jornada de la celda |
| Asistencia | `1`–`5` | asigna un tipo directamente **[NUEVO]** |
| Asistencia | `PgUp` `PgDn` | mes anterior / siguiente |
| Paleta | `↑` `↓` | mueve la selección |
| Paleta | `Enter` | ejecuta |

`Ctrl+Shift+Enter` en el formulario apunta a la carga en tandas: al final del día se cargan diez
movimientos seguidos y volver a abrir el formulario con el mouse cada vez es tiempo perdido.

Los números `1`–`5` en asistencia mapean a `Completa`, `Media`, `Falta`, `FaltaJustificada`,
`Feriado`, en el orden del enum. `0` o `Delete` borra el registro.

## 5. Paleta de comandos

Se abre con `Ctrl+K`. Reemplaza al buscador global.

**[FIX]** La paleta anterior ofrecía sólo 10 destinos de navegación y omitía `reportes`,
`categorias`, `tipos-movimiento` y `seed`. Su filtro comparaba contra un campo `Keywords` que
contenía literalmente el nombre de la ruta, así que buscar «factura» encontraba «facturas» por
casualidad y buscar «cobro» no encontraba nada.

### 5.1 Categorías de resultados

| Categoría | Clave | Contenido |
| --- | --- | --- |
| Navegación | `CommandPalette.Group.Navigation` | las 15 rutas, derivadas de `MENU` |
| Acciones | `CommandPalette.Group.Actions` | acciones del módulo actual y globales |
| Registros | `CommandPalette.Group.Records` | **[NUEVO]** búsqueda en clientes, obras, facturas, empleados y movimientos |
| Configuración | `CommandPalette.Group.Settings` | ir a una sección concreta de configuración |

Los resultados de **Registros** vienen de un comando de búsqueda global en el backend, con debounce
de 300 ms, límite de 5 por entidad y coincidencia por prefijo o subcadena en los campos
significativos (nombre, número, CUIT, DNI, concepto). Seleccionar un registro navega a su detalle o
abre su drawer.

### 5.2 Comportamiento

- Sin texto: muestra los destinos usados recientemente (los últimos 5, persistidos) y luego el menú
  completo.
- Con texto: coincidencia difusa sobre el título traducido, sin distinguir mayúsculas ni acentos.
  Buscar «liquid», «Liquidaciones» o «sueldos» encuentra Liquidaciones — cada ítem declara sus
  sinónimos como una lista de claves i18n, no como un string pegado.
- Los resultados se agrupan por categoría, con el atajo de cada ítem a la derecha.
- `Enter` ejecuta, `Escape` cierra, las flechas navegan y el mouse funciona.
- Placeholder: `CommandPalette.SearchPlaceholder`. Pie de ayuda: `CommandPalette.Hint`.

## 6. Migas de pan

Van en la barra de estado, alineadas a la derecha, y son **clickeables**.

**[FIX]** El indicador anterior era un texto plano `"{AppName} / {Sección}"` construido con
`string.Format` de una clave `Navigation.Breadcrumb`, sin jerarquía y sin poder clickearse. Estando
en el detalle de un trabajo de una obra no había forma de saber de qué obra se trataba.

Se construyen desde la jerarquía declarada en `route.meta.breadcrumb`, resolviendo los nombres de las
entidades que aparecen en la ruta:

```
Obras › Edificio Rivadavia 1230 › Trabajos › Tablero general › Orden 3
```

El primer nivel es siempre el ítem de menú. Cada nivel intermedio con `:id` muestra el nombre de la
entidad, que llega en la respuesta del detalle; mientras carga se muestra un placeholder. El último
nivel no es clickeable. Con más de 4 niveles, los del medio colapsan en `…` con un menú.

El nombre de la aplicación **no** va en las migas: ya está en el título de la ventana.

## 7. Comportamiento responsive

Un único breakpoint, el mismo del sistema anterior:

| Ancho | Barra lateral | Encabezado | Tablas |
| --- | --- | --- | --- |
| `< 768 px` | superpuesta, cerrada por defecto; se cierra al navegar | título y botón de menú; las acciones pasan a un menú | columnas secundarias ocultas; la fila se puede expandir |
| `>= 768 px` | en línea, compacta (56 px) o expandida (260 px) | completo | todas las columnas |

Ancho mínimo de la ventana: `320 px`. Alto mínimo: `480 px`.

Los drawers de edición ocupan el ancho completo por debajo de `768 px` y `480 px` fijos por encima.

## 8. Accesibilidad

1. Todo elemento interactivo es alcanzable con `Tab`, en un orden que sigue la lectura.
2. Todo icono sin texto tiene `aria-label` traducido.
3. El foco se ve siempre, con el anillo del tema.
4. Al abrir un drawer o un modal, el foco entra y queda atrapado adentro; al cerrar, vuelve al
   elemento que lo abrió.
5. Los toasts se anuncian por `aria-live="polite"`; los errores por `assertive`.
6. Contraste mínimo 4.5:1 para texto y 3:1 para elementos de interfaz, en ambos temas.
7. Ninguna información se transmite sólo por color: los chips de estado llevan texto y los tipos de
   jornada llevan símbolo además del fondo.

## 9. Tests obligatorios

| Test | Qué verifica |
| --- | --- |
| `menu_cubre_todas_las_rutas` | toda ruta de nivel superior del router está en `MENU`, y viceversa. Es el test que evitaría el bug de Trabajos |
| `atajos_numericos_derivan_del_menu` | `Ctrl+1..9` coinciden con los primeros nueve ítems |
| `toda_ruta_tiene_titulo_traducido` | cada `labelKey` existe en `es.json` y en `en.json` |
| `toda_ruta_con_id_valida_uuid` | un `:id` inválido redirige al listado padre |
| `escape_respeta_el_orden_de_capas` | test de componente con paleta + drawer + desplegable abiertos |
| `escape_no_navega_hacia_atras` | con nada abierto, `Escape` no cambia la ruta |
| `filtros_van_y_vuelven_de_la_url` | aplicar filtros, recargar y recuperar el mismo estado |
| `migas_reflejan_la_jerarquia` | en `/obras/:id/trabajos` hay 3 niveles con el nombre de la obra resuelto |
| `paleta_encuentra_por_sinonimo` | buscar «sueldos» devuelve Liquidaciones |
| `sidebar_colapsa_bajo_768` | el modo cambia exactamente en el breakpoint |
| `ultima_ruta_se_restaura` | y cae al dashboard si la ruta guardada ya no existe |
