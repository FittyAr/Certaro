# 05 — Entidades del dominio

> Define `crates/eo-domain`. Las entidades son **structs puras**: sin atributos de ORM, sin
> `async`, sin I/O. Los tipos `Money`, `Decimal4`, `RowVersion` vienen de
> [`04-dinero-fechas-y-tipos.md`](./04-dinero-fechas-y-tipos.md); las columnas de
> [`03-modelo-de-datos.md`](./03-modelo-de-datos.md).

## 1. Auditoría compartida

Todas las entidades de negocio embeben este bloque. **No** se usa herencia: se usa composición con
un campo `audit` aplanado en la serialización.

```rust
// crates/eo-domain/src/entities/audit.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Audit {
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub row_version: RowVersion,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Audit {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self { created_at: now, updated_at: None, row_version: RowVersion::INITIAL,
               is_deleted: false, deleted_at: None }
    }
    pub fn touch(&mut self, now: DateTime<Utc>) {
        self.updated_at = Some(now);
        self.row_version = self.row_version.next();
    }
    pub fn soft_delete(&mut self, now: DateTime<Utc>) {
        self.is_deleted = true;
        self.deleted_at = Some(now);
        self.touch(now);
    }
}
```

**[LEGADO]** El sistema anterior tenía `BaseEntity` con exactamente estos campos y estos defaults:
`Id = Guid.NewGuid()`, `CreatedAt = DateTime.UtcNow`, `RowVersion = [0,0,0,0,0,0,0,1]`,
`IsDeleted = false`. Se mantiene la semántica pero `created_at` ya no se autogenera en el
constructor: llega desde el puerto `Clock` para que los tests sean determinísticos.

En cada struct de entidad el campo se declara así, y el `id` es siempre el primer campo:

```rust
pub struct Ejemplo {
    pub id: Uuid,
    // … campos propios …
    #[serde(flatten)]
    pub audit: Audit,
}
```

## 2. Las 20 entidades

Orden alfabético. Cada tabla indica: campo Rust, tipo Rust, columna SQL correspondiente y notas.
Todos los campos son públicos; la validación vive en `eo-application`
([`07-validaciones.md`](./07-validaciones.md)).

### 2.1 `Adjunto` → `adjuntos`

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `entidad_tipo` | `EntidadAdjunto` | `entidad_tipo` | **[NUEVO]** enum, no `String`. Se serializa a la base con su nombre exacto. |
| `entidad_id` | `Uuid` | `entidad_id` | sin FK, relación polimórfica |
| `nombre_archivo` | `String` | `nombre_archivo` | máx 255, nombre original sanitizado |
| `ruta_relativa` | `String` | `ruta_relativa` | máx 500, relativa a la raíz de adjuntos |
| `mime` | `String` | `mime` | máx 100 |
| `tamano` | `u64` | `tamano` | bytes, **no** escalado |
| `audit` | `Audit` | bloque de auditoría | |

**[LEGADO]** `EntidadTipo` era un `string` libre con constantes sueltas. Ahora es un enum cerrado
(§3.7) porque un valor mal escrito dejaba adjuntos huérfanos e invisibles.

### 2.2 `AppMetadata` → `app_metadata`

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `key` | `String` | `key` | PK, máx 100 |
| `value` | `String` | `value` | máx 500 |
| `updated_at` | `DateTime<Utc>` | `updated_at` | |

Única entidad **sin** bloque de auditoría y sin borrado lógico.

### 2.3 `AsistenciaEmpleado` → `asistencias_empleado`

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `empleado_id` | `Uuid` | `empleado_id` | requerido |
| `fecha` | `NaiveDate` | `fecha` | **fecha civil**; se persiste como medianoche UTC |
| `tipo_jornada` | `TipoJornada` | `tipo_jornada` | default `Completa` |
| `trabajo_id` | `Option<Uuid>` | `trabajo_id` | opcional |
| `observaciones` | `Option<String>` | `observaciones` | máx 1000 |
| `audit` | `Audit` | | |

