# 04 — Dinero, fechas y tipos base

> Este documento define los tipos primitivos del dominio. Implementarlo **antes** que cualquier
> entidad o caso de uso. Un error acá se propaga a todo el sistema y corrompe datos financieros
> sin que nadie lo note hasta fin de mes.

## 1. Dinero: `Money(i64)` con escala 10 000

### 1.1 La decisión

SQLite no tiene tipo decimal: sus tipos numéricos son `INTEGER` (i64) y `REAL` (f64). Guardar
importes en `REAL` produce errores de representación (`0.1 + 0.2 != 0.3`) que el usuario detecta
inmediatamente al comparar la app contra sus comprobantes.

**Regla**: todo valor decimal se almacena como un `i64` que representa el valor **multiplicado por
10 000**, es decir con **4 decimales de precisión fija**.

| Valor de negocio | Almacenado en SQLite |
| --- | --- |
| `0` | `0` |
| `1` | `10000` |
| `1.5` | `15000` |
| `40000` (una tarifa diaria) | `400000000` |
| `0.0001` | `1` |
| `-240.75` | `-2407500` |
| `60` (un porcentaje del 60 %) | `600000` |
| `8` (un ajuste UOCRA del 8 %) | `80000` |
| `21.5` (días trabajados) | `215000` |

**[LEGADO]** El sistema anterior llegó a esta misma decisión: un `ValueConverter` de EF Core
aplicado **por reflexión a todas las propiedades `decimal` y `decimal?` del modelo** convertía a
`long` con esta expresión exacta:

```csharp
v => (long)Math.Round(v * 10_000, MidpointRounding.AwayFromZero)   // a la base
v => v / (decimal)10_000                                            // desde la base
```

Por ser aplicado por reflexión, **no distinguía importes de porcentajes**: los porcentajes de
certificación, los multiplicadores de liquidación y `DiasTrabajados` quedaron escalados igual. El
sistema nuevo mantiene esa escala para todos ellos, pero con tipos distintos para que el compilador
impida mezclarlos.

### 1.2 Los dos newtypes

```rust
// crates/eo-domain/src/money.rs

/// Escala fija de la representación en base: 4 decimales.
pub const SCALE: i64 = 10_000;

/// Importe monetario. Entero escalado ×10 000. Nunca usar f64 para dinero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Money(i64);

/// Cantidad decimal no monetaria: porcentajes, multiplicadores, cantidades, días.
/// Misma escala que Money, tipo distinto para que no se puedan sumar entre sí.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Decimal4(i64);
```

Ambos se serializan a JSON **como string decimal** (`"1234.5600"`), nunca como número, para que
JavaScript no los degrade a `f64`. Ver §1.6.

### 1.3 API obligatoria de `Money`

```rust
impl Money {
    pub const ZERO: Money = Money(0);
    pub const SCALE: i64 = SCALE;

    /// Construye desde la representación en base (lo que hay en la columna INTEGER).
    pub const fn from_raw(raw: i64) -> Self;
    /// Devuelve la representación en base (lo que va a la columna INTEGER).
    pub const fn raw(self) -> i64;

    /// Construye desde unidades enteras: from_units(40_000) == "40000.0000".
    pub fn from_units(units: i64) -> Result<Self, DomainError>;
    /// Parsea "1234.56", "-240.75", "1234", ".5". Rechaza más de 4 decimales.
    pub fn parse(s: &str) -> Result<Self, DomainError>;

    pub fn checked_add(self, rhs: Money) -> Result<Money, DomainError>;
    pub fn checked_sub(self, rhs: Money) -> Result<Money, DomainError>;
    /// Multiplica por un factor decimal (cantidad, porcentaje, multiplicador).
    pub fn checked_mul(self, factor: Decimal4) -> Result<Money, DomainError>;
    /// Divide por un factor decimal.
    pub fn checked_div(self, divisor: Decimal4) -> Result<Money, DomainError>;

    /// Redondea a `decimals` decimales (0..=4) con half-away-from-zero.
    pub fn round_to(self, decimals: u32) -> Money;

    pub fn is_zero(self) -> bool;
    pub fn is_negative(self) -> bool;
    pub fn abs(self) -> Money;
    pub fn neg(self) -> Money;

    /// Suma una secuencia con detección de desborde.
    pub fn try_sum<I: IntoIterator<Item = Money>>(iter: I) -> Result<Money, DomainError>;
}
```

