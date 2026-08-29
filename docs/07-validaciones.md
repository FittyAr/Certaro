# 07 — Validaciones

> Define `crates/eo-application/src/validation/`. Toda validación de entrada vive acá: **ni** en las
> entidades de dominio ([`05`](./05-dominio-entidades.md)), **ni** en los comandos Tauri
> ([`11`](./11-contratos-tauri.md)), **ni** en el frontend como única barrera.
>
> Regla dura: **ningún mensaje de error se escribe literal en Rust**. Cada error lleva una **clave
> i18n** y el frontend la resuelve. Las claves de este documento son la fuente canónica y deben
> existir en `src/locales/es.json` y `src/locales/en.json` (ver [`14`](./14-configuracion-e-i18n.md)).

## 1. Mecanismo

### 1.1 Tipos

```rust
// crates/eo-application/src/validation/mod.rs
use std::collections::BTreeMap;

/// Un único error de validación. `key` es una clave i18n; `params` son los
/// valores a interpolar en el mensaje traducido (ej. {"max": "200"}).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldError {
    /// Ruta del campo en notación de punto, tal como la ve el frontend:
    /// `concepto`, `items[2].porcentajeActual`, `contactos[0].email`.
    pub field: String,
    /// Clave i18n del mensaje. Siempre `Validation.<Entidad>.<Regla>`.
    pub key: &'static str,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<&'static str, String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct ValidationErrors(pub Vec<FieldError>);

impl ValidationErrors {
    pub fn is_empty(&self) -> bool { self.0.is_empty() }

    pub fn push(&mut self, field: impl Into<String>, key: &'static str) {
        self.0.push(FieldError { field: field.into(), key, params: BTreeMap::new() });
    }

    pub fn push_with(
        &mut self,
        field: impl Into<String>,
        key: &'static str,
        params: &[(&'static str, String)],
    ) {
        self.0.push(FieldError {
            field: field.into(),
            key,
            params: params.iter().cloned().collect(),
        });
    }

    /// Prefija todos los campos con `prefix`, para validadores anidados.
    /// `prefix = "items[0]"` convierte `descripcion` en `items[0].descripcion`.
    pub fn nested(mut self, prefix: &str) -> Self {
        for e in &mut self.0 {
            e.field = format!("{prefix}.{}", e.field);
        }
        self
    }

    pub fn merge(&mut self, other: ValidationErrors) {
        self.0.extend(other.0);
    }

    pub fn into_result(self) -> Result<(), ValidationErrors> {
        if self.is_empty() { Ok(()) } else { Err(self) }
    }
}
```

### 1.2 El trait

```rust
/// Validación **pura**: sin I/O, sin acceso a la base, sin `async`.
/// Todo lo que necesita consultar la base es una regla de negocio, no una
/// validación de forma: vive en el caso de uso (§5).
pub trait Validate {
    fn validate(&self) -> Result<(), ValidationErrors>;
}
```

No se usa el `derive` de la crate `validator` para los DTO de este proyecto: los mensajes por
atributo no admiten claves i18n con parámetros de forma legible y las reglas condicionales
(«validar email sólo si no está vacío») terminan en código ilegible. Se implementa `Validate` a
mano. La crate `validator` sólo se usa para su helper de formato de email
(`validator::ValidateEmail`).

### 1.3 Helpers obligatorios

Todos los validadores usan estos helpers. No se duplica lógica de `trim`, longitud o rango.

```rust
// crates/eo-application/src/validation/rules.rs

/// Requerido: `None`, `""` o sólo espacios fallan.
pub fn required_str(errs: &mut ValidationErrors, field: &str, value: &str, key: &'static str);

/// Longitud máxima **en caracteres Unicode** (`chars().count()`), no en bytes.
/// Se evalúa sobre el valor ya recortado. Agrega el param `max`.
pub fn max_len(errs: &mut ValidationErrors, field: &str, value: &str, max: usize, key: &'static str);

/// Longitud exacta entre `min` y `max` caracteres. Agrega los params `min` y `max`.
pub fn len_between(errs: &mut ValidationErrors, field: &str, value: &str, min: usize, max: usize, key: &'static str);

/// `value > 0`. Aplica a `Money` y a `Decimal4`.
pub fn positive(errs: &mut ValidationErrors, field: &str, value: i64, key: &'static str);

/// `value >= 0`.
pub fn non_negative(errs: &mut ValidationErrors, field: &str, value: i64, key: &'static str);

/// `min <= value <= max`, inclusivo, sobre valores escalados.
pub fn between(errs: &mut ValidationErrors, field: &str, value: i64, min: i64, max: i64, key: &'static str);

/// `Uuid` no nulo (`Uuid::nil()` falla).
pub fn required_id(errs: &mut ValidationErrors, field: &str, value: Uuid, key: &'static str);

/// Formato de email. Sólo se evalúa si `value` no está vacío.
pub fn email_format(errs: &mut ValidationErrors, field: &str, value: &Option<String>, key: &'static str);

/// El valor cumple el regex. Sólo se evalúa si `value` no está vacío.
/// El `Regex` se compila una sola vez con `LazyLock`.
pub fn matches(errs: &mut ValidationErrors, field: &str, value: &Option<String>, re: &Regex, key: &'static str);
```

