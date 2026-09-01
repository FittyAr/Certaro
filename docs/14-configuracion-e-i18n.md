# 14 — Configuración e i18n

> Define `crates/eo-infrastructure/src/config/` y `src/locales/`. Es la fuente de verdad de **toda**
> clave de configuración y **toda** clave de traducción. Si un valor no está acá, está hardcodeado, y
> eso es un error.

## 1. Modelo de configuración

### 1.1 Las tres capas

```
1. Defaults compilados      → crates/eo-infrastructure/config/defaults.toml (incluido con include_str!)
2. Archivo del usuario      → {data_dir}/config.json          (mutable, lo escribe la app)
3. Variables de entorno     → EO_<SECCION>__<CLAVE>            (sólo para desarrollo y CI)
```

Precedencia: **3 sobre 2 sobre 1**. La capa 1 siempre existe y está completa, así que la aplicación
arranca sin ningún archivo.

**[FIX]** El sistema anterior tenía dos mecanismos que se pisaban: `IConfiguration` leía un
`appsettings.json` copiado al directorio de datos más un overlay por entorno, y en paralelo
`UserSettingsService` leía y escribía **el mismo archivo** por su cuenta, sin pasar por
`IConfiguration`. Resultado: un valor cambiado desde la pantalla de configuración no se veía en el
código que usaba `IConfiguration` hasta reiniciar, y los dos tenían **defaults distintos para la
misma clave** — `Application:Name` valía `"ElectroObraApp"` en un lado y `"ElectroObra"` en el otro,
y `Application:Branding:LogoPath` apuntaba a un `.svg` en uno y a un `.png` en el otro.

### 1.2 Tipado

La configuración es un **struct**, no un diccionario de strings.

```rust
// crates/eo-infrastructure/src/config/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    pub application: ApplicationConfig,
    pub locale: LocaleConfig,
    pub business: BusinessConfig,
    pub settlement: SettlementConfig,
    pub dashboard: DashboardConfig,
    pub external_apis: ExternalApisConfig,
    pub attachments: AttachmentsConfig,
    pub backup: BackupConfig,
    pub communication: CommunicationConfig,
    pub logging: LoggingConfig,
    pub validation: ValidationConfig,
    pub report: ReportConfig,
}
```

Cada sección implementa `Default` con los valores de las tablas de §2, y `defaults.toml` **se genera
desde ese `Default`** por un test, para que no puedan divergir.

**[FIX]** El sistema anterior leía todo con `GetValue("Application:Settlement:MultiplierSaturday", "1.5")`:
strings con clave literal repartida por el código, parseo en el punto de uso y el default escrito de
nuevo en cada llamada. Un typo en la clave no fallaba: devolvía el default en silencio.

### 1.3 Acceso

```rust
/// Se inyecta como estado compartido. Las lecturas son sobre un snapshot inmutable.
pub type SharedConfig = Arc<RwLock<AppConfig>>;
```

- Los casos de uso reciben la sección que necesitan, no el `AppConfig` completo.
- Escribir dispara el evento `config:changed` (doc 11 §6) y persiste **sólo las claves que difieren
  del default**, para que `config.json` quede corto y legible.
- **Ninguna clave se lee con un string literal en el punto de uso.** Siempre por el campo del struct.

## 2. Catálogo de claves

Notación: `Seccion.Clave`. En `config.json` es `{"seccion": {"clave": …}}` en camelCase; como variable
de entorno, `EO_SECCION__CLAVE`.

La columna **Legado** indica de qué clave del sistema anterior proviene.

### 2.1 `Application`

| Clave | Tipo | Default | Legado | Nota |
| --- | --- | --- | --- | --- |
| `Application.Name` | string | `ElectroObra` | `Application:Name` | título de ventana y reportes |
| `Application.Environment` | enum | `Production` | `DOTNET_ENVIRONMENT` | `Development` \| `Production` |
| `Application.SeedEnabled` | bool | `false` (`true` en desarrollo) | `Application:SeedEnabled` | habilita la ruta de sembrado |
| `Application.LastPageSize` | u32 | `30` | `Application:LastPageSize` | 0, 10, 30, 50 o 100 |
| `Application.Theme` | enum | `System` | `Application:Appearance:Theme` | **[FIX]** el default era `Dark`, sin opción de seguir al sistema |
| `Application.LastRoute` | string | `dashboard` | — | **[NUEVO]** |
| `Application.SidebarExpanded` | bool | `true` | — | **[NUEVO]** |
| `Application.DataDir` | path | resuelto por el sistema | — | **[NUEVO]** sólo para pruebas y portabilidad |