`Decimal4` expone la misma superficie más:

```rust
impl Decimal4 {
    pub const ZERO: Decimal4 = Decimal4(0);
    pub const ONE: Decimal4 = Decimal4(10_000);          // 1.0
    pub const HUNDRED: Decimal4 = Decimal4(1_000_000);   // 100.0
    pub const HALF: Decimal4 = Decimal4(5_000);          // 0.5

    /// Interpreta self como porcentaje y devuelve la fracción: 60 % -> 0.6
    pub fn as_fraction(self) -> Decimal4;
}
```

### 1.4 Regla de redondeo: half-away-from-zero

**Siempre** `MidpointRounding.AwayFromZero`, nunca banker's rounding (que es el default de Rust
`round()` para f64 y de muchas bibliotecas decimales).

| Valor exacto | Redondeo a 2 decimales | Resultado |
| --- | --- | --- |
| `2.345` | half-away-from-zero | `2.35` |
| `2.355` | half-away-from-zero | `2.36` |
| `-2.345` | half-away-from-zero | `-2.35` |
| `2.344` | half-away-from-zero | `2.34` |

Implementación de referencia (sin punto flotante en ningún momento):

```rust
fn round_raw(raw: i64, from_scale: i64, to_scale: i64) -> i64 {
    debug_assert!(from_scale % to_scale == 0);
    let factor = from_scale / to_scale;          // p. ej. 10_000 / 100 = 100
    if factor == 1 { return raw; }
    let half = factor / 2;
    let q = raw / factor;
    let r = raw % factor;
    if r.abs() >= half {
        if raw >= 0 { (q + 1) * factor } else { (q - 1) * factor }
    } else {
        q * factor
    }
}
```

### 1.5 Multiplicación: el punto donde se pierde precisión

Multiplicar dos valores escalados ×10 000 da un resultado escalado ×100 000 000. Hay que dividir
por 10 000 **con redondeo half-away-from-zero**, y hacerlo en `i128` para no desbordar:

```rust
impl Money {
    pub fn checked_mul(self, factor: Decimal4) -> Result<Money, DomainError> {
        let product: i128 = (self.0 as i128) * (factor.raw() as i128);
        let scale = SCALE as i128;
        let half = scale / 2;
        let q = product / scale;
        let r = product % scale;
        let adjusted = if r.abs() >= half {
            if product >= 0 { q + 1 } else { q - 1 }
        } else {
            q
        };
        i64::try_from(adjusted).map(Money).map_err(|_| DomainError::MoneyOverflow)
    }
}
```

**Regla de orden de operaciones**: en una fórmula con varias multiplicaciones y una división por
100 (porcentajes), **primero se multiplica todo y se divide al final**, para no acumular errores de
redondeo intermedios. Cada fórmula del documento 06 indica su orden exacto; respetarlo al pie de la
letra o los totales no van a coincidir con los del sistema anterior.

### 1.6 Serialización hacia el frontend

```rust
impl Serialize for Money {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_decimal_string())   // "1234.5600"
    }
}
```

- Formato: signo opcional, parte entera sin separadores de miles, punto, **exactamente 4
  decimales**. Ejemplos: `"0.0000"`, `"40000.0000"`, `"-240.7500"`.
- Deserialización: acepta string decimal con 0 a 4 decimales; rechaza número JSON, notación
  científica y más de 4 decimales con `DomainError::InvalidScale`.
- **El frontend nunca hace aritmética con estos valores.** Los muestra formateados y los envía de
  vuelta como string. Todo cálculo ocurre en Rust. Ver [`16-frontend.md`](./16-frontend.md) §6.

### 1.7 Formato de presentación

El formateo para la interfaz y los reportes vive en el frontend y en el generador de reportes, y sale
de configuración (doc 14):

