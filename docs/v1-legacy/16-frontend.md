# 16 · Frontend: Vue 3, PrimeVue, Shadcn-Vue y Tailwind

Este documento define **cómo** se construye la interfaz. El **qué** de cada pantalla está en
[`09-modulos-funcionales.md`](./09-modulos-funcionales.md) y la navegación en
[`10-navegacion-y-atajos.md`](./10-navegacion-y-atajos.md).

Un implementador que lea este documento debe poder escribir cualquier pantalla del sistema sin
inventar convenciones.

---

## 1. Stack y responsabilidades

| Pieza | Versión | Para qué |
| --- | --- | --- |
| Vue 3 | `^3.5` | `<script setup>` + TypeScript, siempre |
| TypeScript | `^5.6` | `strict: true`, sin excepciones |
| Vite | `^5.4` | dev server y build |
| Vue Router | `^4.4` | rutas de doc 10 |
| Pinia | `^2.2` | estado por módulo |
| vue-i18n | `^10` | textos, doc 14 §4 |
| PrimeVue | `^4.2` | componentes de datos y formularios complejos |
| Shadcn-Vue (Reka UI) | `reka-ui ^2` | primitivas de layout, overlays y composición |
| Tailwind CSS | `^3.4` | todo el estilado |
| `tailwindcss-primeui` | `^0.3` | puente entre los tokens de PrimeVue y Tailwind |
| `@vueuse/core` | `^11` | `useDebounceFn`, `useMagicKeys`, `useLocalStorage` |
| `lucide-vue-next` | `^0.454` | iconos |

### 1.1 El reparto PrimeVue / Shadcn-Vue

Tener dos librerías de componentes es una decisión de diseño, no un accidente, y sin una regla clara
termina en dos botones distintos en la misma pantalla. La regla:

**PrimeVue** para todo lo que tiene comportamiento complejo que no vale la pena reimplementar:

| Componente PrimeVue | Uso |
| --- | --- |
| `DataTable` + `Column` | **todas** las grillas del sistema |
| `Paginator` | integrado al `DataTable` |
| `DatePicker` | toda entrada de fecha |
| `Select`, `MultiSelect`, `AutoComplete` | selección de entidades relacionadas |
| `InputNumber` | **toda** entrada de importe o cantidad (§4.2) |
| `Chart` | los gráficos del dashboard |
| `Toast` + `useToast` | notificaciones (§6.3) |
| `ConfirmDialog` + `useConfirm` | confirmaciones de borrado |
| `Tree`, `TreeSelect` | jerarquía de categorías |
| `Stepper` | wizard de liquidaciones (doc 09 §3.11) |
| `FileUpload` | adjuntos |
| `Tag` | píldoras de estado |
| `ProgressBar` | barras de porcentaje de avance |

**Shadcn-Vue** para las primitivas de estructura, donde lo que se quiere es control total del markup:

| Componente Shadcn-Vue | Uso |
| --- | --- |
| `Button` | **todos** los botones del sistema |
| `Sheet` | los drawers laterales de edición (doc 09 §1.8) |
| `Dialog` | modales que no son confirmación |
| `Command` | la paleta de comandos (doc 10 §5) |
| `Card` | tarjetas de KPI del dashboard |
| `Tabs` | pestañas de detalle |
| `Tooltip`, `DropdownMenu`, `Popover` | menús contextuales y ayudas |
| `Separator`, `ScrollArea`, `Skeleton` | estructura y estados de carga |
| `Label`, `Input`, `Textarea`, `Switch`, `Checkbox` | campos simples |
| `Badge` | contadores |
| `Breadcrumb` | migas de pan (doc 10 §6) |

Casos de solapamiento, resueltos de una vez:

| Necesidad | Gana | Motivo |
| --- | --- | --- |
| Botón | Shadcn-Vue | un solo `Button` en todo el sistema, con variantes propias |
| Diálogo de confirmación | PrimeVue `ConfirmDialog` | la API imperativa `useConfirm` es más simple que montar un `Dialog` |
| Diálogo de formulario | Shadcn-Vue `Sheet` | los formularios son drawers, no modales |
| Input de texto | Shadcn-Vue | markup simple, se estila con Tailwind |
| Input numérico | PrimeVue `InputNumber` | el formateo con separadores y el manejo del cursor no vale reimplementarlo |
| Select | PrimeVue | filtrado, carga diferida y virtualización |
| Toast | PrimeVue | ya viene la cola y el posicionamiento |

Regla operativa: **si el componente ya está usado en el sistema, se reutiliza; no se agrega un
segundo componente para lo mismo.** El test de §8 verifica que no haya imports de `Button` desde
`primevue`.

---

## 2. Estructura de carpetas

