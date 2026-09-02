# 13 — Servicios externos y archivos

> Define los adaptadores de `crates/eo-infrastructure/src/external/`, `.../files/` y
> `.../backup/`. Cada uno implementa un **puerto** (trait) declarado en `eo-application`, así que en
> los tests se sustituye por un doble sin tocar la red ni el disco.

## 1. Adjuntos

### 1.1 Puerto

```rust
// crates/eo-application/src/ports/attachments.rs
#[async_trait]
pub trait AttachmentStore: Send + Sync {
    async fn add(&self, req: AddAttachment) -> Result<Adjunto, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
    async fn open(&self, id: Uuid) -> Result<(), AppError>;
    async fn reveal(&self, id: Uuid) -> Result<(), AppError>;
    async fn list(&self, entidad_tipo: EntidadAdjunto, entidad_id: Uuid)
        -> Result<Vec<Adjunto>, AppError>;
}
```

### 1.2 Convención de rutas

Se conserva la del sistema anterior, que es correcta:

```
{data_dir}/attachments/{entidad_tipo}/{entidad_id}/{uuid}_{nombre_saneado}
```

Ejemplo:

```
C:\Users\usuario\AppData\Local\ElectroObraApp\attachments\Movimiento\0192f3a0-…\0192f3b1-…_factura_luz.pdf
```

La columna `adjuntos.ruta_relativa` guarda **sólo la parte relativa**, con `/` como separador en
todas las plataformas:

```
Movimiento/0192f3a0-…/0192f3b1-…_factura_luz.pdf
```

El prefijo `{uuid}_` garantiza que dos archivos con el mismo nombre no colisionen y que el nombre
original siga siendo legible.

`entidad_tipo` es el nombre de la variante de `EntidadAdjunto` (doc 05 §3.7), no una cadena libre.

### 1.3 Validaciones

**[FIX]** El sistema anterior sólo saneaba el nombre del archivo. No validaba **nada** más: se podía
adjuntar un `.exe` de 2 GB y la aplicación lo copiaba al directorio de datos del usuario.

| Validación | Regla | Error |
| --- | --- | --- |
| Tamaño máximo | `Attachments.MaxSizeMb`, default **25 MB** | `Validation.Adjunto.DemasiadoGrande` con `params { max, actual }` |
| Extensión permitida | lista blanca de §1.4 | `Validation.Adjunto.ExtensionNoPermitida` con `params { extension }` |
| Nombre saneado | se quitan los caracteres inválidos del sistema de archivos, los de control y los puntos iniciales; se colapsan los espacios; máximo 200 caracteres conservando la extensión; si queda vacío, `archivo` | — |
| Ruta de origen | debe existir, ser un archivo (no un directorio ni un enlace simbólico) y ser legible | `Io` |
| Cupo total | `Attachments.MaxTotalMb` por entidad, default **200 MB** | `Validation.Adjunto.CupoExcedido` |
| Tipo real | los primeros bytes tienen que coincidir con la extensión, para PDF, PNG, JPEG, GIF, WEBP, ZIP/OOXML | `Validation.Adjunto.ContenidoNoCoincide` |

La verificación de bytes iniciales evita el caso obvio de renombrar un ejecutable a `.pdf`. No es una
defensa completa, pero es la que corresponde a una aplicación de escritorio de un solo usuario.

Los límites salen de configuración: no se escriben en el código.

### 1.4 Extensiones y tipos MIME

Lista blanca completa. Se conservan los 12 tipos que el sistema anterior sabía identificar y se
agregan los que faltaban.

| Extensión | MIME | Nota |
| --- | --- | --- |
| `.pdf` | `application/pdf` | |
| `.jpg`, `.jpeg` | `image/jpeg` | |
| `.png` | `image/png` | |
| `.gif` | `image/gif` | |
| `.webp` | `image/webp` | |
| `.heic` | `image/heic` | **[NUEVO]** las fotos de iPhone salen así |
| `.txt` | `text/plain` | |
| `.csv` | `text/csv` | |
| `.doc` | `application/msword` | |
| `.docx` | `application/vnd.openxmlformats-officedocument.wordprocessingml.document` | |
| `.xls` | `application/vnd.ms-excel` | |
| `.xlsx` | `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet` | |
| `.odt` | `application/vnd.oasis.opendocument.text` | **[NUEVO]** |
| `.ods` | `application/vnd.oasis.opendocument.spreadsheet` | **[NUEVO]** |
| `.zip` | `application/zip` | **[NUEVO]** |
| `.eml` | `message/rfc822` | **[NUEVO]** para guardar el mail del proveedor |