| Configuración | Default | Efecto |
| --- | --- | --- |
| `Locale.SimboloMoneda` | `$` | prefijo |
| `Locale.DecimalesMoneda` | `2` | decimales mostrados (el almacenamiento sigue siendo 4) |
| `Locale.SeparadorMiles` | `.` | convención argentina |
| `Locale.SeparadorDecimal` | `,` | convención argentina |
| `Locale.DecimalesPorcentaje` | `2` | |

Un importe `40000.0000` se muestra como `$ 40.000,00`. Los cuatro decimales internos **no** se
muestran nunca, pero **sí** se usan en todos los cálculos.

## 2. Columnas escaladas: inventario exhaustivo

Toda columna `INTEGER` de esta lista contiene un valor escalado ×10 000. Cualquier otra columna
`INTEGER` del esquema es un entero real (`obras.numero`, `adjuntos.tamano`, `certificados.numero`,
los enums, los booleanos, `orden_trabajo_items.orden`).

| Tabla | Columna | Tipo de dominio | Semántica |
| --- | --- | --- | --- |
| `movimientos` | `monto` | `Money` | importe unitario |
| `movimientos` | `cantidad` | `Decimal4` | cantidad, default `1.0` |
| `movimientos` | `cotizacion_aplicada` | `Money` (nullable) | cotización del dólar |
| `trabajos` | `presupuesto` | `Money` | |
| `ordenes_trabajo` | `ajuste_uocra_porcentaje` | `Decimal4` | **porcentaje**, no monto |
| `ordenes_trabajo` | `otros_descuentos` | `Money` | monto |
| `orden_trabajo_items` | `cantidad` | `Decimal4` | metros, unidades… |
| `orden_trabajo_items` | `precio_unitario` | `Money` | |
| `orden_trabajo_items` | `porcentaje_anterior` | `Decimal4` | porcentaje |
| `orden_trabajo_items` | `porcentaje_actual` | `Decimal4` | porcentaje |
| `certificados` | `total_certificado` | `Money` | |
| `certificados` | `ajuste_uocra` | `Money` | monto ya calculado |
| `certificados` | `otros_descuentos` | `Money` | |
| `certificados` | `total_neto` | `Money` | |
| `certificado_items` | `cantidad` | `Decimal4` | |
| `certificado_items` | `precio_unitario` | `Money` | |
| `certificado_items` | `porcentaje_anterior` | `Decimal4` | |
| `certificado_items` | `porcentaje_actual` | `Decimal4` | |
| `certificado_items` | `subtotal_actual` | `Money` | |
| `certificado_items` | `subtotal_acumulado` | `Money` | |
| `facturas` | `subtotal` | `Money` | |
| `facturas` | `iva` | `Money` | **monto** de IVA, no tasa |
| `facturas` | `total` | `Money` | |
| `pagos_factura` | `monto` | `Money` | |
| `empleados` | `sueldo_base` | `Money` | |
| `empleados` | `tarifa_diaria` | `Money` | |
| `liquidaciones` | `dias_trabajados` | `Decimal4` | admite medias jornadas |
| `liquidaciones` | `tarifa_aplicada` | `Money` | |
| `liquidaciones` | `multiplicador_sabado` | `Decimal4` | default `1.0` = `10000` |
| `liquidaciones` | `multiplicador_domingo` | `Decimal4` | default `1.0` = `10000` |
| `liquidaciones` | `multiplicador_feriado` | `Decimal4` | default `1.0` = `10000` |
| `liquidaciones` | `total_bruto` | `Money` | |
| `liquidaciones` | `total_adelantos` | `Money` | |
| `liquidacion_adelantos` | `monto` | `Money` | |

Son **34 columnas escaladas**. La tabla de arriba es la fuente para el importador de datos legados
(doc 15) y para el test que verifica que ninguna columna escalada quedó mapeada a `f64`.

**[BUG-LEGADO]** El sistema anterior tenía una lista equivalente, `MonetaryColumnRegistry`, pero con
**21** entradas contra **23** columnas `decimal` a las que el converter se aplicaba por reflexión.
`pagos_factura.monto` y `movimientos.cotizacion_aplicada` quedaron fuera de la lista, y por eso la
migración de reescalado no las tocó: en una base real esas dos columnas pueden tener las dos escalas
mezcladas. El importador las resuelve una por una con las heurísticas de
[`15-migracion-de-datos.md`](./15-migracion-de-datos.md) §3.3.

