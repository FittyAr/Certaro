# 15 · Migración de datos desde el sistema anterior

Este documento describe el binario `eo-import-legacy`: una herramienta **de un solo uso** que lee la
base SQLite del sistema C# y escribe la base del sistema nuevo.

No es parte de la aplicación. No se distribuye con el instalador. Se ejecuta una vez, en la máquina
del usuario, y después se olvida.

Requisitos previos de lectura: [`03-modelo-de-datos.md`](./03-modelo-de-datos.md) (esquema destino),
[`04-dinero-fechas-y-tipos.md`](./04-dinero-fechas-y-tipos.md) (escala e instantes UTC).

---

## 1. Estrategia

### 1.1 Por qué un importador y no una migración incremental

La alternativa era escribir migraciones `sea-orm-migration` que transformaran la base vieja en su
lugar. Se descarta por tres motivos:

1. El esquema cambia de `PascalCase` a `snake_case` en **todas** las tablas y columnas. Renombrar
   columna por columna en SQLite implica recrear cada tabla de todos modos.
2. Hay tres tablas nuevas (`certificados`, `certificado_items`, `liquidacion_adelantos`) que se
   pueblan **derivando** datos de las tablas viejas, no copiándolos.
3. Una migración en el lugar es destructiva. Un importador deja la base vieja intacta, lo que
   permite reintentar tantas veces como haga falta.

### 1.2 Modelo de ejecución

```
base vieja (lectura, solo lectura)        base nueva (escritura)
electroobra_legacy.db          ──────►    electroobra.db
        │                                     ▲
        │                                     │
        └── eo-import-legacy ─────────────────┘
                   │
                   └──► import_report.json + import.log
```

Reglas duras:

- La base vieja se abre en **modo solo lectura** (`?mode=ro` en la cadena de conexión). El
  importador **no puede** modificarla ni siquiera por accidente.
- La base nueva se crea **desde cero** con las migraciones de `eo-migration` y **debe estar vacía**.
  Si tiene datos, el importador aborta.
- Todo el import ocurre dentro de **una transacción**. Si algo falla, no queda nada a medias.
- El importador es **idempotente por reejecución completa**: se borra la base nueva y se vuelve a
  correr. No existe un modo "continuar donde quedó".

### 1.3 Interfaz de línea de comandos

```
eo-import-legacy --source <ruta> --target <ruta> [opciones]

  --source <ruta>       Base SQLite del sistema anterior. Obligatorio.
  --target <ruta>       Base destino. Obligatorio. Se crea si no existe.
  --report <ruta>       Ruta del reporte JSON. Default: ./import_report.json
  --timezone <IANA>     Zona horaria de origen para convertir fechas locales.
                        Default: America/Argentina/Buenos_Aires
  --dry-run             Ejecuta todo y produce el reporte, pero hace rollback al final.
  --allow-orphans       Convierte referencias huérfanas en NULL o las descarta en lugar de abortar.
  --verbose             Log a nivel debug.
```

Códigos de salida:

| Código | Significado |
| --- | --- |
| `0` | Import exitoso, sin discrepancias |
| `1` | Import exitoso con advertencias (ver reporte) |
| `2` | Import abortado por error de validación previa |
| `3` | Import abortado por error durante la transferencia (rollback aplicado) |
| `4` | Import completado pero la verificación post-import falló (rollback aplicado) |

---

## 2. Fases

El importador ejecuta siete fases en orden estricto. Cada fase debe terminar antes de que empiece la
siguiente.

| # | Fase | Qué hace | Aborta si |
| --- | --- | --- | --- |
| 1 | Inspección del origen | Verifica que existan las tablas esperadas y lee sus conteos | falta una tabla, o `PRAGMA integrity_check` no devuelve `ok` |
| 2 | Detección de escala | Determina si los valores monetarios ya están escalados ×10 000 (§3.2) | la detección es ambigua |
| 3 | Preparación del destino | Corre las migraciones y verifica que las tablas estén vacías | el destino tiene filas |
| 4 | Transferencia | Copia tabla por tabla en orden de dependencias (§4) | violación de FK, dato inválido |
| 5 | Derivación | Puebla las 3 tablas nuevas a partir de los datos viejos (§5) | inconsistencia irreparable |
| 6 | Verificación | Compara conteos, sumas monetarias y hashes de muestras (§7) | cualquier diferencia |
| 7 | Reporte | Escribe `import_report.json` y hace commit (o rollback si `--dry-run`) | — |

---

## 3. Transformación de valores

### 3.1 Nombres

| Origen | Destino | Regla |
| --- | --- | --- |
| Tabla `AsistenciasEmpleado` | `asistencias_empleado` | `PascalCase` → `snake_case` |
| Columna `TipoMovimientoId` | `tipo_movimiento_id` | ídem |
| Columna `AjusteUocraPorcentaje` | `ajuste_uocra_porcentaje` | ídem, siglas en minúscula |

El mapeo no se calcula con una función de conversión genérica: está **escrito explícitamente**
columna por columna en §4. Una función genérica se equivoca con siglas (`Uocra`, `Iva`, `Dni`,
`Cuit`, `Mime`) y el error pasa desapercibido.

### 3.2 Importes: la detección de escala

Este es el punto más delicado del import.

El sistema anterior pasó por dos estados:

- **Antes** de la migración `20260828214627_RescaleMonetaryValues`: las columnas monetarias eran
  `INTEGER` con el valor **redondeado a entero** (un importe de `1234.56` estaba guardado como
  `1235`). El converter viejo multiplicaba por 1.
- **Después** de esa migración: las columnas están escaladas ×10 000 (`1234.56` → `12345600`).

Como el destino usa la misma escala ×10 000 que el estado "después", el importador tiene que saber en
qué estado está la base que le dieron. La detección es así:

```rust
// Fase 2. Consulta la tabla de migraciones aplicadas del sistema anterior.
let applied: Vec<String> = read_all(
    "SELECT MigrationId FROM __EFMigrationsHistory ORDER BY MigrationId"
)?;

let scale_state = if applied.iter().any(|m| m.contains("RescaleMonetaryValues")) {
    ScaleState::AlreadyScaled     // multiplicar por 1
} else {
    ScaleState::UnscaledIntegers  // multiplicar por 10_000
};
```

Si la tabla `__EFMigrationsHistory` no existe o está vacía, la detección cae a una heurística de
respaldo y **exige confirmación explícita** con `--assume-scaled` o `--assume-unscaled`; nunca
adivina en silencio.

| Estado detectado | Factor aplicado | Precisión resultante |
| --- | --- | --- |
| `AlreadyScaled` | `valor * 1` | exacta, 4 decimales conservados |
| `UnscaledIntegers` | `valor * 10_000` | los decimales ya se habían perdido antes del import; se recupera la escala, no la precisión |

En el segundo caso el reporte incluye una advertencia explícita: **los centavos originales no son
recuperables**, porque el sistema anterior los redondeó al escribir. El importador no inventa
decimales.

### 3.3 La excepción de `PagosFactura.Monto`