Una extensión fuera de la lista se **rechaza**. **[FIX]** El sistema anterior le asignaba
`application/octet-stream` y la aceptaba igual.

La tabla vive en un `const` de `files/mime.rs`, no repartida por el código.

### 1.5 Abrir y revelar

| Operación | Windows | Linux | macOS |
| --- | --- | --- | --- |
| Abrir | `ShellExecute` sobre el archivo | `xdg-open <ruta>` | `open <ruta>` |
| Revelar | `explorer.exe /select,<ruta>` | `xdg-open <directorio>` | `open -R <ruta>` |

Se usa la API de Tauri (`opener`) cuando alcanza, y el comando del sistema para «revelar» en Windows,
que la API no cubre.

**[NUEVO]** «Revelar en el explorador» no existía; sólo se podía abrir.

Antes de abrir se verifica que el archivo exista. Si no está, el error es
`Validation.Adjunto.ArchivoNoEncontrado` con la ruta relativa como parámetro, no una excepción sin
traducir.

### 1.6 Borrado

1. Borrado **lógico** de la fila (`is_deleted = 1`).
2. El archivo físico **se mueve** a `{data_dir}/attachments/.trash/{uuid}_{nombre}`, no se borra.
3. Los archivos de `.trash` con más de `Attachments.TrashRetentionDays` días (default 30) se eliminan
   en la tarea de mantenimiento (§6).

**[FIX]** El sistema anterior borraba el archivo físico de inmediato y hacía borrado lógico de la
fila. Resultado: el registro se podía «restaurar» pero el archivo ya no existía. Con la papelera, un
borrado por error es reversible durante 30 días.

Al quedar vacío, el directorio de la entidad se elimina.

## 2. Cotización del dólar

### 2.1 Puerto y adaptador

```rust
#[async_trait]
pub trait ExchangeRateProvider: Send + Sync {
    async fn fetch(&self) -> Result<Vec<Cotizacion>, AppError>;
}
```

| Propiedad | Valor |
| --- | --- |
| URL | `ExternalApis.DollarUrl`, default `https://dolarapi.com/v1/dolares` |
| Método | `GET` |
| Cabeceras | `Accept: application/json`, `User-Agent: Certaro/{version}` **[NUEVO]** |
| Timeout | `ExternalApis.TimeoutSeconds`, default **30 s** |
| Reintentos | 2, con espera de 1 s y 3 s **[NUEVO]** |

**[NUEVO]** El sistema anterior no mandaba `User-Agent` ni `Accept` y no reintentaba: un corte de red
de dos segundos dejaba el dashboard sin cotizaciones hasta la próxima visita.

### 2.2 Forma de la respuesta

```json
[
  {
    "moneda": "USD",
    "casa": "oficial",
    "nombre": "Oficial",
    "compra": 950.5,
    "venta": 990.5,
    "fechaActualizacion": "2026-08-28T10:00:00.000Z"
  }
]
```

Campos que se leen: `nombre`, `casa`, `compra`, `venta`, `fechaActualizacion`. Cualquier campo extra
se ignora. Un elemento con `compra` o `venta` no numéricos se descarta y se registra un `warn`, sin
invalidar el resto de la lista.

`compra` y `venta` llegan como número JSON y se convierten a `Money` **por su representación
textual**, no por `f64`, para no arrastrar error de punto flotante (doc 04 §1.3).

### 2.3 Casas que se muestran

El dashboard muestra `casa == "oficial"` y `casa == "blue"`, comparando en minúsculas.

**[FIX]** La comparación anterior era sensible a mayúsculas y además el DTO tenía `Casa` con valor
por defecto vacío, así que si la API cambiaba la capitalización el bloque desaparecía sin explicación.