### 2.2 `Locale`

**[NUEVO]** Sección completa. El sistema anterior usaba `CultureInfo.CurrentCulture` del sistema
operativo para formatear importes, así que el mismo dato se veía distinto en dos máquinas y los
reportes no eran reproducibles.

| Clave | Tipo | Default | Nota |
| --- | --- | --- | --- |
| `Locale.Language` | string | `es` | `es` \| `en` |
| `Locale.FormatoFecha` | string | `dd/MM/yyyy` | |
| `Locale.FormatoFechaHora` | string | `dd/MM/yyyy HH:mm` | |
| `Locale.PrimerDiaSemana` | u8 | `1` | 1 = lunes |
| `Locale.SimboloMoneda` | string | `$` | |
| `Locale.SeparadorMiles` | string | `.` | |
| `Locale.SeparadorDecimal` | string | `,` | |
| `Locale.DecimalesMoneda` | u8 | `2` | decimales **visibles**; se almacenan siempre 4 (doc 04) |
| `Locale.DecimalesPorcentaje` | u8 | `2` | decimales visibles de un porcentaje |
| `Locale.MonedaPorDefecto` | enum | `Ars` | |
| `Locale.ZonaHoraria` | string | `America/Argentina/Buenos_Aires` | nombre IANA; sólo afecta presentación e interpretación de fechas civiles de entrada (doc 04 §3.4) |

### 2.3 `Business`

**[NUEVO]** Sección completa. Todo lo que acá figura estaba hardcodeado en el generador de PDF.

| Clave | Tipo | Default | Nota |
| --- | --- | --- | --- |
| `Business.NombreComercial` | string | vacío | el «GENERCON» del certificado |
| `Business.Lema` | string | vacío | el «ENERGIA CONTROLADA» / «Cuentas Claras» |
| `Business.Contratista` | string | vacío | el «PABLO BAEZ» |
| `Business.Cuit` | string | vacío | |
| `Business.Direccion` | string | vacío | |
| `Business.Telefono` | string | vacío | |
| `Business.Email` | string | vacío | |
| `Business.LogoPath` | path | vacío | si está vacío, el certificado usa el nombre comercial como texto |
| `Business.IvaSugerido` | Decimal4 | `21.0000` | sólo sugiere; el IVA se ingresa a mano (doc 06 §4.1) |
| `Business.FacturaDiasVencimientoDefault` | u32 | `30` | cuando la factura no tiene vencimiento (doc 08 §2.2) |
| `Business.CategoriaProfundidadMaxima` | u8 | `3` | doc 07 §5.2 |
| `Business.DiasPorFrecuencia.Diario` | Decimal4 | `1.0000` | doc 05 §3.5 |
| `Business.DiasPorFrecuencia.Semanal` | Decimal4 | `6.0000` | semana de lunes a sábado |
| `Business.DiasPorFrecuencia.Quincenal` | Decimal4 | `15.0000` | |
| `Business.DiasPorFrecuencia.Mensual` | Decimal4 | `30.0000` | |

Si `Business.NombreComercial` está vacío, los reportes usan `Application.Name`. La pantalla de
configuración muestra un aviso mientras la sección esté sin completar, porque afecta a todos los
documentos que se entregan al cliente.

### 2.4 `Settlement`