Método de dominio:

```rust
impl AsistenciaEmpleado {
    /// Factor de jornada usado por la liquidación. Ver doc 06 §6.3.
    pub fn factor_jornada(&self) -> Decimal4 { self.tipo_jornada.factor() }
}
```

### 2.4 `Categoria` → `categorias`

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `nombre` | `String` | `nombre` | máx 100, requerido |
| `descripcion` | `Option<String>` | `descripcion` | máx 500 |
| `color_hex` | `Option<String>` | `color_hex` | máx 7, `#RRGGBB` |
| `icono` | `Option<String>` | `icono` | máx 50, nombre lógico de icono |
| `categoria_padre_id` | `Option<Uuid>` | `categoria_padre_id` | **[NUEVO]** jerarquía de 2 niveles |
| `audit` | `Audit` | | |

`color_hex` e `icono` son **datos**, no diseño: el usuario elige un color para su categoría. El
frontend los aplica como estilo inline y no forman parte de los tokens del design system.

### 2.5 `Certificado` → `certificados` **[NUEVO]**

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `orden_trabajo_id` | `Uuid` | `orden_trabajo_id` | requerido |
| `numero` | `u32` | `numero` | secuencial desde 1 dentro de la orden |
| `fecha` | `NaiveDate` | `fecha` | fecha civil |
| `observaciones` | `Option<String>` | `observaciones` | máx 1000 |
| `total_certificado` | `Money` | `total_certificado` | congelado al emitir |
| `ajuste_uocra` | `Money` | `ajuste_uocra` | monto congelado |
| `otros_descuentos` | `Money` | `otros_descuentos` | congelado |
| `total_neto` | `Money` | `total_neto` | congelado |
| `items` | `Vec<CertificadoItem>` | — | cargado por el repositorio, no es columna |
| `audit` | `Audit` | | |

### 2.6 `CertificadoItem` → `certificado_items` **[NUEVO]**

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `certificado_id` | `Uuid` | `certificado_id` | |
| `orden_trabajo_item_id` | `Uuid` | `orden_trabajo_item_id` | |
| `cantidad` | `Decimal4` | `cantidad` | copia congelada del ítem |
| `precio_unitario` | `Money` | `precio_unitario` | copia congelada |
| `porcentaje_anterior` | `Decimal4` | `porcentaje_anterior` | |
| `porcentaje_actual` | `Decimal4` | `porcentaje_actual` | |
| `subtotal_actual` | `Money` | `subtotal_actual` | congelado |
| `subtotal_acumulado` | `Money` | `subtotal_acumulado` | congelado |
| `audit` | `Audit` | | |

### 2.7 `Cliente` → `clientes`

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `nombre` | `String` | `nombre` | máx 200, requerido |
| `cuit` | `Option<String>` | `cuit` | máx 13 |
| `direccion` | `Option<String>` | `direccion` | máx 500 |
| `telefono` | `Option<String>` | `telefono` | máx 30 |
| `email` | `Option<String>` | `email` | máx 254, email «principal» heredado |
| `condicion_iva` | `Option<String>` | `condicion_iva` | máx 100, p. ej. «Responsable Inscripto», «Monotributo» |
| `contactos` | `Vec<ClienteContacto>` | — | cargado por el repositorio |
| `audit` | `Audit` | | |

### 2.8 `ClienteContacto` → `cliente_contactos`

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `cliente_id` | `Uuid` | `cliente_id` | |
| `etiqueta` | `String` | `etiqueta` | máx 100, default `"General"`; p. ej. «Personal», «Oficina», «Compras» |
| `email` | `String` | `email` | máx 254, requerido |
| `nombre` | `Option<String>` | `nombre` | **[NUEVO]** máx 200 |
| `telefono` | `Option<String>` | `telefono` | **[NUEVO]** máx 30 |
| `es_principal` | `bool` | `es_principal` | **[NUEVO]** default `false` |
| `audit` | `Audit` | | |