### 1.4 Normalización previa

Antes de validar, el caso de uso llama a `Normalize::normalize(&mut dto)`:

```rust
pub trait Normalize {
    fn normalize(&mut self);
}
```

Reglas de normalización, iguales para todos los DTO:

| Tipo de campo | Normalización |
| --- | --- |
| `String` requerido | `trim()` |
| `Option<String>` opcional | `trim()`; si queda vacío → `None` |
| Email | `trim()` + `to_lowercase()` |
| CUIT | `trim()`; se quitan espacios internos; se conservan los guiones |
| DNI | `trim()`; se quitan puntos y espacios |
| Color hex | `trim()` + `to_uppercase()` |

Normalizar **antes** de validar evita que `"  "` pase un `required` y que dos emails que difieren
sólo en mayúsculas se traten como distintos.

### 1.5 Orden de ejecución en el caso de uso

```
1. normalize(&mut dto)
2. dto.validate()                        → 400 ValidationFailed (todos los errores juntos)
3. reglas de negocio con acceso a datos  → 409 Conflict / 404 NotFound (§5)
4. mapeo a entidad + persistencia
```

Nunca se corta en el primer error de forma: el paso 2 **acumula todos** los `FieldError` y los
devuelve juntos para que el formulario pinte todos los campos en rojo de una sola vez. El paso 3 sí
corta en el primer conflicto.

El error se transporta como `AppError::Validation(ValidationErrors)` y se serializa al frontend según
[`02-arquitectura.md`](./02-arquitectura.md) §5.

## 2. Los 14 validadores

Cada tabla lista: campo, regla, condición exacta, clave i18n. La columna **Cambio** marca lo que
difiere del sistema anterior; los valores sin marca se conservan tal cual.

Convención de las claves: son **exactamente** las del sistema anterior cuando existen, para poder
reutilizar las traducciones ya escritas.

### V-01 `MovimientoDto`

| Campo | Regla | Condición | Clave i18n | Cambio |
| --- | --- | --- | --- | --- |
| `concepto` | requerido | no vacío tras `trim` | `Validation.Movimiento.ConceptoRequired` | |
| `concepto` | longitud | `<= 500` caracteres | `Validation.Movimiento.ConceptoMaxLength` | **[FIX]** era 200 en el validador contra 500 en la columna |
| `monto` | positivo | `> 0` | `Validation.Movimiento.MontoRequired` | |
| `cantidad` | positivo | `> 0` | `Validation.Movimiento.CantidadRequired` | INV-02 |
| `tipo_movimiento_id` | requerido | `!= Uuid::nil()` | `Validation.Movimiento.TipoRequired` | |
| `categoria_id` | requerido | `!= Uuid::nil()` | `Validation.Movimiento.CategoriaRequired` | **[NUEVO]** INV-03: la FK es `NOT NULL` y `RESTRICT`, el validador no lo cubría |
| `unidad` | longitud | `<= 20` | `Validation.Movimiento.UnidadMaxLength` | **[NUEVO]** |
| `cotizacion_aplicada` | requerido | requerido si `moneda == Usd`; `> 0` | `Validation.Movimiento.CotizacionRequired` | **[NUEVO]** doc 04 §2 |
| `cotizacion_aplicada` | vacío | debe ser `None` si `moneda == Ars` | `Validation.Movimiento.CotizacionNotApplicable` | **[NUEVO]** |
| `observaciones` | longitud | `<= 1000` | `Validation.Movimiento.ObservacionesMaxLength` | **[NUEVO]** |
| `fecha` | rango | `>= 2000-01-01` y `<= hoy + 1 año` | `Validation.Common.FechaOutOfRange` | **[NUEVO]** ataja el `01/01/0001` por tipeo |

El límite superior de `fecha` sale de configuración (`Validation.FechaFuturaMaxDias`, default 365).

### V-02 `CategoriaDto`

| Campo | Regla | Condición | Clave i18n | Cambio |
| --- | --- | --- | --- | --- |
| `nombre` | requerido | no vacío | `Validation.Categoria.NombreRequired` | |
| `nombre` | longitud | `<= 100` | `Validation.Categoria.NombreMaxLength` | |
| `color` | formato | `^#[0-9A-F]{6}$` si presente | `Validation.Categoria.ColorInvalid` | **[NUEVO]** |
| `icono` | longitud | `<= 50` | `Validation.Categoria.IconoMaxLength` | **[NUEVO]** |
| `categoria_padre_id` | distinto de sí misma | `!= self.id` | `Validation.Categoria.PadreCiclico` | **[NUEVO]** la jerarquía es nueva (RC-04) |

La detección de ciclos de profundidad > 1 (A → B → A) necesita consultar la base: es regla de
negocio, §5.2.

### V-03 `TipoMovimientoDto`