```
src/
├── main.ts                      # bootstrap: app, pinia, router, i18n, PrimeVue
├── App.vue                      # RouterView + Toast + ConfirmDialog globales
├── env.d.ts
│
├── api/                         # la única capa que llama a invoke()
│   ├── client.ts                # callCommand<T>() y normalización de ApiError
│   ├── types.ts                 # tipos espejo de Rust (doc 11 §4)
│   ├── movimientos.ts
│   ├── clientes.ts
│   ├── obras.ts
│   ├── trabajos.ts
│   ├── ordenes.ts
│   ├── certificados.ts
│   ├── facturas.ts
│   ├── pagos.ts
│   ├── empleados.ts
│   ├── asistencias.ts
│   ├── liquidaciones.ts
│   ├── categorias.ts
│   ├── tipos.ts
│   ├── dashboard.ts
│   ├── reportes.ts
│   ├── adjuntos.ts
│   ├── externos.ts              # dólar, feriados
│   ├── backup.ts
│   └── config.ts
│
├── stores/                      # un store por módulo, mismo nombre que en api/
│   ├── useMovimientosStore.ts
│   ├── … (uno por módulo)
│   ├── useConfigStore.ts        # configuración de la app, se carga en el arranque
│   ├── useUiStore.ts            # sidebar colapsada, tema, paleta abierta
│   └── useCatalogStore.ts       # catálogos compartidos: tipos, categorías, clientes
│
├── composables/
│   ├── useServerTable.ts        # paginación + filtros + orden de servidor (§5.1)
│   ├── useCrudDrawer.ts         # abrir/cerrar drawer de alta y edición (§5.2)
│   ├── useMoney.ts              # formateo y parseo de importes (§4.2)
│   ├── useDateFormat.ts         # formateo de fechas según Locale (§4.3)
│   ├── useConfirmDelete.ts      # confirmación de borrado uniforme (§5.3)
│   ├── useApiError.ts           # traducción de ApiError a mensajes (§6.2)
│   ├── useShortcuts.ts          # registro de atajos (§5.4)
│   ├── useBreadcrumb.ts         # migas de pan derivadas de la ruta
│   └── useExport.ts             # disparar una exportación con diálogo de guardado
│
├── components/
│   ├── ui/                      # Shadcn-Vue generado; NO se edita a mano salvo tokens
│   │   ├── button/ sheet/ dialog/ card/ tabs/ command/ …
│   ├── layout/
│   │   ├── AppShell.vue         # grid principal
│   │   ├── AppSidebar.vue       # navegación de doc 10 §3
│   │   ├── AppHeader.vue        # migas + acciones globales + buscador
│   │   ├── AppStatusBar.vue     # estado de conexión, cotización, versión
│   │   └── CommandPalette.vue   # doc 10 §5
│   └── domain/                  # componentes reutilizables con semántica de negocio
│       ├── MoneyText.vue        # muestra un Money con signo y color
│       ├── MoneyInput.vue       # wrapper de InputNumber (§4.2)
│       ├── DateText.vue
│       ├── DateInput.vue
│       ├── PercentBar.vue
│       ├── StatePill.vue        # Tag con el color del estado (§4.4)
│       ├── EntitySelect.vue     # Select con carga diferida de una entidad
│       ├── ClienteSelect.vue    # EntitySelect preconfigurado
│       ├── ObraSelect.vue
│       ├── TrabajoSelect.vue
│       ├── EmpleadoSelect.vue
│       ├── PageHeader.vue       # título + subtítulo + slot de acciones
│       ├── FilterBar.vue        # contenedor de filtros con "limpiar"
│       ├── DataGrid.vue         # wrapper de DataTable con la config común (§5.1)
│       ├── ListState.vue        # cargando / vacío / error (§6.1)
│       ├── CrudDrawer.vue       # Sheet + header + footer con Guardar/Cancelar
│       ├── FieldError.vue       # error de validación de un campo
│       ├── AttachmentList.vue
│       └── ExportMenu.vue       # DropdownMenu con los formatos disponibles
│
├── views/                       # una carpeta por ruta de doc 10 §2
│   ├── dashboard/DashboardView.vue
│   ├── movimientos/{MovimientosView.vue, MovimientoForm.vue}
│   ├── clientes/{ClientesView.vue, ClienteForm.vue, ClienteDetalleView.vue}
│   ├── obras/{ObrasView.vue, ObraForm.vue, ObraDetalleView.vue}
│   ├── trabajos/{TrabajoForm.vue, TrabajoDetalleView.vue}
│   ├── ordenes/{OrdenForm.vue, OrdenDetalleView.vue}
│   ├── certificados/{CertificadosView.vue, CertificadoForm.vue}
│   ├── facturas/{FacturasView.vue, FacturaForm.vue, FacturaDetalleView.vue}
│   ├── empleados/{EmpleadosView.vue, EmpleadoForm.vue, EmpleadoDetalleView.vue}
│   ├── asistencia/AsistenciaView.vue
│   ├── liquidaciones/{LiquidacionesView.vue, LiquidacionWizard.vue}
│   ├── comercial/{CuentaCorrienteView.vue, AntiguedadDeudaView.vue}
│   ├── reportes/ReportesView.vue
│   ├── categorias/CategoriasView.vue
│   ├── tipos-movimiento/TiposMovimientoView.vue
│   ├── configuracion/ConfiguracionView.vue
│   └── errors/{NotFoundView.vue, ErrorView.vue}
│
├── router/{index.ts, routes.ts, guards.ts}
├── locales/{es.json, en.json}
└── assets/{main.css, tokens.css}
```