### 2.9 `Empleado` → `empleados`

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `nombre` | `String` | `nombre` | máx 200, requerido |
| `dni` | `Option<String>` | `dni` | máx 15 |
| `cargo` | `Option<String>` | `cargo` | máx 100 |
| `sueldo_base` | `Money` | `sueldo_base` | salario según la frecuencia de pago |
| `pago_frecuencia` | `PaymentFrequency` | `pago_frecuencia` | default `Mensual` |
| `tarifa_diaria` | `Money` | `tarifa_diaria` | **la que usa la liquidación** |
| `email` | `Option<String>` | `email` | máx 254 |
| `telefono` | `Option<String>` | `telefono` | máx 30 |
| `fecha_ingreso` | `NaiveDate` | `fecha_ingreso` | fecha civil |
| `activo` | `bool` | `activo` | default `true` |
| `audit` | `Audit` | | |

Método de dominio:

```rust
impl Empleado {
    /// Tarifa diaria sugerida a partir del sueldo base y la frecuencia. Doc 06 §6.2.
    pub fn tarifa_diaria_sugerida(&self) -> Result<Money, DomainError> {
        self.sueldo_base.checked_div(self.pago_frecuencia.dias_por_periodo())
    }
}
```

### 2.10 `Factura` → `facturas`

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `numero` | `String` | `numero` | máx 50, requerido, **no** único |
| `fecha` | `NaiveDate` | `fecha` | fecha civil de emisión |
| `fecha_vencimiento` | `Option<NaiveDate>` | `fecha_vencimiento` | **[NUEVO]** |
| `cliente_id` | `Uuid` | `cliente_id` | requerido |
| `estado` | `EstadoFactura` | `estado` | default `Borrador` |
| `subtotal` | `Money` | `subtotal` | lo ingresa el usuario |
| `iva` | `Money` | `iva` | **monto**, lo ingresa el usuario |
| `total` | `Money` | `total` | derivado, ver doc 06 §4.1 |
| `observaciones` | `Option<String>` | `observaciones` | máx 1000 |
| `pagos` | `Vec<PagoFactura>` | — | cargado por el repositorio |
| `audit` | `Audit` | | |

Métodos de dominio:

```rust
impl Factura {
    pub fn total_calculado(&self) -> Result<Money, DomainError> {
        self.subtotal.checked_add(self.iva)
    }
    pub fn total_pagado(&self) -> Result<Money, DomainError> {
        Money::try_sum(self.pagos.iter().filter(|p| !p.audit.is_deleted).map(|p| p.monto))
    }
    pub fn saldo_pendiente(&self) -> Result<Money, DomainError> {
        self.total.checked_sub(self.total_pagado()?)
    }
    pub fn esta_saldada(&self) -> Result<bool, DomainError> {
        Ok(self.saldo_pendiente()?.raw() <= 0)
    }
}
```

**Atención**: `total` **se persiste** aunque sea derivado. Es una denormalización heredada
deliberada: el usuario copia el total del papel y el sistema lo recalcula como
`subtotal + iva` en cada guardado. La columna existe para que las consultas de deuda no tengan que
sumar dos campos.

### 2.11 `Liquidacion` → `liquidaciones`

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `empleado_id` | `Uuid` | `empleado_id` | requerido |
| `fecha_inicio` | `NaiveDate` | `fecha_inicio` | fecha civil |
| `fecha_fin` | `NaiveDate` | `fecha_fin` | fecha civil, `>= fecha_inicio` |
| `dias_trabajados` | `Decimal4` | `dias_trabajados` | admite medias jornadas |
| `tarifa_aplicada` | `Money` | `tarifa_aplicada` | copia congelada de la tarifa del empleado |
| `incluir_sabados` | `bool` | `incluir_sabados` | |
| `incluir_domingos` | `bool` | `incluir_domingos` | |
| `incluir_feriados` | `bool` | `incluir_feriados` | |
| `multiplicador_sabado` | `Decimal4` | `multiplicador_sabado` | default `1.0` |
| `multiplicador_domingo` | `Decimal4` | `multiplicador_domingo` | default `1.0` |
| `multiplicador_feriado` | `Decimal4` | `multiplicador_feriado` | default `1.0` |
| `total_bruto` | `Money` | `total_bruto` | congelado |
| `total_adelantos` | `Money` | `total_adelantos` | congelado |
| `observaciones` | `Option<String>` | `observaciones` | máx 1000 |
| `adelantos` | `Vec<LiquidacionAdelanto>` | — | **[NUEVO]** cargado por el repositorio |
| `audit` | `Audit` | | |