| Campo | Regla | Condición | Clave i18n | Cambio |
| --- | --- | --- | --- | --- |
| `nombre` | requerido | no vacío | `Validation.TipoMovimiento.NombreRequired` | **[FIX]** antes usaba el mensaje por defecto de FluentValidation, sin i18n |
| `nombre` | longitud | `<= 100` | `Validation.TipoMovimiento.NombreMaxLength` | **[FIX]** ídem |
| `color` | formato | `^#[0-9A-F]{6}$` si presente | `Validation.TipoMovimiento.ColorInvalid` | **[NUEVO]** |
| `icono` | longitud | `<= 50` | `Validation.TipoMovimiento.IconoMaxLength` | **[NUEVO]** |

### V-04 `ClienteDto`

| Campo | Regla | Condición | Clave i18n | Cambio |
| --- | --- | --- | --- | --- |
| `nombre` | requerido | no vacío | `Validation.Cliente.NombreRequired` | |
| `nombre` | longitud | `<= 200` | `Validation.Cliente.NombreMaxLength` | **[FIX]** el validador decía 100, la columna 200 |
| `email` | formato | email válido si presente | `Validation.Cliente.EmailInvalid` | |
| `email` | longitud | `<= 254` | `Validation.Cliente.EmailMaxLength` | **[NUEVO]** |
| `cuit` | formato | `^\d{2}-\d{8}-\d{1}$` si presente | `Validation.Cliente.CuitInvalid` | |
| `telefono` | longitud | `<= 30` | `Validation.Cliente.TelefonoMaxLength` | **[NUEVO]** |
| `direccion` | longitud | `<= 500` | `Validation.Cliente.DireccionMaxLength` | **[NUEVO]** |
| `contactos[i]` | anidado | cada uno con V-05, prefijo `contactos[i]` | — | |
| `contactos` | un solo principal | a lo sumo un contacto con `es_principal == true` | `Validation.Cliente.ContactoPrincipalUnico` | **[NUEVO]** RC-13 |

El mensaje de `NombreMaxLength` interpola `{max}`; no se escribe el número en la traducción.

### V-05 `ClienteContactoDto`

| Campo | Regla | Condición | Clave i18n | Cambio |
| --- | --- | --- | --- | --- |
| `etiqueta` | requerido | no vacío | `Validation.Cliente.ContactoEtiquetaRequired` | |
| `etiqueta` | longitud | `<= 100` | `Validation.Cliente.ContactoEtiquetaMaxLength` | **[NUEVO]** |
| `email` | formato | email válido si presente | `Validation.Cliente.ContactoEmailInvalid` | |
| `nombre` | longitud | `<= 200` | `Validation.Cliente.ContactoNombreMaxLength` | **[NUEVO]** |
| `telefono` | longitud | `<= 30` | `Validation.Cliente.ContactoTelefonoMaxLength` | **[NUEVO]** |
| — | al menos un dato de contacto | `email.is_some() \|\| telefono.is_some()` | `Validation.Cliente.ContactoDatoRequerido` | **[NUEVO]** un contacto sin email ni teléfono no sirve para nada |

### V-06 `ObraDto`

| Campo | Regla | Condición | Clave i18n | Cambio |
| --- | --- | --- | --- | --- |
| `nombre` | requerido | no vacío | `Validation.Obra.NombreRequired` | |
| `nombre` | longitud | `<= 200` | `Validation.Obra.NombreMaxLength` | |
| `cliente_id` | requerido | `!= Uuid::nil()` | `Validation.Obra.ClienteRequired` | |
| `numero` | positivo | `> 0` | `Validation.Obra.NumeroRequired` | |
| `direccion` | longitud | `<= 500` | `Validation.Obra.DireccionMaxLength` | **[NUEVO]** |
| `fecha_fin` | orden | `fecha_fin >= fecha_inicio` si presente | `Validation.Obra.FechaFinInvalid` | **[NUEVO]** |
| `presupuesto` | no negativo | `>= 0` | `Validation.Obra.PresupuestoNegative` | **[NUEVO]** |
| `observaciones` | longitud | `<= 1000` | `Validation.Obra.ObservacionesMaxLength` | **[NUEVO]** |

Las cuatro claves originales de `Obra` **no tenían traducción** en `es.json`: se agregan (§6).
La unicidad global de `numero` (INV-06) se verifica contra la base, §5.1.

### V-07 `TrabajoDto`

| Campo | Regla | Condición | Clave i18n | Cambio |
| --- | --- | --- | --- | --- |
| `descripcion` | requerido | no vacío | `Validation.Trabajo.DescripcionRequired` | |
| `descripcion` | longitud | `<= 500` | `Validation.Trabajo.DescripcionMaxLength` | **[FIX]** el validador decía 200, la columna 500 |
| `obra_id` | requerido | `!= Uuid::nil()` | `Validation.Trabajo.ObraRequired` | **[FIX]** la clave existía en el código pero **no** en `es.json`; había una `Validation.Trabajo.ClienteRequired` huérfana que nadie usaba |
| `fecha_fin` | orden | `fecha_fin >= fecha_inicio` si presente | `Validation.Trabajo.FechaFinInvalid` | **[NUEVO]** |
| `presupuesto` | no negativo | `>= 0` | `Validation.Trabajo.PresupuestoNegative` | **[NUEVO]** |
| `ordenes_trabajo[i]` | anidado | cada una con V-08 | — | |