**[BUG-LEGADO CRÍTICO]** La migración `RescaleMonetaryValues` del sistema anterior reescaló las
columnas listadas en `MonetaryColumnRegistry`, y **`PagosFactura.Monto` no está en ese registro**.
El converter de EF Core sí se le aplicaba por reflexión, así que la aplicación escribía valores
escaladas ×10 000, pero la migración de reescalado no tocó las filas anteriores.

Consecuencia: en una base que pasó por la migración, `PagosFactura.Monto` puede contener **una mezcla
de las dos escalas** según cuándo se creó cada fila.

Tratamiento obligatorio:

```
Para cada fila de PagosFactura:
    monto_pago    = valor crudo de la columna
    total_factura = total de la factura asociada, ya normalizado a escala 10.000

    si scale_state == AlreadyScaled:
        si monto_pago <= total_factura * 2:
            → la fila ya está escalada, factor 1
        sino si monto_pago >= total_factura * 5000:
            → improbable, marcar para revisión manual
        sino:
            → la fila está sin escalar, factor 10.000
```

La heurística compara contra el total de la factura porque un pago sin escalar es ~10 000 veces menor
que uno escalado, y esa diferencia de cuatro órdenes de magnitud es inequívoca frente a la variación
real de un pago parcial.

**Toda** fila cuya escala se decidió por heurística se lista en el reporte, con id, monto crudo,
monto resultante y total de la factura. El usuario tiene que revisar esa lista. Si la lista tiene más
de `0` filas, el código de salida es `1`, no `0`.

### 3.4 Porcentajes, cantidades y multiplicadores

Las columnas escaladas **no** son sólo importes. La lista completa está en
[`04-dinero-fechas-y-tipos.md`](./04-dinero-fechas-y-tipos.md) §2: 34 columnas, de las cuales 12 son
`Decimal4` (porcentajes, cantidades, multiplicadores, días trabajados) y el resto `Money`.

El importador aplica el mismo factor a todas. La distinción `Money` / `Decimal4` importa en el
dominio, no en el import.

Tres columnas tienen un default que el importador debe respetar si encuentra `0`:

| Columna | Valor crudo `0` significa | Qué escribe el importador |
| --- | --- | --- |
| `movimientos.cantidad` | fila creada antes de que existiera la columna | `10000` (= `1.0`) |
| `liquidaciones.multiplicador_sabado` | ídem | `10000` (= `1.0`) |
| `liquidaciones.multiplicador_domingo` | ídem | `10000` (= `1.0`) |
| `liquidaciones.multiplicador_feriado` | ídem | `10000` (= `1.0`) |

Un multiplicador `0` haría que las horas de sábado valgan cero, lo que cambia silenciosamente el
resultado de recalcular una liquidación vieja. Un `cantidad = 0` haría que `total = monto * 0 = 0`.

### 3.5 Fechas: de local mezclado a UTC

El sistema anterior guardaba dos clases de fecha en el mismo formato de texto, sin marca de zona
(ver [`04-dinero-fechas-y-tipos.md`](./04-dinero-fechas-y-tipos.md) §3.2):

| Clase | Columnas | Qué hay guardado | Conversión |
| --- | --- | --- | --- |
| Auditoría | `CreatedAt`, `UpdatedAt`, `DeletedAt`, `AppMetadata.UpdatedAt` | ya es UTC (`DateTime.UtcNow`) | se copia tal cual, sólo se normaliza el formato |
| Negocio con hora | `Movimientos.Fecha` | hora **local** de la máquina | se interpreta en `--timezone` y se convierte a UTC |
| Negocio civil | las 15 columnas de fecha civil | hora local, con hora arbitraria | se toma la **parte de fecha** y se escribe medianoche UTC |

Implementación:

```rust
/// Auditoría: el texto ya representa UTC.
fn audit(raw: &str) -> Result<DateTime<Utc>> {
    let naive = parse_legacy_text(raw)?;         // acepta con y sin fracción de segundo
    Ok(Utc.from_utc_datetime(&naive))
}

/// Negocio con hora: el texto representa hora local del origen.
fn business_instant(raw: &str, tz: Tz) -> Result<DateTime<Utc>> {
    let naive = parse_legacy_text(raw)?;
    match tz.from_local_datetime(&naive) {
        LocalResult::Single(dt)      => Ok(dt.with_timezone(&Utc)),
        LocalResult::Ambiguous(a, _) => { warn_ambiguous(raw); Ok(a.with_timezone(&Utc)) }
        LocalResult::None            => { warn_nonexistent(raw); Ok(shift_forward_one_hour(naive, tz)) }
    }
}

/// Negocio civil: sólo importa el día. NO se convierte de zona.
fn business_civil(raw: &str) -> Result<DateTime<Utc>> {
    let naive = parse_legacy_text(raw)?;
    Ok(civil_to_utc(naive.date()))               // medianoche UTC del mismo día civil
}
```

La decisión crítica está en `business_civil`: **no** se convierte de zona. Si una asistencia estaba
guardada como `2026-03-15 00:00:00` en hora local, convertirla a UTC daría `2026-03-15T03:00:00Z`, y
si estaba guardada como `2026-03-15 22:30:00` daría `2026-03-16T01:30:00Z`, o sea **otro día**. Para
una fecha civil el día es el dato; la hora es ruido. Se descarta la hora y se conserva el día tal
como el usuario lo vio en pantalla.

Formatos de texto que `parse_legacy_text` debe aceptar, porque los tres aparecen en bases reales
según la versión de EF Core que escribió la fila:

```
2026-03-15 22:30:00
2026-03-15 22:30:00.1234567
2026-03-15T22:30:00.123Z
```

### 3.6 GUID → UUID

Los identificadores del sistema anterior son `Guid` de .NET serializados como texto en minúsculas con
guiones: `0f8fad5b-d9cb-469f-a165-70867728950e`.

- Se copian **literalmente**, sin regenerar. Cambiar los ids rompería los adjuntos, que referencian
  la entidad por id en su ruta de archivo (doc 13 §1.2).
- El sistema nuevo usa UUID v7 para filas nuevas (doc 04 §6), pero **acepta** los v4 importados. No
  hay validación de versión de UUID en el esquema, a propósito.
- Los 4 GUID de sistema de `TiposMovimiento` (`…0001` a `…0004`) ya existen en el destino por el seed
  de las migraciones. El importador **no** los inserta: los reconoce y los omite, verificando que
  `nombre` y `es_ingreso` coincidan. Si no coinciden, aborta: significa que el usuario editó un tipo
  de sistema y hay que decidir a mano.

### 3.7 Booleanos, enums y `row_version`

| Origen | Destino |
| --- | --- |
| `INTEGER` 0/1 de un `bool` | `INTEGER` 0/1, sin cambios |
| `INTEGER` de un enum | `INTEGER`, sin cambios; los valores numéricos de los 8 enums se conservan (doc 05 §3) |
| `BLOB` de 8 bytes de `RowVersion` | se copia tal cual; si es `NULL` o de otro largo, se escribe `X'0000000000000001'` |