Las casas visibles son configurables: `Dashboard.CasasDolar`, default `["oficial", "blue"]`.

### 2.4 Degradación y caché

- Cualquier error (timeout, HTTP distinto de 2xx, JSON inválido) devuelve **lista vacía**, se registra
  en el log con el nivel `warn` y **no** se muestra ningún error al usuario. La cotización es
  accesoria: la aplicación funciona sin ella.
- **[NUEVO] Caché:** la última respuesta correcta se guarda en `app_metadata` con su marca de tiempo.
  Si la petición falla, se devuelve la cotización cacheada marcada como `desactualizada: true`, y la
  interfaz muestra «al {fecha}». Un valor viejo con su fecha es más útil que ningún valor.
- Tiempo de vida de la caché: `ExternalApis.DollarCacheMinutes`, default **60 minutos**. Dentro de
  ese plazo no se vuelve a llamar a la API.

**[FIX]** Sin caché, el servicio se llamaba en cada construcción del ViewModel del dashboard.

### 2.5 Uso en movimientos

Al cargar un movimiento en dólares, el campo de cotización se precarga con la **venta** de la casa
`Dashboard.CotizacionPorDefecto` (default `blue`). El usuario puede sobreescribirla; lo que se
persiste es lo que quedó en el campo, no lo que dijo la API.

## 3. Feriados

### 3.1 Puerto y adaptador

```rust
#[async_trait]
pub trait HolidayProvider: Send + Sync {
    async fn fetch(&self, anio: i32) -> Result<Vec<Feriado>, AppError>;
}
```

| Propiedad | Valor |
| --- | --- |
| URL | `{ExternalApis.HolidayUrl}{anio}`, base default `https://api.argentinadatos.com/v1/feriados/` |
| Método | `GET` |
| Timeout | el mismo, 30 s |
| Reintentos | 2 |

La base se normaliza agregando `/` final si no lo tiene, igual que el sistema anterior. Ejemplo
resultante: `https://api.argentinadatos.com/v1/feriados/2026`.

### 3.2 Forma de la respuesta

```json
[
  { "fecha": "2026-01-01", "tipo": "inamovible", "nombre": "Año Nuevo" },
  { "fecha": "2026-05-25", "tipo": "inamovible", "nombre": "Día de la Revolución de Mayo" }
]
```

Se leen `fecha` y `nombre`. `fecha` se parsea como `NaiveDate` en formato `YYYY-MM-DD`; si no parsea,
el elemento se descarta con un `warn`. **[NUEVO]** También se guarda `tipo`, que sirve para
distinguir feriados trasladables.

### 3.3 Degradación

Ante cualquier error: **lista vacía**, con un `warn` en el log.

**[FIX]** El manejo anterior era un `catch (Exception) { }` con un comentario que decía «Log error or
handle as needed». No registraba nada: si la API cambiaba de forma, los feriados dejaban de contar en
las liquidaciones y no quedaba rastro de por qué.

Consecuencia funcional a documentar: si los feriados no se pueden obtener, la sugerencia de
liquidación **no aplica el multiplicador de feriado**. Eso paga de menos. Por eso el asistente de
liquidación muestra una advertencia visible (`Settlements.Warning.FeriadosNoDisponibles`) cuando el
calendario del período está vacío, y ofrece continuar o cancelar. **[NUEVO]**

### 3.4 Almacenamiento local

**[FIX]** El sistema anterior guardaba los feriados manuales en una clave JSON del archivo de
configuración (`Application:Settlement:Holidays`) y tenía **dos deserializaciones incompatibles**: la
pantalla de configuración escribía `[{"Date":…,"Name":…}]` y el recálculo de liquidación leía
`["2026-01-01T00:00:00"]`. Los feriados cargados a mano desde configuración **nunca** llegaban al
cálculo. Y el servicio de liquidación del backend ni siquiera miraba los manuales: usaba sólo la API.

En el sistema nuevo los feriados viven en una **tabla**:

```sql
CREATE TABLE feriados (
    fecha       TEXT NOT NULL PRIMARY KEY,   -- YYYY-MM-DD
    nombre      TEXT NOT NULL,
    tipo        TEXT NULL,                   -- de la API
    origen      TEXT NOT NULL,               -- 'Api' | 'Manual'
    created_at  TEXT NOT NULL
) WITHOUT ROWID;
```

Reglas:

- La sincronización con la API hace `INSERT OR IGNORE`: **nunca** sobreescribe un feriado con
  `origen = 'Manual'`. Lo que el usuario cargó a mano gana.
- El cálculo de liquidación lee **de la tabla**, en una sola consulta por rango. No llama a la API.
  Una sola fuente de verdad, consultable, y el cálculo no depende de la red.
- La sincronización se dispara desde configuración y automáticamente al arrancar si el año en curso
  no tiene feriados cargados.
- Años sincronizados por defecto: el actual y el siguiente.

Esta tabla se agrega al DDL de [`03-modelo-de-datos.md`](./03-modelo-de-datos.md) como tabla 21.

## 4. Backup

### 4.1 Crear

```sql
VACUUM INTO '<ruta_destino>';
PRAGMA integrity_check;
```

Se conserva el enfoque del sistema anterior, que es el correcto: `VACUUM INTO` produce una copia
consistente sin bloquear ni requerir que la aplicación se cierre.

| Propiedad | Valor |
| --- | --- |
| Directorio | `{data_dir}/{Backup.Directory}`, default `Backups` |
| Nombre | `Certaro_{yyyyMMdd_HHmmss}.db`, con la hora en **UTC** |
| Verificación | `PRAGMA integrity_check` debe devolver exactamente `ok` (sin distinguir mayúsculas) |
| Retención | `Backup.RetentionDays`, default **30** días |
| Retención mínima | **[NUEVO]** se conservan siempre los 3 backups más recientes, aunque superen los 30 días |

**[FIX]** La limpieza por antigüedad podía dejar **cero** backups si la aplicación no se usaba por más
de 30 días: al volver a abrirla, se hacía un backup nuevo y se borraban todos los anteriores por
viejos. Con el mínimo de 3, siempre queda algo a lo que volver.

La ruta de destino se escapa duplicando las comillas simples antes de interpolarla en el `VACUUM
INTO`. Es la única interpolación de texto en un `SQL` de todo el sistema y está justificada porque
`VACUUM INTO` no admite parámetros. La ruta no proviene del usuario: se construye a partir del
directorio de datos y de una marca de tiempo.

### 4.2 Cuándo se hace solo

| Disparador | Nota |
| --- | --- |
| Antes de aplicar migraciones | si `Backup.Enabled` (default `true`) |
| Antes de restaurar un backup | para poder deshacer la restauración |
| Antes de importar un JSON | ídem |
| Al arrancar, si el último backup tiene más de `Backup.MaxAgeDays` días (default 7) | **[NUEVO]** |

### 4.3 Restaurar

1. Verificar que el archivo exista.
2. `PRAGMA integrity_check` sobre el archivo de backup.
3. **[NUEVO]** Verificar que la versión del esquema del backup sea menor o igual a la actual. Si es
   mayor, se rechaza: un backup de una versión más nueva de la aplicación no se puede restaurar sobre
   una vieja. Error `Backup.VersionIncompatible`.
4. **[NUEVO]** Backup automático del estado actual.
5. Cerrar todas las conexiones al archivo de la base.
6. Copiar el backup a `{db}.restore.tmp` y de ahí al archivo de la base.
7. Borrar el temporal.
8. Reabrir la conexión y aplicar las migraciones pendientes.
9. Registrar un `warn` con la ruta del backup restaurado.

**[FIX]** El paso 5 no existía: se copiaba sobre el archivo con la conexión abierta. En Windows eso
falla o corrompe; en Linux puede parecer que funciona y dejar la base en un estado inconsistente con
los archivos `-wal` y `-shm`, que tampoco se borraban.

Los archivos `-wal` y `-shm` se eliminan antes de copiar. **[NUEVO]**

### 4.4 Estructura del directorio de datos