### V-08 `OrdenTrabajoDto`

| Campo | Regla | Condición | Clave i18n | Cambio |
| --- | --- | --- | --- | --- |
| `titulo` | requerido | no vacío | `Validation.OrdenTrabajo.TituloRequired` | |
| `titulo` | longitud | `<= 200` | `Validation.OrdenTrabajo.TituloMaxLength` | |
| `numero` | longitud | `<= 50` | `Validation.OrdenTrabajo.NumeroMaxLength` | **[NUEVO]** |
| `trabajo_id` | requerido | `!= Uuid::nil()` | `Validation.OrdenTrabajo.TrabajoRequired` | **[NUEVO]** |
| `items` | no vacía | `!items.is_empty()` | `Validation.OrdenTrabajo.ItemsRequired` | **[NUEVO]** una orden sin ítems no certifica nada |
| `items[i]` | anidado | cada uno con V-09 | — | |
| `observaciones` | longitud | `<= 1000` | `Validation.OrdenTrabajo.ObservacionesMaxLength` | **[NUEVO]** |

### V-09 `OrdenTrabajoItemDto`

| Campo | Regla | Condición | Clave i18n | Cambio |
| --- | --- | --- | --- | --- |
| `descripcion` | requerido | no vacío | `Validation.OrdenTrabajoItem.DescripcionRequired` | |
| `descripcion` | longitud | `<= 500` | `Validation.OrdenTrabajoItem.DescripcionMaxLength` | **[NUEVO]** |
| `cantidad` | positivo | `> 0` | `Validation.OrdenTrabajoItem.CantidadRequired` | |
| `precio_unitario` | no negativo | `>= 0` | `Validation.OrdenTrabajoItem.PrecioNegative` | |
| `porcentaje_actual` | rango | `0 <= x <= 100` | `Validation.OrdenTrabajoItem.PorcentajeInvalid` | |
| `porcentaje_anterior` | rango | `0 <= x <= 100` | `Validation.OrdenTrabajoItem.PorcentajeInvalid` | |
| `porcentaje_anterior + porcentaje_actual` | rango | `<= 100` | `Validation.OrdenTrabajoItem.PorcentajeAcumuladoInvalid` | **[NUEVO]** **el hueco más grave del sistema anterior**: se podía certificar 200 % |
| `unidad` | longitud | `<= 20` | `Validation.OrdenTrabajoItem.UnidadMaxLength` | **[NUEVO]** |

La suma se compara sobre valores **escalados** (`i64`), sin convertir a `f64`:

```rust
const CIEN: i64 = 100 * Decimal4::SCALE; // 1_000_000
if item.porcentaje_anterior.raw() + item.porcentaje_actual.raw() > CIEN {
    errs.push("porcentajeActual", keys::ORDEN_ITEM_PORCENTAJE_ACUMULADO_INVALID);
}
```

El error se reporta en `porcentajeActual` (el campo que el usuario está editando), no en
`porcentajeAnterior` (que es histórico y de sólo lectura). Esto satisface INV-08.

### V-10 `CertificadoDto` **[NUEVO]**

La entidad es nueva (RC-10), así que el validador también.

| Campo | Regla | Condición | Clave i18n |
| --- | --- | --- | --- |
| `orden_trabajo_id` | requerido | `!= Uuid::nil()` | `Validation.Certificado.OrdenTrabajoRequired` |
| `numero` | positivo | `> 0` | `Validation.Certificado.NumeroRequired` |
| `fecha` | rango | `>= 2000-01-01` y `<= hoy + 1 año` | `Validation.Common.FechaOutOfRange` |
| `items` | no vacía | `!items.is_empty()` | `Validation.Certificado.ItemsRequired` |
| `items[i].porcentaje_certificado` | rango | `0 <= x <= 100` | `Validation.Certificado.PorcentajeInvalid` |
| `ajuste_uocra` | rango | `-100 <= x <= 100` | `Validation.Certificado.AjusteInvalid` |
| `otros_descuentos` | no negativo | `>= 0` | `Validation.Certificado.DescuentoNegative` |
| `observaciones` | longitud | `<= 1000` | `Validation.Certificado.ObservacionesMaxLength` |

La secuencialidad de `numero` dentro de la orden (INV-15) y que el acumulado histórico no pase de
100 se verifican contra la base, §5.3.

### V-11 `FacturaDto`