```rust
impl Liquidacion {
    /// NO se persiste. Doc 06 §6.5.
    pub fn total_neto(&self) -> Result<Money, DomainError> {
        self.total_bruto.checked_sub(self.total_adelantos)
    }
}
```

### 2.12 `LiquidacionAdelanto` → `liquidacion_adelantos` **[NUEVO]**

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `liquidacion_id` | `Uuid` | `liquidacion_id` | |
| `movimiento_id` | `Uuid` | `movimiento_id` | el adelanto original |
| `monto` | `Money` | `monto` | congelado |
| `fecha` | `NaiveDate` | `fecha` | congelada; es la que sale en el PDF (RC-02) |
| `concepto` | `String` | `concepto` | máx 500, congelado |
| `audit` | `Audit` | | |

### 2.13 `Movimiento` → `movimientos`

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `fecha` | `DateTime<Utc>` | `fecha` | **instante**, conserva hora |
| `concepto` | `String` | `concepto` | máx 500, requerido |
| `monto` | `Money` | `monto` | importe unitario |
| `cantidad` | `Decimal4` | `cantidad` | default `1.0` (RC-03) |
| `tipo_movimiento_id` | `Uuid` | `tipo_movimiento_id` | **requerido** |
| `moneda` | `Moneda` | `moneda` | default `Ars` |
| `cotizacion_aplicada` | `Option<Money>` | `cotizacion_aplicada` | sólo si `moneda == Usd` |
| `tipo_concepto_pago_id` | `Option<Uuid>` | `tipo_concepto_pago_id` | RC-05 |
| `categoria_id` | `Option<Uuid>` | `categoria_id` | nullable en base, exigido por validación |
| `cliente_id` | `Option<Uuid>` | `cliente_id` | |
| `trabajo_id` | `Option<Uuid>` | `trabajo_id` | vía de imputación a la obra |
| `empleado_id` | `Option<Uuid>` | `empleado_id` | RC-05 |
| `factura_id` | `Option<Uuid>` | `factura_id` | |
| `audit` | `Audit` | | |

```rust
impl Movimiento {
    /// NO se persiste (INV-01). Doc 06 §3.1.
    pub fn total(&self) -> Result<Money, DomainError> {
        self.monto.checked_mul(self.cantidad)
    }
    pub fn es_adelanto(&self) -> bool {
        self.tipo_movimiento_id == constants::tipos_movimiento::ADELANTO
    }
}
```

### 2.14 `Obra` → `obras`

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `numero` | `i32` | `numero` | **único global** (INV-06) |
| `nombre` | `String` | `nombre` | máx 200, requerido |
| `direccion` | `Option<String>` | `direccion` | máx 500 |
| `localidad` | `Option<String>` | `localidad` | máx 200 |
| `cliente_id` | `Uuid` | `cliente_id` | requerido |
| `estado` | `EstadoObra` | `estado` | default `Activa` |
| `audit` | `Audit` | | |

### 2.15 `OrdenTrabajo` → `ordenes_trabajo`

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `trabajo_id` | `Uuid` | `trabajo_id` | requerido |
| `titulo` | `String` | `titulo` | máx 200, requerido |
| `numero_certificado` | `Option<String>` | `numero_certificado` | último certificado emitido |
| `fecha` | `NaiveDate` | `fecha` | fecha civil |
| `observaciones` | `Option<String>` | `observaciones` | sin límite declarado |
| `ajuste_uocra_porcentaje` | `Decimal4` | `ajuste_uocra_porcentaje` | **porcentaje** (p. ej. `8` = 8 %) |
| `otros_descuentos` | `Money` | `otros_descuentos` | **monto** |
| `items` | `Vec<OrdenTrabajoItem>` | — | cargado por el repositorio |
| `audit` | `Audit` | | |