Regla de dependencias, en un solo sentido:

```
views  →  components/domain  →  components/ui
  ↓            ↓
stores  →  composables  →  api
```

- Un componente de `ui/` **no** importa nada de `stores/` ni de `api/`.
- Un componente de `domain/` puede usar `composables/` y `useCatalogStore`, nada más.
- Una `view` no llama a `api/` directamente: pasa por su store.
- `api/` no importa nada de `stores/`.

El test de §8 verifica estas reglas leyendo los imports.

---

## 3. Tailwind y tokens

### 3.1 Prohibición de colores literales

**Ningún** valor de color aparece en un `.vue`. Ni hex, ni `rgb()`, ni una clase como
`bg-blue-500`. Sólo clases que refieran a un token semántico.

```vue
<!-- MAL -->
<div class="bg-[#1e293b] text-gray-400">
<div class="bg-slate-800 text-slate-400">

<!-- BIEN -->
<div class="bg-surface-card text-muted-foreground">
```

El motivo es concreto: el sistema tiene tema claro y oscuro, y un color literal se ve bien en uno y
mal en el otro. Además el sistema anterior acumuló decenas de `#252525` y `#aaa` sueltos que hicieron
imposible cambiar la paleta.

### 3.2 Los tokens

Se definen como variables CSS en `assets/tokens.css`, con un bloque por tema, y se exponen a Tailwind
en `tailwind.config.ts`.

```css
/* assets/tokens.css */
@layer base {
  :root {
    --background:            0 0% 100%;
    --foreground:            222 47% 11%;
    --muted:                 210 40% 96%;
    --muted-foreground:      215 16% 47%;
    --surface-card:          0 0% 100%;
    --surface-raised:        210 40% 98%;
    --border:                214 32% 91%;
    --input:                 214 32% 91%;
    --ring:                  222 47% 11%;
    --primary:               222 47% 11%;
    --primary-foreground:    210 40% 98%;
    --secondary:             210 40% 96%;
    --secondary-foreground:  222 47% 11%;
    --destructive:           0 72% 51%;
    --destructive-foreground:210 40% 98%;

    /* semánticos de negocio */
    --money-positive:        142 71% 45%;   /* ingreso */
    --money-negative:        0 72% 51%;     /* gasto */
    --money-neutral:         215 16% 47%;   /* cero */

    /* estados; un token por variante de cada enum */
    --state-draft:           215 16% 47%;
    --state-issued:          217 91% 60%;
    --state-paid:            142 71% 45%;
    --state-partial:         38 92% 50%;
    --state-overdue:         0 72% 51%;
    --state-void:            215 16% 65%;
    --state-active:          142 71% 45%;
    --state-paused:          38 92% 50%;
    --state-finished:        217 91% 60%;
    --state-cancelled:       215 16% 65%;

    --radius: 0.5rem;
  }

  .dark {
    --background:            222 47% 8%;
    --foreground:            210 40% 98%;
    /* … el mismo conjunto completo, sin faltar ninguno … */
  }
}
```

Regla: **todo token definido en `:root` tiene que estar definido en `.dark`.** El test
`tokens_completos_en_ambos_temas` de §8 lo verifica parseando el CSS.

Los tokens de estado (`--state-*`) existen uno por variante de enum, y `StatePill.vue` los mapea. El
color de un estado nunca se decide en la vista.

### 3.3 Espaciado y tipografía

Sin tokens propios: se usa la escala de Tailwind. Convenciones fijas para que las pantallas se vean
iguales:

| Contexto | Clases |
| --- | --- |
| Padding de página | `p-6` |
| Separación entre secciones | `space-y-6` |
| Separación entre campos de un formulario | `space-y-4` |
| Padding de tarjeta | `p-4` |
| Título de página | `text-2xl font-semibold tracking-tight` |
| Subtítulo | `text-sm text-muted-foreground` |
| Etiqueta de campo | `text-sm font-medium` |
| Texto de tabla | `text-sm` |
| Importe en tabla | `text-sm font-medium tabular-nums text-right` |