La lección que justifica el test de completitud: una lista de columnas escaladas mantenida a mano se
desincroniza del esquema. Acá la lista **es** la que consume el test, no una copia.

## 3. Fechas y horas: todo UTC

### 3.1 La regla

- **Almacenamiento**: `TEXT` con ISO-8601 UTC, milisegundos y sufijo `Z`:
  `2026-08-29T15:04:05.123Z`. Longitud fija de 24 caracteres, con lo cual el orden lexicográfico
  coincide con el orden cronológico y `ORDER BY fecha` funciona.
- **Dominio**: `chrono::DateTime<Utc>`. Nunca `NaiveDateTime` salvo el caso de fecha civil (§3.3).
- **Presentación**: la conversión a hora local ocurre **exclusivamente** en el frontend al formatear
  y en el generador de reportes. Ninguna consulta ni cálculo usa hora local.

### 3.2 El problema que esto corrige

**[BUG-LEGADO]** El sistema anterior mezclaba las dos cosas:

| Uso | Qué hacía |
| --- | --- |
| Auditoría (`CreatedAt`, `UpdatedAt`, `DeletedAt`) | `DateTime.UtcNow` |
| Fechas de negocio (`Movimiento.Fecha`, `Factura.Fecha`, `OrdenTrabajo.Fecha`) | `DateTime.Now`, hora **local** |

Consecuencias reales: un movimiento cargado a las 22:00 hora argentina (UTC−3) se guardaba con
fecha local, pero al compararlo contra un rango construido con `UtcNow` caía en el día siguiente o
el anterior según el caso. Los totales de un mes podían incluir o excluir movimientos de borde.

**[NUEVO]** Regla única: todo instante es UTC. La conversión desde lo que el usuario tipea sucede
en un solo lugar del backend (§3.4).

### 3.3 Fechas civiles (sin hora)

Algunas fechas son **días del calendario**, no instantes: la fecha de una asistencia, el rango de
una liquidación, la fecha de emisión de una factura.

Regla: se almacenan como el instante UTC de **medianoche de ese día civil**, es decir con la hora
en `00:00:00.000Z`. En Rust se manejan como `NaiveDate` en la lógica de negocio y se convierten en
el borde de persistencia:

```rust
pub fn civil_to_utc(d: NaiveDate) -> DateTime<Utc> {
    Utc.from_utc_datetime(&d.and_hms_milli_opt(0, 0, 0, 0).expect("valid midnight"))
}

pub fn utc_to_civil(dt: DateTime<Utc>) -> NaiveDate {
    dt.date_naive()
}
```

Columnas que son fecha civil y **deben** estar normalizadas a medianoche:

| Tabla | Columna |
| --- | --- |
| `asistencias_empleado` | `fecha` |
| `liquidaciones` | `fecha_inicio`, `fecha_fin` |
| `facturas` | `fecha`, `fecha_vencimiento` |
| `pagos_factura` | `fecha` |
| `trabajos` | `fecha_inicio`, `fecha_fin` |
| `empleados` | `fecha_ingreso` |
| `ordenes_trabajo` | `fecha` |
| `certificados` | `fecha` |
| `liquidacion_adelantos` | `fecha` |

Sin esta normalización, el índice único `(empleado_id, fecha)` de asistencia **no funciona**: dos
cargas del mismo día con distinta hora crearían dos filas y se rompería INV-07.

`movimientos.fecha` **no** está en la lista: es la fecha del movimiento y conserva la hora, porque el
usuario puede querer distinguir dos movimientos del mismo día. Los filtros por rango de fechas la
tratan como `[inicio 00:00:00.000Z, fin 23:59:59.999Z]`.

### 3.4 Zona horaria de entrada

La zona horaria del usuario sale de configuración: `Locale.ZonaHoraria`, default
`America/Argentina/Buenos_Aires`.