| Clave | Tipo | Default | Legado |
| --- | --- | --- | --- |
| `Settlement.MultiplicadorSabado` | Decimal4 | `1.5000` | `Application:Settlement:MultiplierSaturday` |
| `Settlement.MultiplicadorDomingo` | Decimal4 | `2.0000` | `…MultiplierSunday` |
| `Settlement.MultiplicadorFeriado` | Decimal4 | `2.0000` | `…MultiplierHoliday` |
| `Settlement.IncluirSabado` | bool | `false` | `…IncludeSaturday` |
| `Settlement.IncluirDomingo` | bool | `false` | `…IncludeSunday` |
| `Settlement.IncluirFeriado` | bool | `false` | `…IncludeHoliday` |
| `Settlement.PeriodoPorDefectoDias` | u32 | `15` | — **[NUEVO]** el asistente arrancaba con `hoy - 15` hardcodeado |
| `Settlement.SincronizarFeriadosAlIniciar` | bool | `true` | — **[NUEVO]** |
| `Settlement.AniosFeriadosASincronizar` | u8 | `2` | — **[NUEVO]** el año actual y los siguientes |
| `Settlement.AsistenciaMaxRangoDias` | u32 | `92` | — **[NUEVO]** máximo de días que la grilla de asistencia puede consultar de una vez (últimos 3 meses) |

Los multiplicadores de aquí son los **defaults por empleado**: cada empleado tiene los suyos en su
ficha y los de configuración se usan al crear uno nuevo.

**[FIX]** `Application:Settlement:Holidays` desaparece: los feriados pasan a la tabla `feriados`
(doc 13 §3.4). Era una lista JSON dentro de un string dentro del archivo de configuración, con dos
formatos incompatibles según quién la leyera.

### 2.5 `Dashboard`

| Clave | Tipo | Default | Legado |
| --- | --- | --- | --- |
| `Dashboard.LastPeriod` | enum | `Mensual` | `Application:Dashboard:Period` |
| `Dashboard.PrivacyMode` | bool | `false` | `Application:Dashboard:IsPrivacyMode` |
| `Dashboard.CasasDolar` | lista de string | `["oficial", "blue"]` | — **[NUEVO]**, estaba hardcodeado |
| `Dashboard.CotizacionPorDefecto` | string | `blue` | — **[NUEVO]** |
| `Dashboard.TopClientesCantidad` | u8 | `5` | — **[NUEVO]** |
| `Dashboard.UltimosMovimientosCantidad` | u8 | `10` | — **[NUEVO]** |
| `Dashboard.ObrasRankingCantidad` | u8 | `5` | — **[NUEVO]** |

### 2.6 `ExternalApis`

| Clave | Tipo | Default | Legado |
| --- | --- | --- | --- |
| `ExternalApis.DollarUrl` | url | `https://dolarapi.com/v1/dolares` | `Application:Settlement:DollarApiUrl` |
| `ExternalApis.HolidayUrl` | url | `https://api.argentinadatos.com/v1/feriados/` | `Application:Settlement:HolidayApiUrl` |
| `ExternalApis.TimeoutSeconds` | u32 | `30` | `Application:HttpTimeoutSeconds` |
| `ExternalApis.Reintentos` | u8 | `2` | — **[NUEVO]** |
| `ExternalApis.DollarAutoUpdate` | bool | `true` | `Application:Dollar:AutoUpdate` |
| `ExternalApis.DollarCacheMinutes` | u32 | `60` | — **[NUEVO]** |

**[FIX]** La URL del dólar vivía bajo `Application:Settlement:*`, junto a los multiplicadores de
liquidación, que no tiene nada que ver. Las dos URL pasan a su propia sección.

### 2.7 `Attachments`

**[NUEVO]** Sección completa; no había ninguna validación configurable.

| Clave | Tipo | Default |
| --- | --- | --- |
| `Attachments.MaxSizeMb` | u32 | `25` |
| `Attachments.MaxTotalMb` | u32 | `200` |
| `Attachments.TrashRetentionDays` | u32 | `30` |
| `Attachments.ExtensionesPermitidas` | lista de string | la tabla de doc 13 §1.4 |

### 2.8 `Backup`

| Clave | Tipo | Default | Legado |
| --- | --- | --- | --- |
| `Backup.Enabled` | bool | `true` | `Application:Migration:BackupEnabled` |
| `Backup.Directory` | string | `Backups` | `Application:Migration:BackupDirectory` |
| `Backup.RetentionDays` | u32 | `30` | `Application:Migration:BackupRetentionDays` |
| `Backup.MinimoAConservar` | u8 | `3` | — **[NUEVO]** |
| `Backup.MaxAgeDays` | u32 | `7` | — **[NUEVO]** backup automático al arrancar si el último es más viejo |