`tabular-nums` en toda columna numérica es obligatorio: sin eso los importes de una columna no se
alinean y la tabla se vuelve difícil de leer.

### 3.4 Integración con PrimeVue

PrimeVue 4 usa su propio sistema de temas (`@primeuix/themes`). Se configura con un preset que
**lee los mismos tokens CSS**, para que un botón de PrimeVue y uno de Shadcn no tengan colores
distintos.

```ts
// main.ts
import PrimeVue from 'primevue/config'
import { definePreset } from '@primeuix/themes'
import Aura from '@primeuix/themes/aura'

const preset = definePreset(Aura, {
  semantic: {
    primary: {
      500: 'hsl(var(--primary))',
      600: 'hsl(var(--primary))',
    },
    colorScheme: {
      light: { surface: { 0: 'hsl(var(--background))', 100: 'hsl(var(--surface-raised))' } },
      dark:  { surface: { 0: 'hsl(var(--background))', 100: 'hsl(var(--surface-raised))' } },
    },
  },
})

app.use(PrimeVue, {
  theme: { preset, options: { darkModeSelector: '.dark', cssLayer: { name: 'primevue', order: 'base, primevue, utilities' } } },
  ripple: false,
})
```

`cssLayer` con ese orden es necesario para que una utilidad de Tailwind pueda sobreescribir un estilo
de PrimeVue. Sin eso hay que usar `!important` y el estilado se vuelve una pelea.

`ripple: false` porque es una aplicación de escritorio, no una app táctil.

---

## 4. Presentación de datos de negocio

### 4.1 La regla central

El backend manda **datos**, no texto formateado. Un `Money` llega como el string de su valor
decimal (`"12345.6700"`), una fecha como ISO-8601 UTC. El frontend formatea. Ver
[`11-contratos-tauri.md`](./11-contratos-tauri.md) §4.

Corolario: **ningún componente hace aritmética con importes.** Si una pantalla necesita un total, el
backend lo manda. El motivo es que sumar en JavaScript con `Number` pierde precisión en centavos, y
si el total de la pantalla no coincide con el del PDF el usuario deja de confiar en el sistema.

Excepción única y explícita: `MovimientosView` muestra el total de la página, y ese total **viene en
la respuesta paginada** (`PagedResult.summary`), no se calcula sumando las filas.

### 4.2 Importes

Un `Money` cruza el IPC como el string de su valor decimal con 4 decimales, exactamente
`"12345.6700"` (doc 04 §1.6). Se elige string y no número porque un `i64` escalado supera el entero
seguro de JavaScript en importes grandes.

```ts
// composables/useMoney.ts
export function useMoney() {
  const { locale } = useConfigStore()

  /** "12345.6700" → "$ 12.345,67" */
  function format(raw: string, opts?: { showSign?: boolean; hideSymbol?: boolean }): string { … }

  /** "12345.6700" → 12345.67  — SOLO para pasarle el valor a InputNumber */
  function toInputValue(raw: string): number { … }

  /** 12345.67 → "12345.6700"  — SOLO para enviar al backend */
  function fromInputValue(n: number): string { … }

  return { format, toInputValue, parse: fromInputValue }
}
```

`toInputValue` / `fromInputValue` son el **único** lugar del frontend donde un importe es un `number`.
Existen porque `InputNumber` de PrimeVue trabaja con números. Fuera de ese borde, un importe es
siempre un string.

`MoneyInput.vue` encapsula el par:

```vue
<script setup lang="ts">
const model = defineModel<string>({ required: true })   // el string de 4 decimales
const { toInputValue, parse } = useMoney()
const { locale } = useConfigStore()

const inner = computed({
  get: () => toInputValue(model.value),
  set: (n: number | null) => { model.value = parse(n ?? 0) },
})
</script>

<template>
  <InputNumber
    v-model="inner"
    mode="currency"
    :currency="locale.monedaPorDefecto"
    :locale="locale.intlTag"
    :min-fraction-digits="locale.decimalesMoneda"
    :max-fraction-digits="locale.decimalesMoneda"
  />
</template>
```

`MoneyText.vue` para mostrar: aplica el token de color según el signo y agrega `tabular-nums`.

### 4.3 Fechas

| Situación | Componente | Formato |
| --- | --- | --- |
| Mostrar una fecha civil | `DateText` | `Locale.FormatoFecha` |
| Mostrar un instante | `DateText` con `show-time` | `Locale.FormatoFechaHora`, convertido a la zona local |
| Ingresar una fecha civil | `DateInput` | envía `YYYY-MM-DD`, sin hora |
| Ingresar un instante | `DateInput` con `show-time` | envía ISO-8601 con offset |