| Campo | Regla | Condición | Clave i18n | Cambio |
| --- | --- | --- | --- | --- |
| `numero` | requerido | no vacío | `Validation.Factura.NumeroRequired` | |
| `numero` | longitud | `<= 50` | `Validation.Factura.NumeroMaxLength` | |
| `cliente_id` | requerido | `!= Uuid::nil()` | `Validation.Factura.ClienteRequired` | |
| `subtotal` | no negativo | `>= 0` | `Validation.Factura.SubtotalInvalid` | |
| `iva` | no negativo | `>= 0` | `Validation.Factura.IvaInvalid` | |
| `total` | coherencia | `total == subtotal + iva` | `Validation.Factura.TotalMismatch` | **[FIX]** antes sólo se validaba `total >= 0`, lo que permitía guardar un total que no cerraba con sus partes |
| `fecha_vencimiento` | orden | `>= fecha` si presente | `Validation.Factura.FechaVencimientoInvalid` | **[NUEVO]** INV-16 |
| `observaciones` | longitud | `<= 1000` | `Validation.Factura.ObservacionesMaxLength` | **[NUEVO]** |

`Validation.Factura.TotalInvalid` (`total >= 0`) se **elimina**: queda cubierta por `TotalMismatch`,
ya que `subtotal >= 0` y `iva >= 0` implican `total >= 0`. La clave se retira de los locales.

Nota: el `total` llega en el DTO porque el formulario lo muestra, pero el caso de uso lo
**recalcula** antes de persistir (doc 06 §4.1). La validación de coherencia existe para detectar un
frontend desincronizado, no para confiar en el valor recibido.

### V-12 `PagoFacturaDto`

| Campo | Regla | Condición | Clave i18n | Cambio |
| --- | --- | --- | --- | --- |
| `factura_id` | requerido | `!= Uuid::nil()` | `Validation.PagoFactura.FacturaRequired` | |
| `monto` | positivo | `> 0` | `Validation.PagoFactura.MontoRequired` | |
| `medio_pago` | requerido | no vacío | `Validation.PagoFactura.MedioPagoRequired` | |
| `medio_pago` | longitud | `<= 100` | `Validation.PagoFactura.MedioPagoMaxLength` | |
| `fecha` | rango | `>= 2000-01-01` y `<= hoy + 1 año` | `Validation.Common.FechaOutOfRange` | **[NUEVO]** |
| `observaciones` | longitud | `<= 1000` | `Validation.PagoFactura.ObservacionesMaxLength` | **[NUEVO]** |

Las cuatro claves originales **no tenían traducción**: se agregan (§6). Que el pago no exceda el
saldo (INV-09) se verifica contra la base, §5.4.

### V-13 `EmpleadoDto`

| Campo | Regla | Condición | Clave i18n | Cambio |
| --- | --- | --- | --- | --- |
| `nombre` | requerido | no vacío | `Validation.Empleado.NombreRequired` | |
| `nombre` | longitud | `<= 200` | `Validation.Empleado.NombreMaxLength` | **[FIX]** el validador decía 100, la columna 200 |
| `dni` | formato | `7..=9` dígitos **si está presente** | `Validation.Empleado.DniLength` | **[FIX]** ver nota |
| `dni` | sólo dígitos | `^\d+$` tras normalizar | `Validation.Empleado.DniFormat` | **[NUEVO]** |
| `tarifa_diaria` | no negativo | `>= 0` | `Validation.Empleado.TarifaNegative` | |
| `sueldo_base` | no negativo | `>= 0` | `Validation.Empleado.SueldoNegative` | **[NUEVO]** |
| `multiplicador_sabado` | no negativo | `>= 0` | `Validation.Empleado.MultiplicadorNegative` | **[NUEVO]** |
| `multiplicador_domingo` | no negativo | `>= 0` | `Validation.Empleado.MultiplicadorNegative` | **[NUEVO]** |
| `multiplicador_feriado` | no negativo | `>= 0` | `Validation.Empleado.MultiplicadorNegative` | **[NUEVO]** |
| `telefono` | longitud | `<= 30` | `Validation.Empleado.TelefonoMaxLength` | **[NUEVO]** |
| `fecha_egreso` | orden | `>= fecha_ingreso` si presente | `Validation.Empleado.FechaEgresoInvalid` | **[NUEVO]** |

**[FIX] `dni`:** el validador anterior lo marcaba `NotEmpty` mientras la columna es **nullable**.
Resultado: no se podía dar de alta un peón sin DNI a mano, aunque el modelo lo permitiera. Se
resuelve haciéndolo **opcional**: si viene, tiene que ser 7 a 9 dígitos. La clave
`Validation.Empleado.DniRequired` se elimina de los locales.

Los tres multiplicadores admiten `0` (no se paga recargo) y valores `< 1` (se paga menos). No se
impone un techo: el negocio puede pagar doble o triple.

### V-14 `AsistenciaEmpleadoDto`

| Campo | Regla | Condición | Clave i18n | Cambio |
| --- | --- | --- | --- | --- |
| `empleado_id` | requerido | `!= Uuid::nil()` | `Validation.Asistencia.EmpleadoRequired` | |
| `fecha` | requerido | fecha válida, no el default | `Validation.Asistencia.FechaRequired` | |
| `fecha` | rango | `>= 2000-01-01` y `<= hoy + 1 año` | `Validation.Common.FechaOutOfRange` | **[NUEVO]** |
| `obra_id` | opcional | sin regla de forma | — | |
| `observaciones` | longitud | `<= 1000` | `Validation.Asistencia.ObservacionesMaxLength` | **[NUEVO]** |