### 2.9 `Communication`

| Clave | Tipo | Default | Legado |
| --- | --- | --- | --- |
| `Communication.EmailCliente` | enum | `SystemDefault` | `Application:Email:PreferredClient` |
| `Communication.GmailUrl` | string | `https://mail.google.com/mail/u/0/?view=cm&fs=1&to={email}` | `Application:Email:GmailUrl` |
| `Communication.OutlookUrl` | string | `https://outlook.live.com/mail/0/deeplink/compose?to={email}` | `…OutlookUrl` |
| `Communication.YahooUrl` | string | `https://mail.yahoo.com/d/compose-message?to={email}` | `…YahooUrl` |
| `Communication.CodigoPais` | string | `54` | — **[NUEVO]** |
| `Communication.WhatsAppTemplate` | string | valor de `Communication.WhatsAppDefault` | — **[NUEVO]** |
| `Communication.WhatsAppLiquidacionTemplate` | string | valor de `Communication.WhatsAppLiquidacionDefault` | — **[NUEVO]** |
| `Communication.EmailLiquidacionAsunto` | string | valor de `Communication.EmailLiquidacionAsuntoDefault` | `Settlements.EmailSubject` |

**[FIX]** Se elimina toda la sección `Application:Email:Smtp:*` (host, puerto, usuario, contraseña,
remitente, SSL): configuraba un servicio que nadie llamaba y guardaba la contraseña en texto plano
(doc 13 §7.3).

### 2.10 `Logging`

| Clave | Tipo | Default | Legado |
| --- | --- | --- | --- |
| `Logging.Level` | enum | `Info` (`Debug` en desarrollo) | `Logging:LogLevel:Default` |
| `Logging.RetentionDays` | u32 | `30` | — **[NUEVO]** |
| `Logging.ConsoleEnabled` | bool | `false` (`true` en desarrollo) | — **[NUEVO]** |
| `Logging.Filter` | string | vacío | — **[NUEVO]** sintaxis de `EnvFilter` |

### 2.11 `Validation`

**[NUEVO]** Los límites que el validador necesita y que no son longitudes de columna.

| Clave | Tipo | Default |
| --- | --- | --- |
| `Validation.FechaMinima` | fecha | `2000-01-01` |
| `Validation.FechaFuturaMaxDias` | u32 | `365` |

### 2.12 `Report`

| Clave | Tipo | Default |
| --- | --- | --- |
| `Report.Font` | string | `Inter` |
| `Report.MostrarLogo` | bool | `true` |
| `Report.MostrarFirmas` | bool | `true` |
| `Report.PieDePagina` | string | vacío; si está vacío se usa el nombre comercial |

## 3. Reglas de configuración

1. **Ninguna clave nueva se agrega sin figurar en este documento.** El test `config_documentada`
   compara los campos de `AppConfig` con las tablas de §2.
2. Todo valor tiene un default válido: la aplicación arranca con `config.json` borrado.
3. Los valores se validan al cargar y al escribir. Un `Application.LastPageSize` de `7` se rechaza
   con `Validation.Config.ValorNoPermitido`; no se corrige en silencio.
4. `config.json` sólo contiene lo que difiere del default.
5. Un `config.json` corrupto **no impide arrancar**: se renombra a `config.json.bak`, se registra un
   `warn` y se usa el default. **[FIX]** El sistema anterior lanzaba al deserializar y la aplicación
   no abría.
6. Nada secreto va en la configuración. Si algún día hace falta una credencial, va al almacén de
   credenciales del sistema operativo.
7. Las claves de entorno con `__` como separador de nivel se usan **sólo** en desarrollo y CI; no se
   documentan como forma de configurar la aplicación para el usuario final.

## 4. i18n

### 4.1 Reglas

1. **Todo** texto visible viene de un archivo de locale. Un literal en un `.vue` o en Rust es un
   error de revisión.