- El frontend envía siempre ISO-8601 **con offset** o ya en UTC (`Z`).
- Un valor de fecha civil se envía como `YYYY-MM-DD` (sin hora) y el backend lo interpreta
  directamente como día civil, **sin** aplicar conversión de zona. Esto evita el clásico
  corrimiento de un día.
- Un instante se envía con offset (`2026-08-29T22:00:00-03:00`) y el backend lo normaliza a UTC.

### 3.5 Rangos de fecha en las consultas

Un filtro «del 1 al 31 de agosto» se traduce a:

```sql
WHERE fecha >= '2026-08-01T00:00:00.000Z'
  AND fecha <= '2026-08-31T23:59:59.999Z'
```

Siempre inclusivo en ambos extremos, y siempre construido con esa granularidad de milisegundos.
Prohibido usar `BETWEEN` con el día siguiente a medianoche: produce inclusiones accidentales.

### 3.6 El reloj es un puerto

```rust
#[async_trait]
pub trait Clock: Send + Sync {
    fn now_utc(&self) -> DateTime<Utc>;
    fn today_civil(&self, tz: &Tz) -> NaiveDate;
}
```

Ningún caso de uso llama a `Utc::now()` directamente. Los tests inyectan un reloj fijo y comparan
resultados exactos. `today_civil` recibe la zona porque «hoy» depende de la zona del usuario: a las
21:00 del 29 de agosto en Argentina, en UTC ya es el 30.

## 4. Borrado lógico (soft delete)

### 4.1 Las columnas

```sql
is_deleted INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
deleted_at TEXT NULL
```

Invariante: `is_deleted = 1` ⟺ `deleted_at IS NOT NULL`. Se verifica en un test de consistencia.

### 4.2 Comportamiento

- Borrar = `UPDATE ... SET is_deleted = 1, deleted_at = <now_utc>, row_version = <siguiente>`.
- **Toda** consulta de lectura agrega `is_deleted = 0`. En la implementación con SeaORM esto se hace
  con un helper obligatorio, no a mano en cada consulta:

```rust
// crates/eo-infrastructure/src/persistence/mod.rs
pub trait AliveFilter: Sized {
    /// Agrega el filtro de borrado lógico. Usar SIEMPRE al construir una consulta.
    fn alive(self) -> Self;
}
```

  Hay un test que recorre cada método de repositorio y falla si alguno emite SQL sin el filtro.
- Restaurar: `is_deleted = 0, deleted_at = NULL`. Sólo se ofrece desde la pantalla de configuración,
  no desde los listados.
- **No hay purga automática.** Si alguna vez se implementa, va como comando explícito con
  confirmación y backup previo obligatorio.

### 4.3 Interacción con los índices únicos

Hay que decidir caso por caso si el índice único filtra por `is_deleted`:

| Índice | ¿Filtra por `is_deleted`? | Por qué |
| --- | --- | --- |
| `ux_obras_numero` | **No** | un número de obra borrado sigue reservado (INV-06) |
| `ux_asistencias_empleado_empleado_fecha` | **No** | fuerza el *upsert* del ciclo de asistencia (INV-07) |
| `ux_certificados_orden_numero` | **No** | un número de certificado no se reutiliza (INV-15) |
| `ux_tipos_movimiento_nombre` | Sí | el usuario puede reusar el nombre de un tipo borrado |
| `ux_tipos_concepto_pago_nombre` | Sí | ídem |
| `ux_categorias_nombre_padre` | Sí | ídem |
| `ux_cliente_contactos_cliente_email` | Sí | se puede volver a agregar un email borrado |
| `ux_liquidacion_adelantos_movimiento` | Sí | anular una liquidación libera sus adelantos (INV-05) |

Esta tabla no es un detalle: si se equivoca el criterio, el usuario recibe errores de unicidad
incomprensibles o bien se corrompe una secuencia.

### 4.4 Interacción con las claves foráneas

Las acciones `ON DELETE` de [`03-modelo-de-datos.md`](./03-modelo-de-datos.md) §4 **no se disparan**
con el borrado lógico, porque no hay `DELETE` real. Por lo tanto:

- Antes de borrar lógicamente un padre, el caso de uso **debe** verificar dependencias vivas y
  devolver `AppError::DependencyInUse` con su clave i18n. Las acciones `RESTRICT` son sólo la red de
  seguridad de la base.
- Al borrar lógicamente un padre con hijos `CASCADE` (cliente → contactos, factura → pagos, orden →
  ítems y certificados, empleado → asistencias y liquidaciones), el caso de uso marca los hijos como
  borrados **en la misma transacción**. La base no lo hace sola.

Matriz de cascada lógica que hay que implementar a mano:

| Al borrar lógicamente | Marcar también como borrados |
| --- | --- |
| `clientes` | `cliente_contactos` del cliente |
| `facturas` | `pagos_factura` de la factura |
| `ordenes_trabajo` | `orden_trabajo_items`, `certificados`, `certificado_items` de la orden |
| `certificados` | `certificado_items` del certificado |
| `empleados` | `asistencias_empleado`, `liquidaciones`, `liquidacion_adelantos` del empleado |
| `trabajos` | `ordenes_trabajo` y su cascada |
| `liquidaciones` | `liquidacion_adelantos` de la liquidación |

## 5. Concurrencia optimista: `row_version`

### 5.1 Representación

```sql
row_version BLOB NOT NULL DEFAULT X'0000000000000001'
```

`BLOB` de **exactamente 8 bytes**, interpretado como `u64` **big-endian**. El valor inicial es 1.
En Rust:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowVersion([u8; 8]);

impl RowVersion {
    pub const INITIAL: RowVersion = RowVersion([0, 0, 0, 0, 0, 0, 0, 1]);

    pub fn next(self) -> RowVersion {
        RowVersion(u64::from_be_bytes(self.0).wrapping_add(1).to_be_bytes())
    }
}
```

**[LEGADO]** El sistema anterior declaraba la columna con este mismo default de 8 bytes pero
delegaba el control a EF Core; el valor casi nunca se incrementaba de forma explícita. En el sistema
nuevo el incremento es responsabilidad del repositorio y es obligatorio.

### 5.2 Protocolo de actualización

1. El caso de uso lee la entidad y obtiene su `row_version` actual.
2. El DTO que viaja al frontend **incluye** `row_version` como string hexadecimal de 16 caracteres
   (`"0000000000000001"`).
3. El frontend lo devuelve tal cual en el DTO de actualización. No lo interpreta ni lo muestra.
4. El repositorio ejecuta:

```sql
UPDATE movimientos
   SET  /* … campos … */,
        updated_at  = :now,
        row_version = :next_version
 WHERE id = :id
   AND row_version = :expected_version
   AND is_deleted = 0;
```

5. Si el `UPDATE` afecta **0 filas**, se devuelve `AppError::Concurrency { entity }` y el frontend
   muestra la clave i18n `Error.Concurrency` invitando a recargar.

Lo mismo aplica al borrado lógico: también lleva `row_version` en el `WHERE`.

### 5.3 Qué NO protege

`row_version` protege una fila. **No** protege un agregado completo: si dos operaciones tocan
distintos ítems de la misma orden de trabajo, ambas pueden tener éxito. Para operaciones que deben
ser atómicas a nivel agregado (emitir certificado, crear liquidación) se usa **transacción + lectura
con `row_version` de la raíz del agregado**, incrementando la raíz aunque no cambien sus campos. Los
casos de uso afectados están marcados en [`06-casos-de-uso-y-formulas.md`](./06-casos-de-uso-y-formulas.md).

## 6. Identificadores

- Tipo: UUID, columna `TEXT`.
- **[NUEVO]** Versión **7** (ordenable por tiempo). El sistema anterior usaba v4. UUID v7 hace que
  el orden de inserción sea el orden del índice primario, lo que mejora la localidad en SQLite.
  El importador de datos legados **conserva los UUID v4 existentes**: no se regeneran.
- Formato en la base: canónico en minúsculas con guiones,
  `0192f3a1-8c2d-7abc-9def-0123456789ab`. Sin llaves, sin mayúsculas.
- Generación: `IdGenerator` es un puerto, para que los tests puedan producir secuencias
  determinísticas.

```rust
pub trait IdGenerator: Send + Sync {
    fn new_id(&self) -> Uuid;
}
```

## 7. Enums en la base

Se almacenan como `INTEGER` con el **valor numérico explícito** del enum, más un `CHECK` del rango.
Nunca como texto: un cambio de nombre no debe migrar datos.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(i32)]
pub enum TipoJornada {
    Completa = 0,
    Media = 1,
    Falta = 2,
    FaltaJustificada = 3,
    Feriado = 4,
}

impl TryFrom<i32> for TipoJornada {
    type Error = DomainError;
    fn try_from(v: i32) -> Result<Self, Self::Error> { /* exhaustivo, sin `_ =>` silencioso */ }
}
```