Las dos claves originales **no tenían traducción**: se agregan (§6). La unicidad
`(empleado_id, fecha)` (INV-07) la garantiza el índice y la resuelve el `upsert`, no el validador.

### V-15 `LiquidacionDto`

| Campo | Regla | Condición | Clave i18n | Cambio |
| --- | --- | --- | --- | --- |
| `empleado_id` | requerido | `!= Uuid::nil()` | `Validation.Liquidacion.EmpleadoRequired` | |
| `fecha_inicio` | orden | `fecha_inicio <= fecha_fin` | `Validation.Liquidacion.FechaInicioInvalid` | INV-17 |
| `dias_trabajados` | positivo | `> 0` | `Validation.Liquidacion.DiasTrabajadosRequired` | |
| `tarifa_aplicada` | positivo | `> 0` | `Validation.Liquidacion.TarifaRequired` | |
| `total_bruto` | no negativo | `>= 0` | `Validation.Liquidacion.BrutoNegative` | **[NUEVO]** |
| `total_adelantos` | no negativo | `>= 0` | `Validation.Liquidacion.AdelantosNegative` | **[NUEVO]** |
| `observaciones` | longitud | `<= 1000` | `Validation.Liquidacion.ObservacionesMaxLength` | **[NUEVO]** |

Deliberadamente **no** se valida `total_adelantos <= total_bruto`: un neto negativo es un caso real
(el empleado retiró más adelantos que lo devengado) y debe poder registrarse. Se muestra en rojo en
la UI y en el PDF, pero no bloquea.

### V-16 `LiquidacionBatchDto`

Se conserva del sistema anterior, que ya tenía la clave definida:

| Campo | Regla | Condición | Clave i18n |
| --- | --- | --- | --- |
| `liquidaciones` | no vacía | `!liquidaciones.is_empty()` | `Validation.Liquidacion.BatchEmpty` |
| `liquidaciones[i]` | anidado | cada una con V-15 | — |

> El plan hablaba de «14 validadores». La cuenta final es **16**: se agregan `CertificadoDto`
> (entidad nueva) y se cuenta `LiquidacionBatchDto` aparte porque tiene su propia clave. Ninguno se
> quitó.

## 3. Claves comunes

| Clave | Uso |
| --- | --- |
| `Validation.Common.EntityNotFound` | el `id` recibido no existe o está borrado lógicamente |
| `Validation.Common.SaveFailed` | falla genérica de persistencia |
| `Validation.Common.FechaOutOfRange` | fecha fuera de `[2000-01-01, hoy + N días]` |
| `Validation.Common.ConcurrencyConflict` | **[NUEVO]** `row_version` no coincide (doc 04 §5) |
| `Validation.Common.RequiredField` | fallback cuando un campo genérico está vacío |

## 4. Inventario de longitudes

Fuente única: `eo_domain::constants::limites` (doc 05 §4). **Ningún validador escribe un número
literal**; todos referencian la constante. Esta tabla existe para verificar que validador y columna
coinciden — la desincronización de los tres `[FIX]` de arriba nació justamente de tener el número
escrito en dos lugares.

| Constante | Valor | Columnas que la usan |
| --- | --- | --- |
| `NOMBRE_CORTO` | 100 | `categorias.nombre`, `tipos_movimiento.nombre`, `tipos_concepto_pago.nombre`, `cliente_contactos.etiqueta` |
| `NOMBRE_LARGO` | 200 | `clientes.nombre`, `empleados.nombre`, `obras.nombre`, `ordenes_trabajo.titulo`, `cliente_contactos.nombre` |
| `CONCEPTO` | 500 | `movimientos.concepto` |
| `DESCRIPCION` | 500 | `trabajos.descripcion`, `orden_trabajo_items.descripcion`, `certificado_items.descripcion` |
| `DIRECCION` | 500 | `clientes.direccion`, `obras.direccion` |
| `OBSERVACIONES` | 1000 | todas las `observaciones` |
| `EMAIL` | 254 | `clientes.email`, `cliente_contactos.email` |
| `TELEFONO` | 30 | `clientes.telefono`, `empleados.telefono`, `cliente_contactos.telefono` |
| `CUIT` | 13 | `clientes.cuit` |
| `DNI` | 15 | `empleados.dni` (el validador exige 7–9 dígitos) |
| `UNIDAD` | 20 | `movimientos.unidad`, `orden_trabajo_items.unidad` |
| `COLOR_HEX` | 7 | `categorias.color`, `tipos_movimiento.color` |
| `ICONO` | 50 | `categorias.icono`, `tipos_movimiento.icono` |
| `NUMERO_FACTURA` | 50 | `facturas.numero` |
| `NUMERO_CERTIFICADO` | 50 | `ordenes_trabajo.numero` |
| `MEDIO_PAGO` | 100 | `pagos_factura.medio_pago` |

## 5. Reglas de negocio con acceso a datos

Estas **no** van en `Validate`: necesitan consultar la base. Viven en el caso de uso, después de la
validación de forma, y devuelven `AppError::Conflict { key, params }` con la misma forma de clave
i18n. Cada una se prueba con un test de integración contra SQLite en memoria.