2. Las claves son jerárquicas con notación de punto: `Movements.Title`, `Validation.Cliente.NombreRequired`.
3. Los parámetros son **nombrados**, con la sintaxis de `vue-i18n`: `{nombre}`, `{max}`, `{count}`.
   **[FIX]** El sistema anterior usaba `string.Format` con índices posicionales (`{0}`, `{1}`), que
   obliga al traductor a saber el orden de los argumentos y hace imposible reordenar la frase en otro
   idioma. Además metía el formato dentro del texto traducible
   (`"{0:dd/MM/yyyy}"`, `"{1:dd/MM/yyyy}"`), lo que impide localizar el formato de fecha.
4. Los plurales usan la sintaxis de `vue-i18n`: `"{count} factura | {count} facturas"`.
   **[FIX]** El sistema anterior tenía `"{0} seleccionado(s)"`, que es lo que se escribe cuando no hay
   soporte de plurales.
5. **Los importes y las fechas no se formatean dentro de la traducción**: llegan ya formateados como
   parámetro.
6. `es.json` es la **fuente canónica**. `en.json` tiene que tener exactamente el mismo conjunto de
   claves.
7. Una clave faltante en desarrollo **falla el test**; en producción muestra la clave entre corchetes
   (`[Movements.Title]`), nunca una cadena vacía.
8. Los archivos están ordenados alfabéticamente por clave dentro de cada nivel, con un formateador
   automático. **[FIX]** El `es.json` anterior tenía las secciones en orden arbitrario y las claves
   sin ordenar, lo que hacía que cada cambio produjera un diff ilegible.

### 4.2 Estado de partida

El `es.json` del sistema anterior tiene **403 claves** repartidas en 18 secciones de nivel superior:

| Sección | Claves | Nota |
| --- | --- | --- |
| `General` | 25 | comunes |
| `Menu` | 19 | incluye `Menu.Trabajos`, sin usar |
| `Navigation` | 7 | |
| `Dashboard` | 35 | |
| `Commercial` | 13 | |
| `Movements` | 27 | |
| `Clients` | 15 | |
| `Employees` | 21 | |
| `Categories` | 6 | |
| `MovementTypes` | 6 | |
| `Reports` | 5 | |
| `Certificates` | 18 | |
| `Jobs` | 14 | |
| `Obras` | 20 | |
| `Invoices` | 11 | |
| `Settlements` | 44 | 27 propias + 17 en `Settlements.Wizard` |
| `Attendance` | 17 | 7 propias + 10 en `Attendance.TipoJornada` |
| `Settings` | 55 | 42 propias + 13 en `Settings.Migration` |
| `CommandPalette` | 2 | |
| `Seed` | 13 | |
| `Validation` | 40 | 28 traducidas de las 40 usadas en el código, doc 07 §6 |

**Desincronización con `en.json`:** 78 claves existen sólo en `es.json` y 31 sólo en `en.json`. Es
decir que la aplicación en inglés mostraba 78 claves crudas y arrastraba 31 traducciones muertas. El
test de §4.5 impide que vuelva a pasar.

### 4.3 Estructura del árbol nuevo

Se conserva la partición por módulo del sistema anterior, que funciona, con estos cambios:

| Cambio | Motivo |
| --- | --- |
| `Jobs` → `Trabajos` | el resto de las secciones de dominio están en español (`Obras`, `Movements`… ) — se unifica: **todas** las secciones de dominio usan el nombre del dominio en español: `Movimientos`, `Clientes`, `Obras`, `Trabajos`, `Certificados`, `Facturas`, `Empleados`, `Asistencia`, `Liquidaciones`, `Categorias`, `TiposMovimiento` |
| `Movements` → `Movimientos` | ídem |
| `Clients` → `Clientes` | ídem |
| `Employees` → `Empleados` | ídem |
| `Invoices` → `Facturas` | ídem |
| `Settlements` → `Liquidaciones` | ídem |
| `Attendance` → `Asistencia` | ídem |
| `Certificates` → `Certificados` | ídem |
| `Categories` → `Categorias` | ídem |
| `MovementTypes` → `TiposMovimiento` | ídem |
| `Commercial` se disuelve | sus claves van a `Clientes.CuentaCorriente.*` y `Report.*` |
| Nueva `State` | nombres de estado y errores de transición (doc 08 §6) |
| Nueva `Report` | rótulos de los reportes (doc 12) |
| Nueva `Actions` | etiquetas de acciones y transiciones |
| Nueva `Error` | errores de la envoltura IPC (doc 11 §2) |
| Nueva `Communication` | plantillas de email y WhatsApp |