### 2.16 `OrdenTrabajoItem` → `orden_trabajo_items`

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `orden_trabajo_id` | `Uuid` | `orden_trabajo_id` | |
| `descripcion` | `String` | `descripcion` | máx 500, requerido |
| `unidad` | `String` | `unidad` | máx 20, default `"u"`; p. ej. `"m"`, `"ml"`, `"u"`, `"gl"` |
| `cantidad` | `Decimal4` | `cantidad` | metros, unidades… |
| `precio_unitario` | `Money` | `precio_unitario` | |
| `porcentaje_anterior` | `Decimal4` | `porcentaje_anterior` | acumulado de certificados previos |
| `porcentaje_actual` | `Decimal4` | `porcentaje_actual` | avance del certificado en curso |
| `ejecutado` | `bool` | `ejecutado` | «el trabajo se hizo» (RC-11) |
| `nota` | `Option<String>` | `nota` | máx 1000, la leyenda de RC-11 |
| `orden` | `i32` | `orden` | **[NUEVO]** posición en la planilla |
| `audit` | `Audit` | | |

```rust
impl OrdenTrabajoItem {
    /// Doc 06 §5.1.
    pub fn porcentaje_acumulado(&self) -> Result<Decimal4, DomainError> {
        self.porcentaje_anterior.checked_add(self.porcentaje_actual)
    }
    /// Doc 06 §5.2. Devuelve (subtotal_actual, subtotal_acumulado).
    pub fn subtotales(&self) -> Result<(Money, Money), DomainError> {
        let base = self.precio_unitario.checked_mul(self.cantidad)?;
        Ok((
            base.checked_mul(self.porcentaje_actual.as_fraction())?,
            base.checked_mul(self.porcentaje_acumulado()?.as_fraction())?,
        ))
    }
    /// Doc 06 §5.3.
    pub fn porcentaje_pendiente(&self) -> Result<Decimal4, DomainError> {
        Decimal4::HUNDRED.checked_sub(self.porcentaje_acumulado()?)
    }
}
```

### 2.17 `PagoFactura` → `pagos_factura`

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `factura_id` | `Uuid` | `factura_id` | requerido |
| `fecha` | `NaiveDate` | `fecha` | fecha civil |
| `monto` | `Money` | `monto` | requerido, > 0 |
| `medio_pago` | `String` | `medio_pago` | máx 100, texto libre por compatibilidad |
| `audit` | `Audit` | | |

### 2.18 `TipoConceptoPago` → `tipos_concepto_pago`

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `nombre` | `String` | `nombre` | máx 100, requerido |
| `es_sistema` | `bool` | `es_sistema` | si es `true` no se borra |
| `audit` | `Audit` | | |

### 2.19 `TipoMovimiento` → `tipos_movimiento`

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `nombre` | `String` | `nombre` | máx 100, requerido |
| `descripcion` | `Option<String>` | `descripcion` | sin límite declarado |
| `es_ingreso` | `bool` | `es_ingreso` | `true` = suma al balance; `false` = resta |
| `es_sistema` | `bool` | `es_sistema` | si es `true` no se borra ni se cambia `es_ingreso` (INV-04) |
| `audit` | `Audit` | | |

```rust
impl TipoMovimiento {
    pub fn es_de_sistema_protegido(&self) -> bool {
        self.es_sistema || constants::tipos_movimiento::TODOS.contains(&self.id)
    }
    /// Signo con el que el movimiento entra al balance.
    pub fn signo(&self) -> i64 { if self.es_ingreso { 1 } else { -1 } }
}
```

### 2.20 `Trabajo` → `trabajos`