Los valores de enum no se remapean. Ese es el motivo por el cual `EstadoFactura::PagadaParcial`
recibió el valor `5` y no se insertó en el medio de la secuencia (doc 05 §3.2).

### 3.8 Recorte de texto

El esquema nuevo tiene límites de longitud distintos en algunas columnas (doc 07 §4 documenta los
casos). El importador **no recorta**: si un texto excede el límite del destino, lo registra en el
reporte y aborta. Recortar en silencio es pérdida de datos.

Excepción única: los espacios en blanco al principio y al final se eliminan de toda columna de texto,
y una cadena que queda vacía después del recorte se convierte en `NULL` si la columna es nullable.

---

## 4. Mapeo tabla por tabla

Orden de inserción, dictado por las claves foráneas. Este orden es obligatorio.

```
 1. tipos_movimiento      (solo verificación del seed; nada que insertar salvo tipos de usuario)
 2. tipos_concepto_pago
 3. categorias
 4. clientes
 5. cliente_contactos
 6. obras
 7. trabajos
 8. ordenes_trabajo
 9. orden_trabajo_items
10. facturas
11. pagos_factura
12. empleados
13. asistencias_empleado
14. liquidaciones
15. movimientos
16. adjuntos
17. app_metadata
18. feriados             (derivado de la configuración vieja, §5.4)
19. certificados         (derivado, §5.1)
20. certificado_items    (derivado, §5.1)
21. liquidacion_adelantos(derivado, §5.2)
```

Convención de las tablas que siguen: la columna **Factor** indica qué transformación se aplica.

| Factor | Significado |
| --- | --- |
| `=` | copia literal |
| `×S` | valor escalado: multiplicar por el factor de escala detectado en §3.2 |
| `UTC-A` | fecha de auditoría, §3.5 clase "Auditoría" |
| `UTC-I` | instante de negocio, §3.5 clase "Negocio con hora" |
| `UTC-C` | fecha civil, §3.5 clase "Negocio civil" |
| `→` | valor calculado o constante; se explica en la nota |

### 4.1 `TiposMovimiento` → `tipos_movimiento`

| Origen | Destino | Factor | Nota |
| --- | --- | --- | --- |
| `Id` | `id` | `=` | |
| `Nombre` | `nombre` | `=` | |
| `Descripcion` | `descripcion` | `=` | |
| `EsIngreso` | `es_ingreso` | `=` | |
| `EsSistema` | `es_sistema` | `=` | |
| `CreatedAt` | `created_at` | `UTC-A` | |
| `UpdatedAt` | `updated_at` | `UTC-A` | |
| `RowVersion` | `row_version` | `=` | |
| `IsDeleted` | `is_deleted` | `=` | |
| `DeletedAt` | `deleted_at` | `UTC-A` | |

Las 4 filas con `es_sistema = 1` se omiten (§3.6). Las filas creadas por el usuario se insertan
normalmente.

### 4.2 `TiposConceptoPago` → `tipos_concepto_pago`

Mismas columnas que §4.1 menos `descripcion` y `es_ingreso`.

| Origen | Destino | Factor |
| --- | --- | --- |
| `Id` | `id` | `=` |
| `Nombre` | `nombre` | `=` |
| `EsSistema` | `es_sistema` | `=` |
| `CreatedAt` / `UpdatedAt` / `DeletedAt` | ídem | `UTC-A` |
| `RowVersion` / `IsDeleted` | ídem | `=` |

**[NUEVO]** El destino tiene filas de semilla para esta tabla (doc 03 §5.2) que el origen no tenía.
El importador inserta las filas del origen **primero** y luego el seed hace `INSERT OR IGNORE`, así
que un concepto que el usuario ya había creado con el mismo nombre no se duplica. Si el nombre
coincide pero el id difiere, gana el del usuario y el seed no inserta; el reporte lo anota.

### 4.3 `Categorias` → `categorias`

| Origen | Destino | Factor | Nota |
| --- | --- | --- | --- |
| `Id` | `id` | `=` | |
| `Nombre` | `nombre` | `=` | |
| `Descripcion` | `descripcion` | `=` | |
| `ColorHex` | `color_hex` | `=` | se valida contra `^#[0-9A-Fa-f]{6}$`; si no coincide, `NULL` y advertencia |
| `Icono` | `icono` | `=` | |
| — | `categoria_padre_id` | `→ NULL` | **[NUEVO]** la jerarquía no existía; todas quedan como raíz |
| auditoría | ídem | `UTC-A` / `=` | |

### 4.4 `Clientes` → `clientes`

| Origen | Destino | Factor | Nota |
| --- | --- | --- | --- |
| `Id` | `id` | `=` | |
| `Nombre` | `nombre` | `=` | |
| `Cuit` | `cuit` | `=` | se normaliza quitando guiones y puntos; se **conserva** aunque no valide el dígito verificador |
| `Email` | — | `→` | ver §5.3: migra a `cliente_contactos` |
| `Telefono` | `telefono` | `=` | |
| `Direccion` | `direccion` | `=` | |
| `CondicionIva` | `condicion_iva` | `=` | |
| auditoría | ídem | `UTC-A` / `=` | |

### 4.5 `ClienteContactos` → `cliente_contactos`

| Origen | Destino | Factor | Nota |
| --- | --- | --- | --- |
| `Id` | `id` | `=` | |
| `ClienteId` | `cliente_id` | `=` | |
| `Email` | `email` | `=` | |
| `Etiqueta` | `etiqueta` | `=` | |
| — | `nombre` | `→ NULL` | **[NUEVO]** columna nueva |
| — | `telefono` | `→ NULL` | **[NUEVO]** columna nueva |
| — | `es_principal` | `→` | ver §5.3 |
| auditoría | ídem | `UTC-A` / `=` | |

### 4.6 `Obras` → `obras`

| Origen | Destino | Factor | Nota |
| --- | --- | --- | --- |
| `Id` | `id` | `=` | |
| `Numero` | `numero` | `=` | entero real, **no** escalado |
| `Nombre` | `nombre` | `=` | |
| `Direccion` | `direccion` | `=` | |
| `Localidad` | `localidad` | `=` | |
| `ClienteId` | `cliente_id` | `=` | |
| `Estado` | `estado` | `=` | `EstadoObra`, valores 0-3 sin cambios |
| auditoría | ídem | `UTC-A` / `=` | |

El índice único de `numero` **no** está filtrado por `is_deleted` (doc 03 §3.6). Si el origen tiene
dos obras con el mismo número —una borrada y una viva— el insert falla. Se marca en el reporte y
aborta: hay que decidir a mano qué número conserva cuál.

### 4.7 `Trabajos` → `trabajos`

| Origen | Destino | Factor | Nota |
| --- | --- | --- | --- |
| `Id` | `id` | `=` | |
| `ObraId` | `obra_id` | `=` | |
| `Descripcion` | `descripcion` | `=` | |
| `Presupuesto` | `presupuesto` | `×S` | |
| `FechaInicio` | `fecha_inicio` | `UTC-C` | |
| `FechaFin` | `fecha_fin` | `UTC-C` | nullable |
| `Estado` | `estado` | `=` | `EstadoTrabajo`, valores sin cambios |
| auditoría | ídem | `UTC-A` / `=` | |

