# Especificación Técnica: Módulo de Calendario, UX y Onboarding

## 1. Diagnóstico Actual

1. **Bug Crítico de Huso Horario en Calendario:**
   - En [CalendarioView.vue:47-49](../../src/views/calendario/CalendarioView.vue#L47-L49):
     ```typescript
     function formatearFechaHoraIso(d: Date): string {
       return `${formatearFechaIso(d)}T${pad(d.getHours())}:${pad(d.getMinutes())}:00.000Z`
     }
     ```
   - La función toma las horas locales (`d.getHours()`) y les concatena directamente el sufijo `Z` (Zulu/UTC). Esto produce que una fecha local de las 09:00 en Argentina (UTC-3) se guarde como `09:00:00.000Z`. Cuando el motor la recupera y la vuelve a convertir a hora local para mostrarla en la cuadrícula, la traslada a las 06:00 hs (desfasaje de 3 horas).
2. **Inconsistencia Visual Severa en Calendario:**
   - A diferencia del resto de la aplicación que utiliza componentes estandarizados de PrimeVue, [CalendarioView.vue](../../src/views/calendario/CalendarioView.vue) utiliza controles HTML crudos (`<input>`, `<select>`, `<button>`), no usa `PageHeader`, no incluye `HelpButton`, y construye un modal flotante no estándar mediante `fixed inset-0`.
3. **Eventos Desconectados de Proyectos/Trabajos:**
   - A pesar de que el modelo backend `CalendarioEventoDto` soporta `trabajoId` y `kanbanTarjetaId`, el formulario del evento no permite seleccionar ningún trabajo u obra relacionada.
4. **Ausencia de Bienvenida para Nuevos Usuarios (Onboarding):**
   - En [App.vue:48-58](../../src/App.vue#L48-L58), la pantalla [WelcomeView.vue](../../src/views/WelcomeView.vue) sólo se muestra si se detecta una base de datos del sistema anterior (C# legacy). Un usuario que instala la aplicación por primera vez es depositado sin aviso en un Dashboard repleto de ceros y tarjetas vacías, sin indicación de qué paso dar primero.

---

## 2. Solución Propuesta

### 2.1. Corrección del Formateo de Fechas ISO
Reemplazar la manipulación manual de cadenas por el estándar ISO UTC o representación civil:
```typescript
function formatearFechaHoraIso(d: Date): string {
  // ISO 8601 UTC estricto generado por el objeto Date nativo
  return d.toISOString()
}
```
Para la entrada de los inputs del formulario (tipo `datetime-local` o `DatePicker`):
```typescript
function fechaLocalParaInput(isoStr: string): string {
  const d = new Date(isoStr)
  const pad = (n: number) => n.toString().padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`
}
```

### 2.2. Estandarización de Interfaz con PrimeVue
Refactorizar [CalendarioView.vue](../../src/views/calendario/CalendarioView.vue):
- Reemplazar el encabezado plano por:
  ```html
  <PageHeader :title="$t('Menu.Calendario')" :subtitle="$t('Calendario.Subtitle')">
    <template #actions>
      <Button @click="abrirCrearEvento()">
        <AppIcon name="plus" :size="16" />
        {{ $t('Calendario.NuevoEvento') }}
      </Button>
      <HelpButton topic-id="calendario-overview" title="Ayuda sobre la Agenda Operativa" />
    </template>
  </PageHeader>
  ```
- Reemplazar el modal flotante por un componente `Dialog` de PrimeVue:
  - Campos con `InputText`, `Select` y `Textarea`.
  - Integrar selector de **Proyecto / Trabajo** opcional en el formulario de creación de eventos.

### 2.3. Onboarding Amigable para Nuevos Usuarios
En [App.vue](../../src/App.vue):
- Si `!localStorage.getItem('eo:welcomed')`, verificar si hay datos legacy.
- Si no hay datos legacy pero la base está limpia (0 clientes, 0 movimientos), abrir [WelcomeView.vue](../../src/views/WelcomeView.vue) mostrando un mensaje de bienvenida amigable:
  1. *"Bienvenido a Certaro: Gestión para Pymes por Proyecto"*.
  2. Ofrecer un breve asistente para configurar:
     - Nombre de la Empresa / Razón Social.
     - CUIT y Condición de IVA.
     - Saldo inicial de caja chica / banco.
  3. Ofrecer la opción de:
     - *"Comenzar desde cero"*
     - *"Cargar datos de ejemplo (Demostración)"* (si el entorno de desarrollo tiene seed habilitado).
- Al guardar o descartar, marcar `localStorage.setItem('eo:welcomed', 'true')`.

---

## 3. Modificaciones de Archivos y Componentes

### Frontend
- **[src/views/calendario/CalendarioView.vue](../../src/views/calendario/CalendarioView.vue):**
  - Corrección de la función `formatearFechaHoraIso`.
  - Reemplazo del modal y los inputs nativos por componentes PrimeVue (`Dialog`, `InputText`, `Select`).
  - Incorporación de `PageHeader` y `HelpButton`.
  - Adición de selector de Trabajo en la creación de eventos.
- **[src/App.vue](../../src/App.vue):**
  - Ajuste de la condición de primera apertura para dar la bienvenida a nuevos usuarios.
- **[src/views/WelcomeView.vue](../../src/views/WelcomeView.vue):**
  - Incorporación de paso de configuración inicial rápida para usuarios nuevos sin base previa.
- **[src/locales/es.json](../../src/locales/es.json):**
  - Claves para el asistente de configuración inicial y eventos de calendario.