```
{data_dir}/                       # %LOCALAPPDATA%\FittyAr\Certaro en Windows
                                  # ~/.local/share/FittyAr/Certaro en Linux
                                  # ~/Library/Application Support/FittyAr/Certaro en macOS
├── certaro.db
├── certaro.db-wal
├── certaro.db-shm
├── config.json                   # configuración mutable del usuario (doc 14)
├── Backups/
│   └── certaro_20260829_143012.db
├── attachments/
│   ├── Movimiento/{id}/…
│   └── .trash/
├── logs/
│   └── certaro-20260829.log
└── exports/                      # [NUEVO] último destino usado, para el diálogo de guardado
```

**[FIX]** El sistema anterior guardaba la configuración mutable en un archivo llamado
`appsettings.json` **dentro del directorio de datos**, con el mismo nombre que el archivo de
configuración empaquetado con la aplicación. Los dos archivos se confundían constantemente al
depurar. El mutable pasa a llamarse `config.json`.

La resolución del directorio de datos usa la API de Tauri (`app_data_dir`), no una construcción
manual con carpetas especiales.

## 5. Exportar e importar JSON de la base

Sirve para dos cosas: un respaldo legible que no depende del formato binario de SQLite, y mover los
datos entre máquinas.

### 5.1 Exportar

```json
{
  "formatVersion": 2,
  "appVersion": "0.1.0",
  "schemaVersion": "m20260901_000001_init",
  "exportedAt": "2026-08-29T14:30:12Z",
  "tables": {
    "movimientos": {
      "columns": ["id", "fecha", "concepto", "monto", "..."],
      "rows": [
        ["0192f3a0-…", "2026-08-14T00:00:00Z", "Cable 2.5 mm", 15000000, "..."]
      ]
    }
  }
}
```

**[FIX]** Tres cambios respecto del formato anterior:

- **`schemaVersion`**: el formato anterior sólo tenía la versión de la aplicación, así que al importar
  no había forma de saber si el esquema coincidía. Ahora se compara con la última migración aplicada
  y se rechaza si no es compatible.
- **Filas como arrays con las columnas declaradas una vez**: el formato anterior repetía el nombre de
  cada columna en cada fila. Con 20.000 movimientos el archivo pesaba varias veces lo necesario.
- **`formatVersion`**: para poder cambiar el formato sin romper los archivos ya exportados.

Los valores se serializan tal como están en la base: los importes escalados como enteros, las fechas
como el texto que guarda SQLite. Es un volcado, no un reporte: la fidelidad importa más que la
legibilidad.

La lista de tablas se obtiene del **modelo**, no de `sqlite_master`, y excluye las tablas internas de
migración.

### 5.2 Importar

Pasos, todos dentro de una transacción:

1. Backup automático.
2. Validar `formatVersion` y `schemaVersion`.
3. `PRAGMA foreign_keys = OFF`.
4. Para cada tabla del archivo:
   - verificar que esté en la lista blanca derivada del modelo; si no, abortar,
   - verificar que **cada** columna esté en la lista blanca de esa tabla; si no, abortar,
   - `DELETE FROM "<tabla>"`,
   - insertar las filas con sentencias **parametrizadas**.
5. `PRAGMA foreign_keys = ON`.
6. `PRAGMA foreign_key_check` **[NUEVO]**: si hay violaciones, se revierte todo.
7. Confirmar la transacción.

Reglas de seguridad, heredadas y ampliadas:

- Los nombres de tabla y de columna se validan con `^[A-Za-z_][A-Za-z0-9_]*$` **y** contra la lista
  blanca del modelo. Ninguno se interpola sin pasar por ambas.
- Los valores **siempre** van como parámetros. Nunca se concatenan.
- El orden de inserción respeta las dependencias de claves foráneas, para que
  `foreign_key_check` del paso 6 sea significativo.

**[FIX]** La importación anterior no estaba dentro de una transacción: si fallaba en la tabla 12 de
16, la base quedaba con las 11 primeras reemplazadas y el resto viejas. Tampoco verificaba la
integridad referencial al terminar. Y no tenía interfaz: la función existía y ningún botón la
llamaba.