### 4.8 `OrdenesTrabajo` → `ordenes_trabajo`

| Origen | Destino | Factor | Nota |
| --- | --- | --- | --- |
| `Id` | `id` | `=` | |
| `TrabajoId` | `trabajo_id` | `=` | |
| `Titulo` | `titulo` | `=` | |
| `Fecha` | `fecha` | `UTC-C` | |
| `NumeroCertificado` | `numero_certificado` | `=` | queda como texto libre; ver §5.1 |
| `AjusteUocraPorcentaje` | `ajuste_uocra_porcentaje` | `×S` | es un **porcentaje**, no un monto |
| `OtrosDescuentos` | `otros_descuentos` | `×S` | es un **monto** |
| `Observaciones` | `observaciones` | `=` | |
| auditoría | ídem | `UTC-A` / `=` | |

### 4.9 `OrdenTrabajoItems` → `orden_trabajo_items`

| Origen | Destino | Factor | Nota |
| --- | --- | --- | --- |
| `Id` | `id` | `=` | |
| `OrdenTrabajoId` | `orden_trabajo_id` | `=` | |
| `Descripcion` | `descripcion` | `=` | |
| `Unidad` | `unidad` | `=` | |
| `Cantidad` | `cantidad` | `×S` | |
| `PrecioUnitario` | `precio_unitario` | `×S` | |
| `PorcentajeAnterior` | `porcentaje_anterior` | `×S` | |
| `PorcentajeActual` | `porcentaje_actual` | `×S` | |
| `Ejecutado` | `ejecutado` | `=` | |
| `Nota` | `nota` | `=` | |
| — | `orden` | `→` | **[NUEVO]** se asigna `ROW_NUMBER()` dentro de cada `orden_trabajo_id`, ordenando por `CreatedAt`, `Id` |
| auditoría | ídem | `UTC-A` / `=` | |

Validación en el import: si `porcentaje_anterior + porcentaje_actual > 100`, se emite advertencia y se
**conserva** el valor. No se corrige. El sistema anterior no validaba esto (doc 07 §5.3) y hay datos
reales que lo violan; corregirlos automáticamente falsearía el histórico de certificación.

### 4.10 `Facturas` → `facturas`

| Origen | Destino | Factor | Nota |
| --- | --- | --- | --- |
| `Id` | `id` | `=` | |
| `Numero` | `numero` | `=` | |
| `ClienteId` | `cliente_id` | `=` | |
| `Fecha` | `fecha` | `UTC-C` | |
| — | `fecha_vencimiento` | `→` | **[NUEVO]** `fecha + Business.DiasVencimientoFactura` días; ver nota |
| `Subtotal` | `subtotal` | `×S` | |
| `Iva` | `iva` | `×S` | es el **monto** de IVA, no la tasa |
| `Total` | `total` | `×S` | se copia; §7.3 verifica que sea igual a `subtotal + iva` |
| `Estado` | `estado` | `=` | `EstadoFactura`, valores 0-4 sin cambios |
| `Observaciones` | `observaciones` | `=` | |
| auditoría | ídem | `UTC-A` / `=` | |

Sobre `fecha_vencimiento`: la columna es nueva y el dato no existe en el origen. Se calcula con el
default de configuración (doc 14 §2.3), no se deja `NULL`, porque el cálculo de antigüedad de deuda
(doc 06 §4.6) y el KPI de facturas vencidas dependen de ella. El reporte anota cuántas facturas
recibieron un vencimiento **estimado**, y la interfaz muestra el aviso correspondiente la primera vez
que se abre el módulo de facturas después del import.

Ninguna factura recibe `estado = 5` (`PagadaParcial`) durante el import. La reclasificación de
estados según los pagos registrados se hace en la fase 6 de derivación, descrita en §5.5.

### 4.11 `PagosFactura` → `pagos_factura`

| Origen | Destino | Factor | Nota |
| --- | --- | --- | --- |
| `Id` | `id` | `=` | |
| `FacturaId` | `factura_id` | `=` | |
| `Fecha` | `fecha` | `UTC-C` | |
| `Monto` | `monto` | **especial** | §3.3: escala decidida fila por fila |
| `MedioPago` | `medio_pago` | `=` | |
| auditoría | ídem | `UTC-A` / `=` | |

### 4.12 `Empleados` → `empleados`

| Origen | Destino | Factor | Nota |
| --- | --- | --- | --- |
| `Id` | `id` | `=` | |
| `Nombre` | `nombre` | `=` | |
| `Dni` | `dni` | `=` | |
| `Telefono` | `telefono` | `=` | |
| `Email` | `email` | `=` | |
| `Cargo` | `cargo` | `=` | |
| `FechaIngreso` | `fecha_ingreso` | `UTC-C` | |
| `SueldoBase` | `sueldo_base` | `×S` | |
| `TarifaDiaria` | `tarifa_diaria` | `×S` | |
| `PagoFrecuencia` | `pago_frecuencia` | `=` | `PaymentFrequency`, valores sin cambios |
| `Activo` | `activo` | `=` | |
| auditoría | ídem | `UTC-A` / `=` | |

### 4.13 `AsistenciasEmpleado` → `asistencias_empleado`

| Origen | Destino | Factor | Nota |
| --- | --- | --- | --- |
| `Id` | `id` | `=` | |
| `EmpleadoId` | `empleado_id` | `=` | |
| `TrabajoId` | `trabajo_id` | `=` | nullable |
| `Fecha` | `fecha` | `UTC-C` | **crítico**: la normalización a medianoche es lo que hace funcionar el índice único |
| `TipoJornada` | `tipo_jornada` | `=` | valores sin cambios |
| `Observaciones` | `observaciones` | `=` | |
| auditoría | ídem | `UTC-A` / `=` | |

El origen ya tiene índice único `(EmpleadoId, Fecha)`, pero sobre un `Fecha` que **incluye hora**.
Después de normalizar a medianoche pueden aparecer colisiones que en el origen no existían: dos
asistencias del mismo empleado el mismo día con distinta hora.

Resolución determinista, sin intervención del usuario:

```
Agrupar por (empleado_id, día civil).
Si el grupo tiene más de una fila:
    conservar la de CreatedAt más reciente (desempate por Id ascendente)
    las demás se insertan con is_deleted = 1 y deleted_at = created_at
    registrar cada colisión en el reporte
```

Se conserva la más reciente porque el ciclo de click de la interfaz vieja hacía upsert, así que la
última carga es la intención vigente del usuario. Las descartadas quedan borradas lógicamente, no
eliminadas: el dato sigue ahí si hace falta auditarlo.

### 4.14 `Liquidaciones` → `liquidaciones`