| Campo | Tipo | Columna | Notas |
| --- | --- | --- | --- |
| `id` | `Uuid` | `id` | |
| `obra_id` | `Uuid` | `obra_id` | requerido; el cliente se alcanza por acá |
| `descripcion` | `String` | `descripcion` | máx 500, requerido |
| `fecha_inicio` | `NaiveDate` | `fecha_inicio` | fecha civil |
| `fecha_fin` | `Option<NaiveDate>` | `fecha_fin` | fecha civil |
| `presupuesto` | `Money` | `presupuesto` | |
| `estado` | `EstadoTrabajo` | `estado` | default `Presupuestado` |
| `audit` | `Audit` | | |

## 3. Enums

7 enums. Los seis primeros existen en el sistema anterior; `EntidadAdjunto` y `MedioPago` son
nuevos. Todos derivan `Serialize`/`Deserialize` con `rename_all = "camelCase"` y todos implementan
`TryFrom<i32>` de forma exhaustiva.

### 3.1 `TipoJornada`

| Miembro | Valor | Factor de jornada | Significado |
| --- | --- | --- | --- |
| `Completa` | `0` | `1.0` | jornada completa trabajada |
| `Media` | `1` | `0.5` | media jornada |
| `Falta` | `2` | `0.0` | ausencia sin justificar: no se paga |
| `FaltaJustificada` | `3` | `0.0` | ausencia justificada: **tampoco se paga** |
| `Feriado` | `4` | `1.0` | feriado trabajado; usa el multiplicador de feriado |

```rust
impl TipoJornada {
    pub fn factor(self) -> Decimal4 {
        match self {
            TipoJornada::Completa | TipoJornada::Feriado => Decimal4::ONE,
            TipoJornada::Media => Decimal4::HALF,
            TipoJornada::Falta | TipoJornada::FaltaJustificada => Decimal4::ZERO,
        }
    }
    /// Siguiente estado en el ciclo de clic de la grilla de asistencia. Doc 09 §3.10.
    pub fn siguiente(self) -> TipoJornada {
        match self {
            TipoJornada::Completa => TipoJornada::Media,
            TipoJornada::Media => TipoJornada::Falta,
            TipoJornada::Falta => TipoJornada::FaltaJustificada,
            TipoJornada::FaltaJustificada => TipoJornada::Feriado,
            TipoJornada::Feriado => TipoJornada::Completa,
        }
    }
}
```

**[LEGADO]** `FaltaJustificada` con factor `0.0` significa que una falta justificada **no se
paga**. Es lo que hace hoy el sistema y se conserva. Si el negocio quisiera pagarla, es un cambio de
regla y hay que reflejarlo en este documento y en el test correspondiente, no parchear el `match`.

### 3.2 `EstadoFactura`

| Miembro | Valor | Nota |
| --- | --- | --- |
| `Borrador` | `0` | |
| `Emitida` | `1` | |
| `Pagada` | `2` | |
| `Anulada` | `3` | |
| `Vencida` | `4` | |
| `PagadaParcial` | `5` | **[NUEVO]** se agrega al final para no correr los valores ya persistidos |

Default: `Borrador`. `Pagada`, `PagadaParcial` y `Vencida` **sólo** los escribe el recálculo
automático, nunca el usuario. Transiciones válidas en
[`08-maquinas-de-estado.md`](./08-maquinas-de-estado.md) §2.

### 3.3 `EstadoObra`

| Miembro | Valor |
| --- | --- |
| `Activa` | `0` |
| `Pausada` | `1` |
| `Finalizada` | `2` |
| `Cancelada` | `3` |

Default: `Activa`.

### 3.4 `EstadoTrabajo`

| Miembro | Valor |
| --- | --- |
| `Presupuestado` | `0` |
| `EnProceso` | `1` |
| `Pausado` | `2` |
| `Finalizado` | `3` |
| `Cancelado` | `4` |

Default: `Presupuestado`. Cubre RC-08 («si están finalizados o no, si están pausados, si están en
proceso»).