- Hacia la base: `as i32`.
- Desde la base: `TryFrom<i32>`; un valor desconocido es un error de datos, **no** se mapea a un
  default silencioso.
- Hacia el frontend: `camelCase` como string (`"faltaJustificada"`), para que el código TypeScript
  sea legible y no dependa de números.
- La lista completa de enums y sus valores está en
  [`05-dominio-entidades.md`](./05-dominio-entidades.md) §3.

## 8. Tipos de resultado y paginación

```rust
// crates/eo-application/src/result.rs
pub type AppResult<T> = Result<T, AppError>;
```

**[LEGADO]** El sistema anterior tenía un `Result<T>` propio con `IsSuccess`, `Error` y `Errors`, y
los mensajes ya traducidos dentro. Se descarta: se usa el `Result` de Rust con `AppError`
(doc 02 §6), que lleva **claves i18n** y no texto.

```rust
// crates/eo-application/src/paging.rs
#[derive(Debug, Clone, Copy)]
pub struct PageRequest {
    /// 1-based. Página 0 no existe.
    pub page: u32,
    /// 0 significa "sin paginar": devolver todo.
    pub size: u32,
}

impl PageRequest {
    pub const DEFAULT_SIZE: u32 = 30;
    pub const ALLOWED_SIZES: [u32; 5] = [10, 30, 50, 100, 0];

    pub fn offset(self) -> u64 { ((self.page.saturating_sub(1)) as u64) * self.size as u64 }
    pub fn limit(self) -> Option<u64> { if self.size == 0 { None } else { Some(self.size as u64) } }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagedResult<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub page: u32,
    pub size: u32,
    pub total_pages: u32,
    pub has_previous: bool,
    pub has_next: bool,
}
```

Fórmulas derivadas, idénticas a las del sistema anterior:

```
total_pages  = if size == 0 { if total_count == 0 { 0 } else { 1 } }
               else { ceil(total_count / size) }
has_previous = page > 1
has_next     = page < total_pages
```

`ALLOWED_SIZES` es la lista que el frontend ofrece (`10 / 30 / 50 / 100 / Todos`) y el default es
**30**. El backend valida que `size` esté en la lista y rechaza cualquier otro valor con
`AppError::Validation`, para que nadie pida 1 000 000 de filas.

## 9. Checklist de implementación de este documento

- [ ] `Money` y `Decimal4` con la API de §1.3, sin ninguna operación que use `f64`.
- [ ] Test: `Money::parse("2.345").round_to(2)` da `2.35`; con `-2.345` da `-2.35`.
- [ ] Test: `Money::from_units(40_000).checked_mul(Decimal4::parse("1.5"))` da `60000.0000`.
- [ ] Test de ida y vuelta: `from_raw(r).raw() == r` para todos los bordes, incluido `i64::MIN + 1`.
- [ ] Test: serializar `Money` produce string con 4 decimales; deserializar un número JSON falla.
- [ ] `RowVersion` con `next()` big-endian y test de desborde.
- [ ] `civil_to_utc` / `utc_to_civil` con test de la medianoche y del cambio de día por zona.
- [ ] `Clock` e `IdGenerator` como puertos, con implementaciones fake en los tests.
- [ ] `PageRequest::ALLOWED_SIZES` validado, con test del caso `size = 0`.
- [ ] Test que recorre el modelo SeaORM y falla si alguna columna de la tabla de §2 no está mapeada
      a `Money` o `Decimal4`.