| Origen | Destino | Factor | Nota |
| --- | --- | --- | --- |
| `Id` | `id` | `=` | |
| `EmpleadoId` | `empleado_id` | `=` | |
| `FechaInicio` | `fecha_inicio` | `UTC-C` | |
| `FechaFin` | `fecha_fin` | `UTC-C` | |
| `DiasTrabajados` | `dias_trabajados` | `×S` | `Decimal4`, admite medias jornadas |
| `TarifaAplicada` | `tarifa_aplicada` | `×S` | |
| `IncluirSabados` | `incluir_sabados` | `=` | |
| `IncluirDomingos` | `incluir_domingos` | `=` | |
| `IncluirFeriados` | `incluir_feriados` | `=` | |
| `MultiplicadorSabado` | `multiplicador_sabado` | `×S` | `0` → `10000`, §3.4 |
| `MultiplicadorDomingo` | `multiplicador_domingo` | `×S` | `0` → `10000`, §3.4 |
| `MultiplicadorFeriado` | `multiplicador_feriado` | `×S` | `0` → `10000`, §3.4 |
| `TotalBruto` | `total_bruto` | `×S` | |
| `TotalAdelantos` | `total_adelantos` | `×S` | se copia; §5.2 verifica contra el detalle derivado |
| `Observaciones` | `observaciones` | `=` | |
| auditoría | ídem | `UTC-A` / `=` | |

### 4.15 `Movimientos` → `movimientos`

La tabla con más volumen y más columnas nullable.

| Origen | Destino | Factor | Nota |
| --- | --- | --- | --- |
| `Id` | `id` | `=` | |
| `Fecha` | `fecha` | `UTC-I` | **única** columna de negocio que conserva la hora |
| `Concepto` | `concepto` | `=` | max 500 en ambos lados |
| `Monto` | `monto` | `×S` | |
| `Cantidad` | `cantidad` | `×S` | `0` → `10000`, §3.4 |
| `Moneda` | `moneda` | `=` | valores sin cambios |
| `CotizacionAplicada` | `cotizacion_aplicada` | `×S` | nullable |
| `TipoMovimientoId` | `tipo_movimiento_id` | `=` | obligatorio |
| `TipoConceptoPagoId` | `tipo_concepto_pago_id` | `=` | nullable |
| `CategoriaId` | `categoria_id` | `=` | nullable |
| `ClienteId` | `cliente_id` | `=` | nullable |
| `EmpleadoId` | `empleado_id` | `=` | nullable |
| `TrabajoId` | `trabajo_id` | `=` | nullable |
| `FacturaId` | `factura_id` | `=` | nullable |
| auditoría | ídem | `UTC-A` / `=` | |

**[FIX]** En el origen, las FK `ClienteId`, `EmpleadoId` y `TrabajoId` de `Movimientos` **no tenían
`ON DELETE` declarado** (EF Core resolvía la integridad en memoria, no en la base). Con
`PRAGMA foreign_keys` desactivado —que era el caso durante el import JSON del sistema anterior— es
posible que existan movimientos apuntando a filas que ya no están.

Tratamiento:

| Situación | Sin `--allow-orphans` | Con `--allow-orphans` |
| --- | --- | --- |
| FK nullable huérfana (`cliente_id`, `empleado_id`, `trabajo_id`, `factura_id`, `categoria_id`, `tipo_concepto_pago_id`) | aborta y lista los ids | escribe `NULL` y lo registra |
| FK obligatoria huérfana (`tipo_movimiento_id`) | aborta | aborta igual; no hay valor de reemplazo aceptable |

La FK obligatoria nunca se "arregla": un movimiento sin tipo no se puede clasificar como ingreso o
gasto, con lo cual falsearía todos los totales. Si aparece, el usuario tiene que corregirlo en el
sistema viejo antes de reintentar.

### 4.16 `Adjuntos` → `adjuntos`

| Origen | Destino | Factor | Nota |
| --- | --- | --- | --- |
| `Id` | `id` | `=` | |
| `EntidadTipo` | `entidad_tipo` | `=` | se valida contra el enum controlado; un valor desconocido aborta |
| `EntidadId` | `entidad_id` | `=` | |
| `NombreArchivo` | `nombre_archivo` | `=` | |
| `RutaRelativa` | `ruta_relativa` | `=` | |
| `Mime` | `mime` | `=` | |
| `Tamano` | `tamano` | `=` | **bytes**, entero real, no escalado |
| auditoría | ídem | `UTC-A` / `=` | |

Los **archivos** son responsabilidad aparte. El importador:

1. Copia el árbol de `attachments/` del directorio de datos viejo al nuevo, preservando las rutas
   relativas.
2. Para cada fila, verifica que el archivo exista en el destino y que su tamaño coincida con
   `tamano`.
3. Las filas cuyo archivo falta se insertan igual, con una advertencia en el reporte. Borrar la fila
   perdería la referencia a un archivo que el usuario podría recuperar de un backup.
4. Los archivos presentes en el disco sin fila correspondiente se listan como huérfanos. No se
   borran.

**[NUEVO]** El destino tiene un límite de tamaño y una whitelist de MIME (doc 13 §1.3 y §1.4) que el origen
no tenía. Esas reglas se aplican a los adjuntos **nuevos**, no a los importados. Un adjunto viejo que
excede el límite se importa y se marca en el reporte.

### 4.17 `AppMetadata` → `app_metadata`

| Origen | Destino | Factor |
| --- | --- | --- |
| `Key` | `key` | `=` |
| `Value` | `value` | `=` |
| `UpdatedAt` | `updated_at` | `UTC-A` |

### 4.18 `SchemaVersions` y `__EFMigrationsHistory`: no se importan

Ambas son metadatos del mecanismo de migración de EF Core y no tienen equivalente. El destino lleva
su propio registro en la tabla de `sea-orm-migration`.

El importador **lee** `__EFMigrationsHistory` en la fase 2 (§3.2) y copia su contenido al reporte,
como constancia de qué versión del esquema viejo se importó.

---

## 5. Derivación de datos nuevos

### 5.1 `certificados` y `certificado_items`

El sistema anterior no guardaba historial de certificación: la orden de trabajo tenía un único
`NumeroCertificado` de texto libre y los porcentajes acumulados vivían en los ítems, sobreescritos en
cada certificación. Todo certificado anterior al último es irrecuperable (RC-10, doc 01).

Lo que el importador puede reconstruir es **un** certificado por orden, el vigente:

```
Para cada orden_trabajo con al menos un item con porcentaje_actual > 0:

    numero = 1
    fecha  = orden_trabajo.fecha
    observaciones = "Importado del sistema anterior. Certificado único reconstruido."   ← i18n
    (clave: Import.Certificado.Reconstruido)

    Para cada item de la orden:
        insertar certificado_item con:
            cantidad             = item.cantidad
            precio_unitario      = item.precio_unitario
            porcentaje_anterior  = item.porcentaje_anterior
            porcentaje_actual    = item.porcentaje_actual
            subtotal_actual      = cantidad × precio_unitario × (porcentaje_actual / 100)
            subtotal_acumulado   = cantidad × precio_unitario
                                     × ((porcentaje_anterior + porcentaje_actual) / 100)

    total_certificado = Σ subtotal_actual de los items
    ajuste_uocra      = total_certificado × (orden.ajuste_uocra_porcentaje / 100)
    otros_descuentos  = orden.otros_descuentos
    total_neto        = total_certificado − ajuste_uocra − otros_descuentos
```