### 3.5 `PaymentFrequency`

| Miembro | Valor | Días por período |
| --- | --- | --- |
| `Diario` | `0` | `1` |
| `Semanal` | `1` | `6` |
| `Quincenal` | `2` | `15` |
| `Mensual` | `3` | `30` |

Default: `Mensual`.

```rust
impl PaymentFrequency {
    /// Divisor para derivar la tarifa diaria del sueldo base. Doc 06 §6.2.
    pub fn dias_por_periodo(self) -> Decimal4 {
        match self {
            PaymentFrequency::Diario   => Decimal4::from_units(1),
            PaymentFrequency::Semanal  => Decimal4::from_units(6),
            PaymentFrequency::Quincenal=> Decimal4::from_units(15),
            PaymentFrequency::Mensual  => Decimal4::from_units(30),
        }
    }
}
```

**[LEGADO]** El sistema anterior documentaba «SueldoBase / 30 si es mensual, SueldoBase / 15 si es
quincenal» pero **nunca implementó la derivación**: `TarifaDiaria` se cargaba a mano. El divisor de
`Semanal` es **6** (semana laboral de lunes a sábado), no 7. Estos cuatro divisores salen de
configuración (`Business.DiasPorFrecuencia.*`, doc 14) y la tabla de arriba son sus defaults.

### 3.6 `Moneda`

| Miembro | Valor | Código ISO |
| --- | --- | --- |
| `Ars` | `0` | `ARS` |
| `Usd` | `1` | `USD` |

Default: `Ars`. Si `moneda == Usd`, `cotizacion_aplicada` es obligatoria.

### 3.7 `EntidadAdjunto` **[NUEVO]**

Valores exactos, tal como se guardan en `adjuntos.entidad_tipo` (se persiste el **nombre**, no un
número, para no romper los datos existentes):

| Miembro | Valor persistido | Tabla apuntada |
| --- | --- | --- |
| `Obra` | `"Obra"` | `obras` |
| `Trabajo` | `"Trabajo"` | `trabajos` |
| `Factura` | `"Factura"` | `facturas` |
| `Movimiento` | `"Movimiento"` | `movimientos` |
| `Empleado` | `"Empleado"` | `empleados` |

**[HUECO]** Considerar agregar `OrdenTrabajo` y `Certificado` cuando se implemente la carga de la
foto de la planilla original. No está en el sistema anterior y no es urgente.

### 3.8 `MedioPago` **[NUEVO]**

Se usa para poblar el desplegable de `pagos_factura.medio_pago`, que sigue siendo `TEXT`:

| Miembro | Valor persistido |
| --- | --- |
| `Efectivo` | `"Efectivo"` |
| `Transferencia` | `"Transferencia"` |
| `Cheque` | `"Cheque"` |
| `Deposito` | `"Depósito"` |
| `Otro` | `"Otro"` |

Al leer un valor que no está en la lista se muestra tal cual: los datos históricos tienen texto
libre y no se normalizan.

## 4. Constantes del dominio