## 6. Tarea de mantenimiento

**[NUEVO]** No existía. Corre al arrancar, en segundo plano, después de `db:ready`, y una vez al día
si la aplicación queda abierta.

| Tarea | Detalle |
| --- | --- |
| Recalcular estados de factura | doc 08 §2.4, T-F11 |
| Sincronizar feriados | si falta el año en curso o el siguiente |
| Actualizar cotizaciones | si la caché venció |
| Limpiar backups | por antigüedad, respetando el mínimo de 3 |
| Vaciar la papelera de adjuntos | archivos con más de 30 días |
| Rotar logs | los de más de `Logging.RetentionDays` días, default 30 |
| Registrar métricas de la base | tamaño, cantidad de filas por tabla, en `app_metadata` |

Ninguna de estas tareas bloquea la interfaz ni muestra un diálogo. Los errores se registran y no
interrumpen: si la sincronización de feriados falla, el resto sigue.

## 7. Email y WhatsApp

Ambos son **enlaces profundos**: la aplicación no envía nada, abre el cliente del sistema. Se
conserva el enfoque.

### 7.1 Email

| Cliente preferido | URL |
| --- | --- |
| `SystemDefault` | `mailto:{destinatarios}?subject={asunto}&body={cuerpo}` |
| `Gmail` | `Email.GmailUrl`, default `https://mail.google.com/mail/u/0/?view=cm&fs=1&to={email}` |
| `Outlook` | `Email.OutlookUrl`, default `https://outlook.live.com/mail/0/deeplink/compose?to={email}` |
| `Yahoo` | `Email.YahooUrl`, default `https://mail.yahoo.com/d/compose-message?to={email}` |

El marcador `{email}` se reemplaza por los destinatarios **codificados para URL**. El asunto y el
cuerpo se agregan como parámetros, codificados, respetando si la plantilla ya tenía `?`.

**[NUEVO]** Varios destinatarios: con la tabla de contactos por cliente (RC-13), el envío admite una
lista separada por comas. La interfaz permite elegir a cuáles enviar.

Los asuntos y cuerpos son **plantillas i18n con parámetros nombrados**, no `string.Format` con
índices posicionales.

**[FIX]** El sistema anterior tenía `"Liquidación del periodo {0:dd/MM/yyyy} al {1:dd/MM/yyyy}"` con
el formato de fecha embebido en la cadena traducible. Un traductor al inglés no puede cambiar
`dd/MM/yyyy` por `MM/dd/yyyy` sin que se rompa. Ahora las fechas se formatean antes y la plantilla
recibe `{desde}` y `{hasta}` ya como texto.

### 7.2 WhatsApp

```
https://api.whatsapp.com/send?phone={telefono_normalizado}&text={mensaje_codificado}
```

El teléfono se **normaliza** antes: se quitan espacios, guiones, paréntesis y el `+`, y se agrega el
prefijo de país de `Communication.CodigoPais` (default `54`) si no lo tiene.

**[FIX]** El teléfono se pasaba tal como estaba cargado. Un teléfono con formato
`(011) 4567-8901` producía un enlace roto, y no había ningún mensaje de error: se abría WhatsApp con
un número inválido.

Si el teléfono queda vacío tras normalizar, la acción se deshabilita con el tooltip
`Communication.SinTelefono`.

Los mensajes son plantillas configurables con valor inicial i18n:

| Plantilla | Clave del valor inicial | Marcadores |
| --- | --- | --- |
| `Communication.WhatsAppTemplate` | `Communication.WhatsAppDefault` | `{nombre}` |
| `Communication.WhatsAppLiquidacionTemplate` | `Communication.WhatsAppLiquidacionDefault` | `{nombre}`, `{desde}`, `{hasta}` |

**[FIX]** El mensaje para empleados,
`"Hola {Nombre}, me pongo en contacto contigo desde ElectroObraApp."`, estaba escrito en el código.
Además mezclaba tratamientos: usaba «contigo» (tuteo peninsular) mientras el resto de la aplicación
está en español rioplatense.

### 7.3 SMTP