`DateInput` envuelve `DatePicker` de PrimeVue y hace la conversión en sus dos bordes. Una vista nunca
manipula un `Date` de JavaScript directamente.

**Nunca** se pone un formato de fecha en un archivo de locale (doc 14 §4). El formato es
configuración, no traducción; están separados porque el usuario puede querer la interfaz en inglés
con fechas en formato argentino.

### 4.4 Estados

```vue
<!-- StatePill.vue -->
<script setup lang="ts">
const props = defineProps<{ entity: 'factura' | 'obra' | 'trabajo'; value: number }>()
const { t } = useI18n()

// El mapa vive acá y en ningún otro lado.
const label = computed(() => t(`State.${capitalize(props.entity)}.${VARIANTS[props.entity][props.value]}`))
const token = computed(() => STATE_TOKEN[props.entity][props.value])
</script>

<template>
  <Tag :value="label" :style="{ backgroundColor: `hsl(var(--state-${token}))` }" />
</template>
```

**[FIX]** El sistema anterior tenía un converter con las etiquetas de estado **en español, en el
código** (`EstadoTrabajoDisplayConverter`). Acá la etiqueta sale de i18n y el color de un token. Un
estado nuevo se agrega en tres lugares y el test lo verifica: el enum de Rust, la clave i18n y el
token de color.

### 4.5 Porcentajes

`PercentBar.vue` recibe el string decimal del `Decimal4` y muestra una `ProgressBar` con el número
al lado. Un porcentaje mayor a 100 se muestra en el color `--state-overdue`, sin recortar la barra:
el dato erróneo tiene que ser visible, no escondido (los datos importados pueden tenerlo, doc 15
§4.9).

---

## 5. Composables

### 5.1 `useServerTable`

El composable más importante: implementa el patrón de lista de doc 09 §1. Toda vista de listado lo
usa; no hay dos implementaciones de paginación en el sistema.

```ts
export interface ServerTableOptions<TFilter, TRow> {
  /** Clave para persistir tamaño de página y orden. Ej: 'movimientos' */
  key: string
  /** Filtros iniciales. */
  initialFilter: TFilter
  /** La función del store que trae la página. */
  fetch: (query: PagedQuery<TFilter>) => Promise<PagedResult<TRow>>
  /** Campo de orden por defecto. */
  defaultSort?: { field: string; order: 1 | -1 }
}

export function useServerTable<TFilter extends object, TRow>(opts: ServerTableOptions<TFilter, TRow>) {
  return {
    rows,          // Ref<TRow[]>
    total,         // Ref<number>
    summary,       // Ref<unknown | null>  — agregados que manda el backend
    loading,       // Ref<boolean>
    error,         // Ref<ApiError | null>
    isEmpty,       // ComputedRef<boolean>  — !loading && !error && rows.length === 0
    filter,        // Ref<TFilter>          — mutarlo dispara reload con debounce
    page,          // Ref<number>           — 1-based
    pageSize,      // Ref<number>
    sort,          // Ref<{ field, order } | null>
    reload,        // () => Promise<void>   — inmediato, sin debounce
    resetFilter,   // () => void
    onPage,        // handler para el evento del DataTable
    onSort,        // handler para el evento del DataTable
  }
}
```

Comportamiento obligatorio:

| Aspecto | Regla |
| --- | --- |
| Debounce | 300 ms sobre cambios de `filter`. Los cambios de `page`, `pageSize` y `sort` son inmediatos. |
| Cancelación | una petición en vuelo se descarta si llegó otra después. Se compara un contador de secuencia; la respuesta con secuencia menor a la última se ignora. |
| Reset de página | cambiar un filtro o `pageSize` vuelve a `page = 1`. Cambiar el orden **no**. |
| Tamaños | `10 / 30 / 50 / 100 / 0`, donde `0` es "todos". Default `30`. |
| Persistencia | `pageSize` y `sort` se guardan en `localStorage` bajo `eo.table.{key}`. Los **filtros no** se persisten: al volver a una pantalla se ve todo. |
| URL | `page`, `pageSize` y los filtros no triviales se reflejan en el query string, para que una pantalla filtrada se pueda compartir o recargar. |
| Error | un fallo deja `rows` intacto y llena `error`. No se vacía la tabla: perder lo que ya se veía por un error de red es peor que mostrarlo desactualizado. |

Sobre la cancelación: sin ella, escribir rápido en un filtro produce respuestas que llegan
desordenadas y la tabla muestra el resultado de una búsqueda anterior. Es un bug difícil de
reproducir y fácil de prevenir.

Uso típico:

```vue
<script setup lang="ts">
const store = useMovimientosStore()
const table = useServerTable({
  key: 'movimientos',
  initialFilter: { texto: '', desde: null, hasta: null, tipoMovimientoId: null },
  fetch: (q) => store.fetchPaged(q),
  defaultSort: { field: 'fecha', order: -1 },
})
</script>

<template>
  <PageHeader :title="$t('Movimientos.Title')">
    <template #actions>
      <Button @click="drawer.openCreate()">{{ $t('Common.New') }}</Button>
    </template>
  </PageHeader>

  <FilterBar @clear="table.resetFilter">
    <Input v-model="table.filter.value.texto" :placeholder="$t('Movimientos.Filter.Text')" />
    <DateInput v-model="table.filter.value.desde" :label="$t('Common.From')" />
    <DateInput v-model="table.filter.value.hasta" :label="$t('Common.To')" />
  </FilterBar>

  <DataGrid :table="table">
    <Column field="fecha" :header="$t('Movimientos.Field.Fecha')" sortable>
      <template #body="{ data }"><DateText :value="data.fecha" /></template>
    </Column>
    <Column field="monto" :header="$t('Movimientos.Field.Monto')" sortable>
      <template #body="{ data }"><MoneyText :value="data.total" show-sign /></template>
    </Column>
  </DataGrid>
</template>
```

`DataGrid.vue` recibe el objeto de `useServerTable` completo y conecta `rows`, `total`, `loading`,
`onPage`, `onSort` y `ListState`. Una vista **no** cablea eventos del `DataTable` a mano.

### 5.2 `useCrudDrawer`

Todo alta y edición es un drawer lateral (`Sheet`), no una página ni un modal. La razón: se ve la
lista atrás y no se pierde el contexto de filtrado.

```ts
export function useCrudDrawer<TDto, TId = string>(opts: {
  empty: () => TDto
  load: (id: TId) => Promise<TDto>
  create: (dto: TDto) => Promise<unknown>
  update: (id: TId, dto: TDto) => Promise<unknown>
  onSaved?: () => void
}) {
  return {
    open,          // Ref<boolean>
    mode,          // Ref<'create' | 'edit'>
    model,         // Ref<TDto>
    saving,        // Ref<boolean>
    fieldErrors,   // Ref<Record<string, string>>  — de ApiError.field_errors
    openCreate,    // () => void
    openEdit,      // (id: TId) => Promise<void>
    save,          // () => Promise<void>
    close,         // () => void   — pide confirmación si hay cambios sin guardar
    isDirty,       // ComputedRef<boolean>
  }
}
```

Reglas:

- Al guardar bien: cierra el drawer, muestra un toast de éxito y llama `onSaved` (que hace
  `table.reload()`).
- Al recibir un error de validación (`ApiError.kind === 'Validation'`): **no** cierra, llena
  `fieldErrors` y hace foco en el primer campo con error.
- Al recibir un conflicto de concurrencia (`kind === 'Conflict'`): no cierra, muestra un mensaje que
  ofrece recargar el registro y descartar los cambios.
- `close` con `isDirty` pide confirmación. Un `Escape` accidental no puede perder un formulario
  cargado a medias.

### 5.3 `useConfirmDelete`

```ts
const { confirmDelete } = useConfirmDelete()

await confirmDelete({
  // El nombre de la entidad para el mensaje; siempre una clave i18n.
  entityKey: 'Entity.Movimiento',
  label: row.concepto,
  action: () => store.remove(row.id),
  onDone: () => table.reload(),
})
```

Un solo mensaje de confirmación, parametrizado, para todo el sistema. El sistema anterior tenía
textos de confirmación distintos por pantalla, algunos sin traducir.

### 5.4 `useShortcuts`

Registra los atajos de doc 10 §4 sobre `useMagicKeys`.

```ts
useShortcuts({
  'ctrl+n': () => drawer.openCreate(),
  'ctrl+s': { handler: () => drawer.save(), when: () => drawer.open.value },
  'f5':     () => table.reload(),
})
```

Reglas:

- Un atajo **no** se dispara si el foco está en un campo de texto, salvo que se declare
  `allowInInput: true`. `Ctrl+S` dentro de un formulario es la excepción esperada.
- Los atajos se desregistran al desmontar el componente. El composable lo hace solo con
  `onScopeDispose`.
- `Escape` no se maneja acá: es cascada y vive en `useEscapeStack`, porque el orden importa (cierra
  primero la paleta, después el drawer, después el filtro).

### 5.5 `useApiError`

Traduce un `ApiError` del backend a un mensaje mostrable. Ver §6.2.

---

## 6. Estados y errores

### 6.1 Los cuatro estados de una lista

Toda lista tiene exactamente cuatro estados, y `ListState.vue` los cubre. No se permite una lista sin
`ListState`.