Las fórmulas son literalmente las de [`06-casos-de-uso-y-formulas.md`](./06-casos-de-uso-y-formulas.md)
§5, incluida la regla de redondeo de la multiplicación (doc 04 §1.5). El importador **no** tiene su
propia aritmética: llama a las mismas funciones del dominio. Un test verifica que un certificado
derivado por el importador y uno creado por la aplicación con los mismos datos den bit a bit el mismo
resultado.

`NumeroCertificado` del origen es texto libre (`"C-004"`, `"4"`, `"cuarto"`) y `certificados.numero`
es entero. No se intenta parsear: el certificado reconstruido siempre recibe `numero = 1` y el texto
original se preserva en `ordenes_trabajo.numero_certificado`, que sigue existiendo en el esquema
nuevo. Si el texto era `"C-004"`, el usuario sabe que en realidad hubo cuatro y que sólo el cuarto
sobrevivió.

### 5.2 `liquidacion_adelantos`

El origen guardaba únicamente `TotalAdelantos` y recalculaba el detalle consultando movimientos, lo
que permitía descontar el mismo adelanto en dos liquidaciones con períodos solapados (INV-05).

Derivación:

```
Para cada liquidacion, ordenadas por fecha_inicio ascendente, luego created_at ascendente:

    candidatos = movimientos donde
        tipo_movimiento_id = '00000000-0000-0000-0000-000000000003'   (Adelanto)
        empleado_id        = liquidacion.empleado_id
        fecha              ∈ [liquidacion.fecha_inicio, liquidacion.fecha_fin]
        is_deleted         = 0
        y el movimiento NO fue ya vinculado a una liquidación anterior

    Para cada candidato:
        insertar liquidacion_adelantos con monto, fecha y concepto congelados del movimiento

    suma_derivada = Σ monto de los candidatos
```

El orden de recorrido es lo que resuelve los solapamientos: la liquidación **más antigua** se queda
con el adelanto. Es la interpretación correcta, porque en la práctica el adelanto se descontó cuando
se pagó la primera liquidación del período.

Después de derivar, para cada liquidación:

| Comparación | Acción |
| --- | --- |
| `suma_derivada == total_adelantos` | nada; caso normal |
| `suma_derivada < total_adelantos` | el faltante corresponde a un adelanto ya consumido por una liquidación anterior. Se **conserva** `total_adelantos` del origen y se registra la diferencia en el reporte |
| `suma_derivada > total_adelantos` | hay adelantos en el rango que el origen no había contado. Se **conserva** `total_adelantos` del origen y se registra |

En los dos casos de diferencia gana el valor del origen, nunca el derivado: `total_bruto −
total_adelantos` es el importe que el empleado **efectivamente recibió**, y ese número es histórico.
Recalcularlo cambiaría un pago ya hecho.

Consecuencia: el test de consistencia de doc 03 §3.20 (`SUM(monto)` de `liquidacion_adelantos` igual
a `liquidaciones.total_adelantos`) **no** aplica a las filas importadas. Se implementa con una
excepción explícita para las liquidaciones cuyo `created_at` es anterior a la marca de import
guardada en `app_metadata` bajo la clave `import.completed_at`.

### 5.3 `Clientes.Email` → `cliente_contactos`

RC-13 pide N emails por cliente. El origen tenía una columna `Email` en `Clientes` **y** una tabla
`ClienteContactos`, o sea los dos modelos a la vez, sin regla de cuál gana.

Derivación, en este orden:

```
1. Importar todas las filas de ClienteContactos con es_principal = 0.

2. Para cada cliente cuyo Email no sea NULL ni vacío:
     si ya existe un contacto de ese cliente con el mismo email (comparación case-insensitive):
         marcar ese contacto existente como es_principal = 1
     sino:
         insertar un contacto nuevo con:
             id           = UUID v7 generado
             email        = Clientes.Email
             etiqueta     = clave i18n Import.Contacto.EtiquetaPrincipal
             es_principal = 1
             created_at   = Clientes.CreatedAt

3. Para cada cliente que quedó sin ningún contacto con es_principal = 1
   pero tiene al menos un contacto:
         marcar como principal el de created_at más antiguo (desempate por id).
```

El paso 3 garantiza la invariante "a lo sumo uno, y al menos uno si hay contactos" que el caso de uso
mantiene de ahí en adelante (doc 03 §3.5).

La columna `Email` de `clientes` **no** existe en el esquema nuevo. Después del import el dato vive
únicamente en `cliente_contactos`.

### 5.4 `feriados`

El origen guardaba los feriados en el archivo de configuración, con dos serializaciones incompatibles
(doc 13 §3.4). El importador:

1. Lee el `appsettings.json` del directorio de datos viejo, si existe.
2. Intenta las **dos** formas de deserialización conocidas y toma la que produzca filas.
3. Inserta cada feriado con `origen = 'Manual'`, porque no hay manera de saber si vino de la API.
4. Los feriados que no se puedan parsear se listan en el reporte con su texto crudo.

Los feriados de la API se recuperan solos en el primer arranque. Esta derivación existe únicamente
para no perder los que el usuario cargó a mano.

### 5.5 Reclasificación de estados de factura

`EstadoFactura::PagadaParcial` es nuevo (doc 05 §3.2) y ninguna factura del origen lo tiene. La
reclasificación corre al final de la fase 5, cuando ya están importadas las facturas y sus pagos:

```
Para cada factura con estado ∈ {Emitida, Vencida}:
    total_pagado = Σ pagos_factura.monto de esa factura (is_deleted = 0)

    si total_pagado >= factura.total:
        estado = Pagada
    sino si total_pagado > 0:
        estado = PagadaParcial
    sino si fecha_vencimiento < hoy:
        estado = Vencida
    sino:
        estado sin cambios
```

Las facturas en `Borrador`, `Pagada` o `Anulada` no se tocan: son estados terminales o previos a la
emisión, y el usuario los puso a mano.

**[FIX]** Esto corrige el comportamiento del sistema anterior, donde registrar un pago **no**
cambiaba el estado de la factura (doc 08 §2). Después del import los estados reflejan los pagos
reales por primera vez. El reporte lista cuántas facturas cambiaron de estado y a cuál, porque para
el usuario esos números van a parecer distintos de lo que veía ayer.

### 5.6 Marca de import

Al final, en `app_metadata`:

| Clave | Valor |
| --- | --- |
| `import.completed_at` | instante UTC del commit |
| `import.source_path` | ruta absoluta de la base de origen |
| `import.source_schema` | último `MigrationId` de `__EFMigrationsHistory` |
| `import.scale_state` | `AlreadyScaled` \| `UnscaledIntegers` |
| `import.tool_version` | versión del binario `eo-import-legacy` |
| `import.warning_count` | cantidad de advertencias del reporte |