```rust
// crates/eo-domain/src/constants.rs

/// GUID fijos de los tipos de movimiento de sistema. Sembrados por migración (doc 03 §5.1).
pub mod tipos_movimiento {
    use uuid::{uuid, Uuid};

    pub const INGRESO:  Uuid = uuid!("00000000-0000-0000-0000-000000000001");
    pub const GASTO:    Uuid = uuid!("00000000-0000-0000-0000-000000000002");
    pub const ADELANTO: Uuid = uuid!("00000000-0000-0000-0000-000000000003");
    pub const AJUSTE:   Uuid = uuid!("00000000-0000-0000-0000-000000000004");

    pub const TODOS: [Uuid; 4] = [INGRESO, GASTO, ADELANTO, AJUSTE];
}

/// GUID fijos de los conceptos de pago de sistema. [NUEVO], doc 03 §5.2.
pub mod tipos_concepto_pago {
    use uuid::{uuid, Uuid};

    pub const ADELANTO:    Uuid = uuid!("00000000-0000-0000-0000-000000000101");
    pub const QUINCENA:    Uuid = uuid!("00000000-0000-0000-0000-000000000102");
    pub const LIQUIDACION: Uuid = uuid!("00000000-0000-0000-0000-000000000103");
    pub const VIATICO:     Uuid = uuid!("00000000-0000-0000-0000-000000000104");

    pub const TODOS: [Uuid; 4] = [ADELANTO, QUINCENA, LIQUIDACION, VIATICO];
}

/// Longitudes máximas. Espejan los CHECK y los límites del doc 03.
pub mod limites {
    pub const NOMBRE_CORTO: usize = 100;
    pub const NOMBRE_LARGO: usize = 200;
    pub const CONCEPTO: usize = 500;
    pub const DESCRIPCION: usize = 500;
    pub const DIRECCION: usize = 500;
    pub const OBSERVACIONES: usize = 1000;
    pub const EMAIL: usize = 254;
    pub const TELEFONO: usize = 30;
    pub const CUIT: usize = 13;
    pub const DNI: usize = 15;
    pub const UNIDAD: usize = 20;
    pub const COLOR_HEX: usize = 7;
    pub const ICONO: usize = 50;
    pub const NUMERO_FACTURA: usize = 50;
    pub const NUMERO_CERTIFICADO: usize = 50;
    pub const MEDIO_PAGO: usize = 100;
    pub const NOMBRE_ARCHIVO: usize = 255;
    pub const RUTA_RELATIVA: usize = 500;
    pub const MIME: usize = 100;
    pub const METADATA_KEY: usize = 100;
    pub const METADATA_VALUE: usize = 500;
}
```

`ADELANTO` es el GUID que filtra los adelantos en la liquidación (doc 06 §6.4). Está aquí para que
exista **una sola** definición en todo el código.

## 5. Errores del dominio

```rust
// crates/eo-domain/src/error.rs
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DomainError {
    #[error("money overflow")]
    MoneyOverflow,
    #[error("invalid decimal scale: more than 4 decimals")]
    InvalidScale,
    #[error("cannot parse decimal from '{0}'")]
    ParseDecimal(String),
    #[error("division by zero")]
    DivisionByZero,
    #[error("unknown enum value {value} for {enum_name}")]
    UnknownEnumValue { enum_name: &'static str, value: i32 },
    #[error("invalid state transition on {entity}: {from} -> {to}")]
    InvalidStateTransition { entity: &'static str, from: &'static str, to: &'static str },
    #[error("invariant violated: {0}")]
    InvariantViolated(&'static str),
}
```

`InvariantViolated` lleva el identificador del invariante de
[`01-vision-y-reglas-del-negocio.md`](./01-vision-y-reglas-del-negocio.md) §6, por ejemplo
`InvariantViolated("INV-08")`, para que el log sea rastreable.

## 6. Lo que las entidades NO tienen

| Ausencia | Motivo |
| --- | --- |
| Propiedades de navegación bidireccionales | provocan ciclos en la serialización; el repositorio carga sólo hacia abajo (`Factura.pagos`, `OrdenTrabajo.items`, `Certificado.items`, `Cliente.contactos`, `Liquidacion.adelantos`) |
| Nombres desnormalizados (`empleado_nombre`, `cliente_nombre`) | van en los **DTO de salida**, no en la entidad |
| Totales persistidos que se pueden derivar | `Movimiento.total`, `Liquidacion.total_neto`, `OrdenTrabajoItem.porcentaje_acumulado` son métodos |
| Atributos de ORM | los modelos de SeaORM viven en `eo-infrastructure::persistence::models` y se mapean explícitamente |
| Lógica de acceso a datos | ninguna entidad conoce el repositorio |
| Validación | vive en `eo-application::validation` (doc 07) |

Excepción consciente: `Factura.total` y `Liquidacion.total_bruto`/`total_adelantos` **sí** se
persisten. Los tres son valores congelados en el momento de la operación, no derivaciones vivas.