| Estado | Qué se muestra |
| --- | --- |
| Cargando (primera vez) | `Skeleton` con la forma de la tabla, no un spinner |
| Cargando (recarga) | la tabla anterior con opacidad reducida y una barra de progreso arriba |
| Vacío | icono, mensaje de vacío específico del módulo, y el botón de alta |
| Error | icono, el mensaje traducido del `ApiError`, y un botón de reintentar |

La distinción entre las dos clases de carga es intencional: reemplazar una tabla con datos por un
spinner en cada tecla del filtro produce un parpadeo desagradable.

El estado vacío distingue dos casos, con mensajes distintos:

- **Sin datos**: no hay registros en absoluto → invita a crear el primero.
- **Sin resultados**: hay registros pero los filtros no dan nada → ofrece limpiar los filtros.

### 6.2 Manejo de `ApiError`

Todo error del backend llega como el `ApiError` de doc 11 §2. `useApiError` decide qué hacer según
`kind`:

| `kind` | Tratamiento |
| --- | --- |
| `Validation` | **no** hay toast. Los errores se pintan en los campos del formulario. |
| `NotFound` | toast de advertencia y recarga de la lista: el registro lo borró otro. |
| `Conflict` | mensaje en el drawer con la opción de recargar (§5.2). |
| `BusinessRule` | toast de advertencia con el mensaje traducido de `code`. |
| `Database` | toast de error genérico. `detail` va sólo al log. |
| `External` | toast informativo, no de error: el sistema sigue funcionando sin la API externa. |
| `Io` | toast de error con la operación que falló. |
| `Unexpected` | toast de error genérico con el `trace_id` visible y copiable. |

Regla sobre los mensajes: el `code` del `ApiError` es una clave i18n. El frontend hace
`t(error.code, error.params)`. Si la clave no existe, cae a un mensaje genérico y **loguea la clave
faltante**, para que aparezca en el test de §8 y no en producción.

`detail` del `ApiError` **nunca** se muestra al usuario: puede contener una ruta de archivo o un
fragmento de SQL. Va al log del frontend junto con el `trace_id`.

### 6.3 Toasts

Un solo `<Toast>` en `App.vue`. Convenciones:

| Severidad | Duración | Uso |
| --- | --- | --- |
| `success` | 3 s | operación completada |
| `info` | 4 s | degradación de un servicio externo |
| `warn` | 5 s | regla de negocio, registro no encontrado |
| `error` | sin autocierre | error inesperado, error de base |

Un error que el usuario tiene que leer no se cierra solo. Un éxito no puede tapar la pantalla.

### 6.4 Barrera de errores

`App.vue` registra `app.config.errorHandler` y `onErrorCaptured` en un componente barrera. Un error
de render de una vista muestra `ErrorView` con el botón de recargar, no una pantalla en blanco.

---

## 7. Stores Pinia

### 7.1 Anatomía

Todos los stores usan la forma de setup y siguen el mismo esqueleto:

```ts
export const useMovimientosStore = defineStore('movimientos', () => {
  const api = useMovimientosApi()

  // Estado: sólo lo que se comparte entre componentes.
  const current = ref<MovimientoDto | null>(null)

  // Acciones: envuelven la capa api y no atrapan errores.
  async function fetchPaged(query: PagedQuery<MovimientoFilter>) {
    return api.getPaged(query)
  }

  async function create(dto: MovimientoCreateDto) { return api.create(dto) }
  async function update(id: string, dto: MovimientoUpdateDto) { return api.update(id, dto) }
  async function remove(id: string) { return api.remove(id) }

  return { current, fetchPaged, create, update, remove }
})
```

Reglas:

1. **Un store no cachea listas paginadas.** La página vive en el componente, en `useServerTable`. Un
   store con la lista adentro obliga a invalidarla y se desincroniza.
2. **Un store no atrapa errores.** Deja propagar; el manejo está en el componente y en el
   interceptor.
3. **Un store no formatea.** No hay `montoFormateado` en un store.
4. Lo que **sí** va en un store: la entidad actualmente seleccionada, catálogos compartidos, y estado
   de interfaz que sobrevive a la navegación.

### 7.2 `useCatalogStore`

Los catálogos que casi no cambian y que muchas pantallas necesitan: tipos de movimiento, tipos de
concepto de pago, categorías, y la lista corta de clientes, obras y empleados activos para los
selects.

- Se cargan de manera diferida, la primera vez que alguien los pide.
- Se invalidan cuando una operación los modifica: crear una categoría invalida el catálogo de
  categorías.
- **No** tienen expiración por tiempo. Es una aplicación de escritorio con un solo usuario; nadie más
  cambia los datos por detrás.

### 7.3 `useConfigStore`

Se carga **antes** de montar la aplicación, en `main.ts`, porque el formateo de importes y fechas
depende de él y no puede haber un primer render con los valores equivocados.