La aplicación lee `import.completed_at` en el arranque. Si existe y `import.warning_count > 0`,
muestra un aviso no bloqueante con un enlace al reporte. Se descarta con un click y no vuelve.

---

## 6. Orden de operaciones y transacción

```rust
pub async fn run(opts: ImportOptions) -> Result<ImportReport, ImportError> {
    let legacy = open_readonly(&opts.source)?;          // fase 1
    let inventory = inspect_source(&legacy).await?;
    let scale = detect_scale(&legacy).await?;           // fase 2

    let target = prepare_target(&opts.target).await?;   // fase 3: migraciones + verificar vacío
    let tx = target.begin().await?;

    tx.execute_unprepared("PRAGMA foreign_keys = ON;").await?;

    let mut report = ImportReport::new(inventory, scale);

    transfer_all(&tx, &legacy, scale, &mut report).await?;   // fase 4, orden de §4
    derive_all(&tx, &mut report).await?;                     // fase 5
    verify(&tx, &legacy, scale, &mut report).await?;         // fase 6

    if report.has_blocking_issues() {
        tx.rollback().await?;
        return Err(ImportError::VerificationFailed(report));
    }

    if opts.dry_run {
        tx.rollback().await?;
    } else {
        tx.commit().await?;
        copy_attachment_files(&opts).await?;             // fuera de la transacción, ver nota
    }

    write_report(&report, &opts.report_path)?;           // fase 7
    Ok(report)
}
```

`PRAGMA foreign_keys = ON` durante todo el import es deliberado, y es lo contrario de lo que hacía el
importador JSON del sistema anterior, que las desactivaba para poder insertar en cualquier orden. Las
restricciones activas son la red de seguridad que detecta las referencias huérfanas descritas en
§4.15. Por eso el orden de inserción de §4 es obligatorio.

La copia de los archivos de adjuntos ocurre **después** del commit y no es transaccional: el sistema
de archivos no participa de la transacción SQLite. Si falla, la base ya está bien y el importador
informa qué archivos no se pudieron copiar; se copian a mano y no hace falta reimportar.

---

## 7. Verificación post-import

La fase 6 no es opcional y no se puede saltear con un flag. Un import que no verifica no sirve: el
usuario no tiene manera de saber si perdió datos.

### 7.1 Conteos

Para cada par de tablas origen/destino:

```sql
-- origen
SELECT COUNT(*) FROM Movimientos;
-- destino
SELECT COUNT(*) FROM movimientos;
```

| Tabla | Relación esperada |
| --- | --- |
| las 15 tablas de copia directa | destino `==` origen |
| `tipos_movimiento` | destino `==` origen (el seed ya estaba, las 4 de sistema no se duplican) |
| `tipos_concepto_pago` | destino `>=` origen (el seed agrega filas) |
| `cliente_contactos` | destino `>=` origen (§5.3 puede agregar) |
| `asistencias_empleado` | destino `==` origen, pero con `is_deleted = 1` en las colisiones (§4.13) |

Cualquier diferencia que no encaje en la relación esperada es **bloqueante**.

### 7.2 Sumas monetarias

Para cada una de las 34 columnas escaladas (doc 04 §2), se compara la suma:

```
suma_destino == suma_origen × factor_de_escala
```

La comparación es de **enteros exactos**, sin tolerancia. No hay épsilon porque no hay punto flotante
en ninguno de los dos lados. Una diferencia de una unidad es un error real.

Las dos excepciones documentadas:

| Columna | Por qué no cuadra la suma exacta |
| --- | --- |
| `pagos_factura.monto` | la escala se decidió fila por fila (§3.3); se verifica sumando con el factor de cada fila |
| las 4 columnas con default `0 → 10000` (§3.4) | la suma del destino es mayor; se verifica sumando el ajuste esperado |

### 7.3 Invariantes de negocio

Sobre la base ya importada, en la misma transacción:

| # | Invariante | Consulta |
| --- | --- | --- |
| 1 | `total == subtotal + iva` en toda factura | `SELECT COUNT(*) FROM facturas WHERE total <> subtotal + iva` → 0 |
| 2 | ningún movimiento sin tipo | `SELECT COUNT(*) FROM movimientos WHERE tipo_movimiento_id IS NULL` → 0 |
| 3 | ninguna FK huérfana | una consulta `LEFT JOIN` por cada FK del esquema; 0 filas |
| 4 | fechas civiles a medianoche | `SELECT COUNT(*) FROM asistencias_empleado WHERE fecha NOT LIKE '%T00:00:00.000Z'` → 0, repetido para las 15 columnas civiles |
| 5 | `movimientos.cantidad` nunca cero | `SELECT COUNT(*) FROM movimientos WHERE cantidad = 0` → 0 |
| 6 | multiplicadores nunca cero | ídem para las 3 columnas de `liquidaciones` |
| 7 | a lo sumo un contacto principal por cliente | `GROUP BY cliente_id HAVING SUM(es_principal) > 1` → 0 filas |
| 8 | un adelanto en una sola liquidación | garantizado por el índice único; se verifica igual |
| 9 | `row_version` de 8 bytes | `SELECT COUNT(*) FROM t WHERE LENGTH(row_version) <> 8` → 0, por tabla |
| 10 | todo `estado` dentro del rango del enum | los `CHECK` del esquema ya lo garantizan; se verifica que no haya `NULL` |

Las invariantes 1 a 9 son **bloqueantes**: si fallan, rollback y código de salida `4`.

### 7.4 Muestreo de filas

Además de los agregados, se comparan filas completas: 100 filas al azar de cada tabla, más las 10
primeras y las 10 últimas por `created_at`. Para cada una se verifica campo por campo aplicando la
transformación esperada.

El muestreo detecta errores que las sumas no ven: dos filas con los importes intercambiados dan la
misma suma.

### 7.5 Estructura del reporte

`import_report.json`:

```json
{
  "tool_version": "0.1.0",
  "started_at": "2026-08-29T14:30:12.000Z",
  "finished_at": "2026-08-29T14:31:47.412Z",
  "source": {
    "path": "C:\\Users\\...\\electroobra_legacy.db",
    "schema_version": "20260828214627_RescaleMonetaryValues",
    "scale_state": "AlreadyScaled",
    "integrity_check": "ok"
  },
  "target": { "path": "C:\\Users\\...\\electroobra.db" },
  "dry_run": false,
  "outcome": "SuccessWithWarnings",
  "tables": [
    {
      "source": "Movimientos",
      "target": "movimientos",
      "source_rows": 4821,
      "target_rows": 4821,
      "skipped": 0,
      "monetary_sums": [
        { "column": "monto", "source": 918_450_000_000, "target": 918_450_000_000, "match": true }
      ]
    }
  ],
  "derived": {
    "certificados": 37,
    "certificado_items": 214,
    "liquidacion_adelantos": 96,
    "contactos_creados": 12,
    "feriados_recuperados": 5,
    "facturas_reclasificadas": { "Pagada": 8, "PagadaParcial": 3, "Vencida": 1 },
    "vencimientos_estimados": 142
  },
  "warnings": [
    {
      "code": "PAGO_ESCALA_HEURISTICA",
      "table": "PagosFactura",
      "row_id": "0f8fad5b-d9cb-469f-a165-70867728950e",
      "detail": { "raw": 45000, "resolved": 450000000, "invoice_total": 1200000000 }
    }
  ],
  "blocking_issues": [],
  "attachments": { "files_copied": 88, "files_missing": 2, "orphan_files": 1 }
}
```