**[FIX]** La mezcla de inglés y español en los nombres de sección (`Jobs` y `Obras` conviviendo) era
una fuente constante de claves duplicadas: `Jobs.Client` y `Obras.Client` existen las dos y dicen lo
mismo.

Árbol canónico de nivel superior:

```
General            Actions            Error
Menu               Navigation         CommandPalette
Dashboard          Movimientos        Clientes
Obras              Trabajos           Certificados
Facturas           Empleados          Asistencia
Liquidaciones      Categorias         TiposMovimiento
Reportes           Report             Settings
State              Validation         Communication
Seed
```

### 4.4 Secciones que se detallan en otros documentos

Para no duplicar, cada rama tiene su documento de referencia:

| Rama | Documento |
| --- | --- |
| `Validation.*` | [`07-validaciones.md`](./07-validaciones.md) §2 y §6 |
| `State.*` | [`08-maquinas-de-estado.md`](./08-maquinas-de-estado.md) §6 |
| `Menu.*`, `Navigation.*`, `CommandPalette.*` | [`10-navegacion-y-atajos.md`](./10-navegacion-y-atajos.md) |
| `Error.*` | [`11-contratos-tauri.md`](./11-contratos-tauri.md) §2 |
| `Report.*` | [`12-reportes-y-exportaciones.md`](./12-reportes-y-exportaciones.md) |
| `Communication.*` | [`13-servicios-externos-y-archivos.md`](./13-servicios-externos-y-archivos.md) §7 |
| El resto | §4.6 de este documento |

### 4.5 Convenciones de nombre de clave

| Tipo de texto | Patrón | Ejemplo |
| --- | --- | --- |
| Título de pantalla | `<Modulo>.Title` | `Movimientos.Title` |
| Subtítulo | `<Modulo>.Subtitle` | |
| Encabezado de columna | `<Modulo>.Col.<Campo>` | `Movimientos.Col.Concepto` |
| Etiqueta de campo de formulario | `<Modulo>.Field.<Campo>` | `Movimientos.Field.Monto` |
| Texto de ayuda de un campo | `<Modulo>.Hint.<Campo>` | |
| Marcador de posición | `<Modulo>.Placeholder.<Campo>` | |
| Título de alta / edición | `<Modulo>.NewTitle` / `<Modulo>.EditTitle` | |
| Confirmación de borrado | `<Modulo>.DeleteConfirm` | con `{nombre}` |
| Estado vacío | `<Modulo>.EmptyState` | |
| Acción | `Actions.<Modulo>.<Accion>` | `Actions.Factura.Emitir` |
| Mensaje de éxito | `<Modulo>.Success.<Operacion>` | |
| Nombre de estado | `State.<Entidad>.<Variante>` | `State.Factura.Emitida` |
| Validación | `Validation.<Entidad>.<Regla>` | |
| Error | `Error.<Categoria>` | `Error.Internal` |

**[FIX]** El sistema anterior mezclaba patrones sin criterio: los encabezados de columna a veces eran
`Movements.Amount` y a veces `Movements.Total`; los nombres de campo a veces `Clients.Name` y a veces
`Employees.FullName` para la misma cosa; las confirmaciones de borrado sí eran consistentes
(`<Modulo>.DeleteConfirm`), y eso se conserva.

### 4.6 Claves comunes canónicas

Se conservan las 25 de `General` y se agregan las que faltaban. Estas son las que aparecen en más de
un módulo y **no** se duplican por módulo:

```json
{
  "General": {
    "Add": "Agregar",
    "AppName": "ElectroObra",
    "Cancel": "Cancelar",
    "ClearFilters": "Limpiar filtros",
    "Close": "Cerrar",
    "Delete": "Eliminar",
    "DeleteSuccess": "Registro eliminado.",
    "DiscardChangesConfirm": "Hay cambios sin guardar. ¿Descartarlos?",
    "Edit": "Editar",
    "Error": "Error",
    "Export": "Exportar",
    "From": "Desde",
    "ItemsPerPage": "Elementos por página:",
    "Loading": "Cargando…",
    "New": "Nuevo",
    "Next": "Siguiente",
    "NoData": "No hay datos para mostrar",
    "NoResults": "Ningún registro coincide con los filtros",
    "OpenFolder": "Abrir carpeta",
    "PageSizeAll": "Todos",
    "Previous": "Anterior",
    "Refresh": "Actualizar",
    "Retry": "Reintentar",
    "Save": "Guardar",
    "SaveAndNew": "Guardar y nuevo",
    "SaveChanges": "Guardar cambios",
    "SaveSuccess": "Cambios guardados.",
    "Search": "Buscar…",
    "Select": "Seleccionar…",
    "SelectRecordHint": "Seleccioná un registro de la lista para ver o editar.",
    "Success": "Éxito",
    "To": "Hasta",
    "Total": "Total",
    "DateRange": {
      "Hoy": "Hoy",
      "EstaSemana": "Esta semana",
      "EsteMes": "Este mes",
      "MesAnterior": "Mes anterior",
      "EsteAnio": "Este año"
    }
  }
}
```

**[FIX]** `General.All` (`"Todos"`) y `General.NoData` se usaban para tres cosas distintas: la opción
«todos» de un filtro, la etiqueta del tamaño de página `0` y el estado vacío de una lista. Ahora son
`General.All`, `General.PageSizeAll` y `General.NoData` / `General.NoResults`.

### 4.7 Textos que dejan de ser i18n

Estos estaban en `es.json` y **no** son texto de interfaz:

| Clave | Qué era | A dónde va |
| --- | --- | --- |
| `General.AppName` | el nombre de la aplicación | se conserva como fallback, pero la fuente es `Application.Name` |
| `Navigation.Breadcrumb` | la plantilla `"{0} / {1}"` | las migas se construyen por componentes (doc 10 §6) |
| `Navigation.SearchShortcut` | el literal `"Ctrl+K"` | se deriva del registro de atajos |
| `Dashboard.DatabaseStatus` | la plantilla `"SQLite: {0}"` | el motor no es un texto traducible |
| `Settlements.EmailSubject` | asunto con formato de fecha embebido | `Communication.EmailLiquidacionAsuntoDefault`, sin formato |
| `Settlements.WhatsAppMessage` | ídem | `Communication.WhatsAppLiquidacionDefault` |

### 4.8 Textos que se agregan porque estaban hardcodeados

Inventario de los literales en español encontrados en el código del sistema anterior. Cada uno pasa
a ser una clave:

| Literal | Ubicación original | Clave nueva |
| --- | --- | --- |
| `"Nuevo Movimiento"` | `MovimientosViewModel` | `Movimientos.NewTitle` |
| `"Editar Movimiento"` | ídem | `Movimientos.EditTitle` |
| `"Editar Cliente"` | `ClientesViewModel` | `Clientes.EditTitle` |
| `"Editar Trabajo"` | `TrabajosViewModel` | `Trabajos.EditTitle` |
| `"Todos"` | `PageSizeConverter` | `General.PageSizeAll` |
| `"Presupuestado"`, `"En Curso"`, `"Pausado"`, `"Finalizado"`, `"Cancelado"` | `EstadoTrabajoDisplayConverter` | `State.Trabajo.*` |
| `"Mensual"`, `"Anual"`, `"Total"` | `DashboardViewModel` | `Dashboard.Period.*` |
| `"Saludable"` | `DashboardViewModel` | `Dashboard.DbStatus.Healthy` |
| `"General"`, `"Personalización"`, `"Liquidaciones"`, `"Sistema"` | `SettingsView` | `Settings.Section.*` |
| `"PABLO BAEZ"`, `"GENERCON"`, `"ENERGIA CONTROLADA"` | `ExportService` | no son i18n: van a `Business.*` |
| `"Cuentas Claras"`, `"Software de Gestión Profesional"` | ídem | `Business.Lema` y `Report.PieDePagina` |
| `"Obra General"` | ídem | `Report.Certificado.ObraGeneral` |
| `"Días trabajados en el periodo"` | ídem | `Report.Liquidacion.DiasTrabajados` |
| `"No se registraron adelantos en este periodo"` | ídem | `Report.Liquidacion.SinAdelantos` |
| `"Firma de Revisión"`, `"Generado por Administración"` | ídem | `Report.Liquidacion.Firma*` |
| `"Fecha"`, `"Concepto"`, `"Tipo"`, `"Monto"`, `"Cantidad"`, `"Total"` | ídem, todos los encabezados de reporte | `Report.Col.*` |
| `"UND"`, `"CANT"`, `"ANT"`, `"ACT"`, `"ACU"`, `"CÓMPUTOS"`, `"AVANCE (%)"`, `"IMPORTE"`, `"SUB-TOTAL:"`, `"AJUSTE UOCRA"`, `"OTROS DESCUENTOS:"`, `"TOTAL A FACTURAR:"` | ídem | `Report.Certificado.*` |
| `"Hola {Nombre}, me pongo en contacto contigo desde ElectroObraApp."` | `EmpleadosViewModel` | `Communication.WhatsAppDefault` |
| `"Manual"` | `SettingsViewModel`, nombre de feriado por defecto | `Settings.Feriado.NombrePorDefecto` |
| `"Feriado"` | ídem, fallback al deserializar | eliminado: la tabla exige nombre |
| `"📋"` | `CertificadosView`, panel vacío | icono, no texto |

### 4.9 Carga en el frontend

```
src/locales/
├── es.json
├── en.json
└── index.ts        # registra los locales y configura vue-i18n
```

Los locales se **empaquetan** con la aplicación, no se leen del disco en tiempo de ejecución. Cambiar
de idioma no requiere reiniciar: `vue-i18n` reacciona al cambio de `locale`.

**[FIX]** El sistema anterior leía los JSON como recursos embebidos y algunos cambios de idioma
requerían reiniciar («Nota: Algunos cambios estéticos pueden requerir reiniciar la aplicación»).

Los mensajes que el **backend** produce (validaciones, conflictos, errores) viajan como **clave +
parámetros** y los traduce el frontend. El backend no tiene tabla de traducciones: no le hace falta y
así no hay dos copias del mismo texto.

## 5. Tests obligatorios

| Test | Qué verifica |
| --- | --- |
| `config_arranca_sin_archivo` | con `config.json` ausente, `AppConfig::default()` es válido |
| `config_corrupto_no_impide_arrancar` | un JSON inválido se renombra y se usa el default |
| `config_solo_persiste_diferencias` | cambiar una clave produce un `config.json` con una sola clave |
| `config_rechaza_valores_invalidos` | `LastPageSize = 7` falla |
| `config_defaults_toml_coincide_con_default_rust` | genera el TOML y lo compara con el archivo |
| `config_documentada` | todo campo de `AppConfig` figura en las tablas de §2 |
| `locales_tienen_las_mismas_claves` | `es.json` y `en.json` tienen conjuntos idénticos |
| `locales_estan_ordenados` | el formateador no produce cambios |
| `toda_clave_usada_existe` | recorre los `$t('…')` del frontend y las claves de Rust, y verifica que existan en ambos locales |
| `ninguna_clave_esta_sin_usar` | advierte (no falla) sobre claves definidas y no referenciadas |
| `ninguna_traduccion_tiene_formato_de_fecha` | ningún valor contiene `dd/MM`, `MM/dd` ni `yyyy` |
| `ninguna_traduccion_usa_parametros_posicionales` | ningún valor contiene `{0}`, `{1}`… |
| `ningun_literal_visible_en_vue` | busca cadenas de texto en templates fuera de `$t()` |
| `ningun_literal_visible_en_rust` | busca literales en los generadores de reportes |
| `claves_de_validacion_coinciden_con_doc_07` | las ramas `Validation.*` de los locales coinciden con las claves declaradas en Rust |
| `claves_de_estado_coinciden_con_los_enums` | existe `State.<Entidad>.<Variante>` para toda variante |