```ts
const app = createApp(App)
app.use(createPinia())

const config = useConfigStore()
await config.load()                    // bloqueante, antes del mount
i18n.global.locale.value = config.locale.language

app.use(router).use(i18n).use(PrimeVue, primeOptions)
app.mount('#app')
```

### 7.4 `useUiStore`

Sidebar colapsada, tema (`light` / `dark` / `system`), paleta de comandos abierta, última ruta
visitada. Se persiste en `localStorage`. No toca el backend.

---

## 8. Reglas verificables

Estas reglas son tests, no recomendaciones. Viven en `src/__tests__/architecture.spec.ts` y se
implementan leyendo los archivos del proyecto.

| Test | Qué verifica |
| --- | --- |
| `sin_colores_literales` | ningún `.vue` ni `.css` fuera de `tokens.css` contiene `#`, `rgb(`, `hsl(` ni una clase de color de Tailwind (`bg-red-500`, `text-slate-400`…) |
| `sin_literales_de_texto` | ningún template tiene texto visible fuera de `$t()`; la excepción es contenido numérico y los `data-*` |
| `tokens_completos_en_ambos_temas` | todo token de `:root` está en `.dark` |
| `un_solo_boton` | ningún archivo importa `Button` desde `primevue` |
| `vistas_no_llaman_invoke` | `invoke` sólo aparece en `api/client.ts` |
| `vistas_no_importan_api` | ninguna vista importa de `src/api/` |
| `api_no_importa_stores` | ningún archivo de `api/` importa de `stores/` |
| `ui_no_importa_stores` | ningún componente de `components/ui/` importa de `stores/` ni `api/` |
| `toda_lista_usa_useServerTable` | toda vista con un `DataTable` usa `useServerTable` |
| `toda_lista_tiene_ListState` | toda vista con un `DataTable` renderiza `ListState` |
| `sin_aritmetica_de_importes` | ningún `.vue` contiene una operación aritmética sobre un campo cuyo nombre sugiere importe (`monto`, `total`, `subtotal`, `precio`, `saldo`) |
| `sin_formato_de_fecha_hardcodeado` | ningún archivo contiene `dd/MM`, `MM/dd` ni `toLocaleDateString` con opciones literales |
| `claves_i18n_existen` | toda clave usada en `$t()` existe en los dos locales |
| `tabular_nums_en_columnas_numericas` | toda `Column` con un `MoneyText` o `PercentBar` tiene la clase |

Los cuatro primeros son los que impiden que la interfaz se degrade con el tiempo. El sistema anterior
no los tenía y terminó con colores sueltos y textos en español dentro del código.

---

## 9. Accesibilidad y usabilidad

No es una lista de deseos: son requisitos verificables de la interfaz.

| Requisito | Cómo se cumple |
| --- | --- |
| Todo se puede operar con teclado | Shadcn-Vue y PrimeVue ya manejan el foco; se verifica pantalla por pantalla |
| Todo campo tiene `Label` asociada | `Label` con `for` apuntando al `id` del input |
| Un error de validación se anuncia | `aria-invalid` y `aria-describedby` apuntando al `FieldError` |
| El foco vuelve donde estaba al cerrar un drawer | `Sheet` de Shadcn-Vue lo hace; no se desactiva |
| El primer campo del drawer recibe el foco al abrir | `autofocus` en el formulario |
| Contraste mínimo AA | los tokens se eligen cumpliéndolo; se verifica una vez por tema |
| Ninguna acción depende sólo del color | los estados tienen etiqueta además de color |
| La tabla no se reordena sola | el orden sólo cambia por acción del usuario |

Sobre el último punto: una lista que se reordena mientras el usuario lee es una fuente constante de
clicks en la fila equivocada. Los datos se refrescan, el orden no cambia solo.

---

## 10. Qué NO se hace en el frontend

Lista explícita, para cortar la tentación:

| No se hace | Dónde va |
| --- | --- |
| Calcular un total, un saldo, un margen o un porcentaje | backend, doc 06 |
| Decidir si una transición de estado es válida | backend, doc 08 |
| Validar una regla de negocio | backend, doc 07. El frontend valida sólo formato de entrada. |
| Generar un PDF o un XLSX | backend, doc 12 |
| Leer o escribir un archivo | backend, doc 13 |
| Llamar a una API externa | backend, doc 13 §2 y §3 |
| Guardar un secreto o una credencial | no existen en este sistema |
| Consultar la base directamente | no hay acceso a la base desde el frontend |

El frontend valida formato de entrada (un campo obligatorio está vacío, un email no tiene forma de
email) para dar respuesta inmediata, pero **el backend valida todo de nuevo**. La validación del
frontend es cortesía; la del backend es la que cuenta.