### 5.1 Número de obra único (INV-06)

```
al crear:    existe otra obra no borrada con numero = X            → Conflict
al editar:   existe otra obra no borrada con numero = X y id != Y  → Conflict
clave:       Validation.Obra.NumeroDuplicado   params: { numero }
```

El índice `UNIQUE` de la base es la red de seguridad; esta verificación existe para dar un mensaje
legible en vez de un error de constraint.

### 5.2 Ciclo de categorías (V-02)

```
recorrer la cadena de padres desde categoria_padre_id hacia arriba;
si se llega a self.id  → Conflict
si la profundidad supera Business.CategoriaProfundidadMaxima (default 3) → Conflict
claves: Validation.Categoria.PadreCiclico / Validation.Categoria.ProfundidadExcedida
```

### 5.3 Certificación acumulada (INV-08, INV-15)

```
para cada item del certificado:
    acumulado = SUM(porcentaje_certificado) de certificados anteriores del mismo orden_trabajo_item
    si acumulado + porcentaje_certificado > 100  → Conflict
    clave: Validation.Certificado.AcumuladoExcedido   params: { item, acumulado }

numero del certificado:
    debe ser MAX(numero) + 1 dentro de la orden de trabajo, o 1 si no hay ninguno
    clave: Validation.Certificado.NumeroNoSecuencial   params: { esperado }
```

La suma se calcula sobre `i64` escalados. Comparar contra `100` significa comparar contra
`100 * 10_000 = 1_000_000`.

### 5.4 Pago no excede el saldo (INV-09)

```
saldo = factura.total - SUM(pagos no borrados de la factura)   [excluyendo el pago que se edita]
si monto > saldo  → Conflict
clave: Validation.PagoFactura.ExcedeSaldo   params: { saldo }
```

**[BUG-LEGADO]** El sistema anterior no tenía esta verificación: se podía imputar 10.000.000 a una
factura de 1.000. Ver doc 06 §4.3.

### 5.5 Adelanto no se descuenta dos veces (INV-05)

```
al confirmar una liquidación, para cada movimiento de adelanto incluido:
    si ya existe una fila en liquidacion_adelantos con ese movimiento_id
       y su liquidacion_id no está borrada  → Conflict
    clave: Validation.Liquidacion.AdelantoYaDescontado   params: { concepto, fecha }
```

Esta es la razón de existir de la tabla `liquidacion_adelantos` (doc 03). El sistema anterior sumaba
adelantos por rango de fechas sin dejar rastro de qué liquidación los consumió, así que dos
liquidaciones con rangos solapados descontaban el mismo adelanto dos veces.

### 5.6 Tipos de sistema protegidos (INV-04)

```
si tipo_movimiento.id ∈ tipos_movimiento::TODOS:
    no se puede borrar               → Conflict  Validation.TipoMovimiento.EsSistema
    no se puede cambiar es_ingreso   → Conflict  Validation.TipoMovimiento.EsIngresoInmutable
    el nombre sí se puede editar
```

Lo mismo para `tipos_concepto_pago::TODOS`, con `Validation.TipoConceptoPago.EsSistema`.

### 5.7 Borrado con dependencias (INV-11)

Antes de un borrado lógico se cuentan las referencias. Si hay alguna, `Conflict` con la clave
correspondiente y el conteo como parámetro:

| Entidad | Se bloquea si tiene | Clave |
| --- | --- | --- |
| `Cliente` | obras o facturas no borradas | `Validation.Cliente.TieneDependencias` |
| `Obra` | trabajos o movimientos no borrados | `Validation.Obra.TieneDependencias` |
| `TipoMovimiento` | movimientos no borrados | `Validation.TipoMovimiento.TieneDependencias` |
| `Categoria` | movimientos o subcategorías no borradas | `Validation.Categoria.TieneDependencias` |
| `Empleado` | liquidaciones no borradas | `Validation.Empleado.TieneDependencias` |
| `Trabajo` | órdenes de trabajo no borradas | `Validation.Trabajo.TieneDependencias` |
| `OrdenTrabajo` | certificados no borrados | `Validation.OrdenTrabajo.TieneDependencias` |

Todas llevan `params: { count }`.

### 5.8 Conflicto de concurrencia

Toda actualización compara el `row_version` recibido contra el almacenado. Si difiere,
`AppError::Conflict { key: "Validation.Common.ConcurrencyConflict" }`. Detalle en doc 04 §5.

### 5.9 Existencia de las FK

Antes de persistir se verifica que cada `Uuid` de FK apunte a una fila existente y **no borrada**.
Un `cliente_id` que apunta a un cliente con `is_deleted = 1` es un `Validation.Common.EntityNotFound`
con `params: { entity: "Cliente" }`, no un error de base.

## 6. Claves i18n a agregar

El sistema anterior tenía 40 claves de validación en el código y **28** traducidas. Estas faltaban
por completo y hay que crearlas en `es.json` y `en.json`:

| Grupo | Claves faltantes en el sistema anterior |
| --- | --- |
| `Validation.Trabajo` | `ObraRequired` (existía en el código, sin traducción) |
| `Validation.Obra` | `NombreRequired`, `NombreMaxLength`, `ClienteRequired`, `NumeroRequired` |
| `Validation.Asistencia` | `EmpleadoRequired`, `FechaRequired` |
| `Validation.PagoFactura` | `FacturaRequired`, `MontoRequired`, `MedioPagoRequired`, `MedioPagoMaxLength` |
| `Validation.TipoMovimiento` | todas (el validador usaba los mensajes por defecto en inglés) |

Claves a **eliminar** de los locales:

| Clave | Motivo |
| --- | --- |
| `Validation.Trabajo.ClienteRequired` | huérfana: traducida pero nunca usada; el trabajo cuelga de la obra, no del cliente |
| `Validation.Factura.TotalInvalid` | reemplazada por `TotalMismatch` |
| `Validation.Empleado.DniRequired` | el DNI pasa a ser opcional (V-13) |

Más las **[NUEVO]** de cada tabla de §2. El árbol completo y definitivo de claves está en
[`14-configuracion-e-i18n.md`](./14-configuracion-e-i18n.md) §4; este documento es la fuente de las
ramas `Validation.*`.

Textos en español de las nuevas claves más relevantes (el resto sigue el mismo tono, imperativo y
sin tecnicismos):

```json
{
  "Validation": {
    "Common": {
      "FechaOutOfRange": "La fecha debe estar entre el {min} y el {max}.",
      "ConcurrencyConflict": "Otro usuario modificó este registro. Recargá y volvé a intentar."
    },
    "Obra": {
      "NombreRequired": "El nombre de la obra es obligatorio.",
      "NombreMaxLength": "El nombre no puede exceder los {max} caracteres.",
      "ClienteRequired": "Debe seleccionar un cliente.",
      "NumeroRequired": "El número de obra debe ser mayor a cero.",
      "NumeroDuplicado": "Ya existe una obra con el número {numero}."
    },
    "OrdenTrabajoItem": {
      "PorcentajeAcumuladoInvalid": "El porcentaje anterior más el actual no puede superar el 100 %."
    },
    "PagoFactura": {
      "ExcedeSaldo": "El pago supera el saldo pendiente de la factura ({saldo})."
    },
    "Liquidacion": {
      "AdelantoYaDescontado": "El adelanto «{concepto}» del {fecha} ya fue descontado en otra liquidación."
    }
  }
}
```

## 7. Reglas para el implementador

1. Un validador **nunca** hace I/O, **nunca** es `async`, **nunca** toca el repositorio. Si necesita
   la base, no es un validador: es una regla de negocio de §5.
2. Un validador **nunca** contiene un mensaje literal, sólo claves de `validation::keys`.
3. Un validador **nunca** escribe un número de longitud literal: usa `limites::*`.
4. Se acumulan todos los errores de forma; no se retorna en el primer fallo.
5. Los índices de las colecciones anidadas se propagan con `nested("items[i]")`, en **camelCase**,
   para que el frontend pueda mapear el error al input exacto.
6. La misma validación **se replica** en el frontend para dar feedback inmediato, pero el backend es
   la única autoridad. El frontend usa las mismas claves i18n para no duplicar mensajes.
7. Toda clave nueva se agrega a `es.json` y `en.json` en el mismo commit. El test
   `locales_have_no_missing_keys` (doc 17) falla si una clave usada en Rust no existe en ambos
   archivos.

## 8. Tests obligatorios

Por cada validador, en `crates/eo-application/tests/validation/`:

| Test | Qué verifica |
| --- | --- |
| `<dto>_valido_pasa` | un DTO completo y correcto no produce errores |
| `<dto>_campos_requeridos_fallan` | un DTO vacío produce **exactamente** el conjunto esperado de claves |
| `<dto>_longitudes_al_limite_pasan` | `max_len` con exactamente `max` caracteres pasa; con `max + 1` falla |
| `<dto>_acumula_todos_los_errores` | un DTO con 3 errores devuelve 3 `FieldError`, no 1 |
| `<dto>_normaliza_antes_de_validar` | `"  "` en un campo requerido falla; `"  hola  "` pasa y queda `"hola"` |
| `<dto>_anidado_prefija_campos` | el error del ítem 2 llega como `items[2].descripcion` |

Casos límite específicos, cada uno con su propio test:

- `porcentaje_anterior = 60`, `porcentaje_actual = 40` → válido (suma exacta 100).
- `porcentaje_anterior = 60`, `porcentaje_actual = 40.0001` → inválido.
- Email con mayúsculas → normalizado a minúsculas y válido.
- CUIT sin guiones → inválido (el formato exige `XX-XXXXXXXX-X`).
- DNI ausente → válido; DNI de 6 dígitos → inválido; DNI con puntos → normalizado y válido.
- `total = subtotal + iva` con centavos que no cierran por redondeo → inválido, con el valor
  esperado en los params.
- Empleado con los tres multiplicadores en `0` → válido.
- Liquidación con `total_adelantos > total_bruto` → válido (neto negativo permitido).