El sistema anterior tenía un servicio SMTP completo, con configuración de host, puerto, usuario,
contraseña y SSL, que **nadie llamaba**: todos los caminos de envío usaban los enlaces profundos.

**Decisión: no se implementa.** No se porta código muerto. Si en el futuro hace falta enviar
liquidaciones por correo sin intervención del usuario, se agrega como un módulo nuevo con su propia
documentación, y sobre todo con un lugar seguro para la contraseña — la configuración anterior la
guardaba en texto plano en un JSON del directorio de datos, que es exactamente lo que no hay que
hacer.

## 8. Logging

| Propiedad | Valor |
| --- | --- |
| Biblioteca | `tracing` + `tracing-subscriber` |
| Destinos | consola (sólo en desarrollo) y archivo |
| Ruta | `{data_dir}/logs/Certaro-{yyyyMMdd}.log` |
| Rotación | diaria |
| Retención | `Logging.RetentionDays`, default 30 |
| Nivel por defecto | `info` en producción, `debug` en desarrollo |
| Sobreescritura | variable de entorno `EO_LOG` con la sintaxis de `EnvFilter` |
| Formato del archivo | JSON por línea |

**[FIX]** El formato anterior era texto con plantilla, lo que hace imposible filtrar un log de
miles de líneas por algo que no sea `grep`. Con JSON por línea se puede consultar con `jq`.

Cada operación de negocio abre un `span` con su `trace_id`, el mismo que viaja en `ApiError`
(doc 11 §2), de modo que un error reportado por el usuario se localiza en el log por ese
identificador.

**Nunca se registran:** contraseñas, contenido de adjuntos, ni el detalle de importes de una entidad
en un log de nivel `info`. Los importes van en `debug`.

El sink a base de datos que el sistema anterior tenía como stub comentado **no se implementa**: en una
aplicación de escritorio de un solo usuario, el archivo alcanza. Se documenta la decisión para que no
reaparezca como un `TODO`.

## 9. Tests obligatorios

| Test | Qué verifica |
| --- | --- |
| `adjunto_ruta_sigue_la_convencion` | la ruta relativa generada es exactamente `{tipo}/{id}/{uuid}_{nombre}` con `/` |
| `adjunto_nombre_se_sanea` | `../../etc/passwd` y `con.txt` producen nombres seguros |
| `adjunto_rechaza_extension_no_permitida` | `.exe` falla con la clave correcta |
| `adjunto_rechaza_tamano_excesivo` | un archivo de `max + 1` bytes falla |
| `adjunto_rechaza_contenido_que_no_coincide` | un ejecutable renombrado a `.pdf` falla |
| `adjunto_borrado_mueve_a_papelera` | el archivo aparece en `.trash` y no en su lugar original |
| `dolar_error_devuelve_cache` | con la API caída y caché presente, devuelve la caché marcada como desactualizada |
| `dolar_error_sin_cache_devuelve_vacio` | y no propaga el error |
| `dolar_ignora_elemento_invalido` | 3 elementos, uno con `venta` no numérica, devuelve 2 |
| `dolar_no_usa_f64` | `990.50` se convierte a `Money` sin pérdida |
| `feriados_manuales_ganan_sobre_api` | sincronizar no sobreescribe un feriado manual de la misma fecha |
| `feriados_error_no_borra_los_existentes` | |
| `backup_verifica_integridad` | un archivo corrupto es rechazado |
| `backup_conserva_los_tres_mas_recientes` | con 5 backups de 60 días, quedan 3 |
| `restore_rechaza_esquema_mas_nuevo` | |
| `import_json_es_atomico` | un fallo en la tabla 12 no deja nada modificado |
| `import_json_rechaza_tabla_desconocida` | |
| `import_json_rechaza_columna_desconocida` | |
| `import_json_verifica_integridad_referencial` | un archivo con una FK huérfana se revierte |
| `export_import_ida_y_vuelta` | exportar, importar en una base vacía y comparar conteos y sumas |
| `telefono_se_normaliza` | `(011) 4567-8901` produce `5401145678901` |
| `plantillas_no_tienen_formato_de_fecha` | ninguna cadena traducible contiene `dd/MM` |