Códigos de advertencia definidos:

| Código | Significado |
| --- | --- |
| `PAGO_ESCALA_HEURISTICA` | la escala de un pago se decidió por §3.3 |
| `ESCALA_SIN_DECIMALES` | la base venía sin escalar; los centavos originales se perdieron antes del import |
| `ASISTENCIA_COLISION` | dos asistencias del mismo día se colapsaron (§4.13) |
| `PORCENTAJE_EXCEDE_100` | un ítem tiene acumulado mayor a 100 (§4.9) |
| `ADELANTO_SUMA_DIFIERE` | el detalle derivado no coincide con `total_adelantos` (§5.2) |
| `VENCIMIENTO_ESTIMADO` | la fecha de vencimiento de la factura se calculó, no se importó (§4.10) |
| `ADJUNTO_ARCHIVO_FALTA` | la fila existe y el archivo no |
| `ADJUNTO_HUERFANO` | el archivo existe y la fila no |
| `ADJUNTO_EXCEDE_LIMITE` | adjunto viejo mayor que el límite nuevo |
| `COLOR_HEX_INVALIDO` | `ColorHex` de categoría no parseable, se guardó `NULL` |
| `FERIADO_NO_PARSEABLE` | un feriado del JSON viejo no se pudo leer |
| `CONCEPTO_PAGO_ID_DISTINTO` | el seed y el origen tienen el mismo nombre con distinto id (§4.2) |
| `FK_HUERFANA_ANULADA` | una FK nullable huérfana se puso en `NULL` con `--allow-orphans` |

---

## 8. Tests del importador

El importador tiene su propia suite. No se considera terminado sin ella.

| Test | Qué verifica |
| --- | --- |
| `import_base_vacia` | una base vieja sin filas produce un destino con sólo el seed |
| `import_base_escalada` | fixture con `RescaleMonetaryValues` aplicada: factor 1, sumas idénticas |
| `import_base_sin_escalar` | fixture sin esa migración: factor 10 000, advertencia `ESCALA_SIN_DECIMALES` |
| `pago_escala_mixta` | fixture con pagos en las dos escalas: cada uno se resuelve bien |
| `fecha_civil_no_cambia_de_dia` | una asistencia guardada a las 22:30 locales queda en el mismo día |
| `fecha_negocio_con_hora_se_convierte` | un movimiento a las 22:00 locales queda en `01:00Z` del día siguiente |
| `auditoria_no_se_desplaza` | un `CreatedAt` no cambia de valor |
| `colision_asistencia_conserva_la_ultima` | dos filas del mismo día: la de `CreatedAt` mayor queda viva |
| `cantidad_cero_se_vuelve_uno` | `cantidad = 0` produce `10000` |
| `multiplicador_cero_se_vuelve_uno` | ídem para los tres multiplicadores |
| `certificado_derivado_coincide_con_el_dominio` | el certificado derivado es bit a bit igual al que produciría la aplicación |
| `adelanto_no_se_duplica_entre_liquidaciones` | dos liquidaciones solapadas: el adelanto va a la más antigua |
| `email_cliente_se_vuelve_contacto_principal` | y no se duplica si ya existía |
| `factura_con_pago_parcial_queda_pagada_parcial` | la reclasificación de §5.5 |
| `fk_huerfana_obligatoria_aborta` | sin `--allow-orphans` y con él, aborta igual |
| `fk_huerfana_nullable_se_anula` | sólo con `--allow-orphans` |
| `texto_que_excede_el_limite_aborta` | no se recorta |
| `tipo_de_sistema_modificado_aborta` | un `TiposMovimiento` de sistema con nombre cambiado |
| `dry_run_no_escribe` | con `--dry-run` el destino queda vacío y el reporte se escribe |
| `fallo_en_medio_hace_rollback` | inyectando un error en la tabla 10, el destino queda vacío |
| `reejecucion_sobre_destino_con_datos_aborta` | no se puede importar dos veces sobre la misma base |
| `todas_las_columnas_escaladas_estan_mapeadas` | recorre la lista de doc 04 §2 y verifica que el mapeo de §4 las cubra |
| `todas_las_tablas_del_origen_estan_mapeadas` | lee el `sqlite_master` del fixture y verifica que cada tabla esté en §4 o en la lista de exclusión de §4.18 |

Los dos últimos son tests de **completitud del mapeo**. Son los que impiden que agregar una columna
al esquema deje un hueco silencioso en el importador.

### 8.1 Fixtures

Los fixtures son bases SQLite reales, versionadas en `crates/eo-import-legacy/tests/fixtures/`:

| Fixture | Contenido |
| --- | --- |
| `legacy_empty.db` | esquema completo, sin filas de negocio |
| `legacy_scaled.db` | ~200 filas repartidas, con `RescaleMonetaryValues` aplicada |
| `legacy_unscaled.db` | las mismas filas, sin esa migración |
| `legacy_dirty.db` | contiene a propósito cada caso patológico: FK huérfanas, colisiones de asistencia, pagos de escala mixta, porcentajes acumulados sobre 100, colores inválidos, adjuntos sin archivo |

`legacy_dirty.db` es el fixture importante: es el que reproduce lo que aparece en una base real de
producción con dos años de uso.

---

## 9. Procedimiento para el usuario

Lo que se documenta en el README y se le indica al usuario:

1. Cerrar la aplicación vieja por completo.
2. Copiar `electroobra.db` a un lugar seguro. Esta copia es el respaldo real; no se toca.
3. Ejecutar con `--dry-run` primero:

```bash
eo-import-legacy \
  --source "C:\Users\Usuario\AppData\Local\ElectroObraApp\electroobra.db" \
  --target "C:\Users\Usuario\AppData\Local\ElectroObraApp\electroobra_new.db" \
  --dry-run
```

4. Leer `import_report.json`. Revisar cada advertencia, en particular las de código
   `PAGO_ESCALA_HEURISTICA` y `ADELANTO_SUMA_DIFIERE`.
5. Si el reporte está bien, ejecutar de nuevo sin `--dry-run`.
6. Renombrar `electroobra_new.db` a `electroobra.db` y abrir la aplicación nueva.
7. Verificar a ojo tres cosas: el total del dashboard del mes en curso, el saldo de un cliente
   conocido, y una liquidación reciente. Si los tres coinciden con lo que mostraba el sistema viejo,
   el import está bien.
8. Conservar la base vieja al menos un mes.

El paso 7 existe porque la verificación automática compara el destino contra el origen, pero no
contra lo que el usuario **recuerda**. Un error de interpretación de negocio pasa las 10 invariantes
de §7.3 sin problema.
