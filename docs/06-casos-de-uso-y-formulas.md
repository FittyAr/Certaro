# 06 — Casos de uso y fórmulas

> **El documento más peligroso de equivocar.** Toda fórmula está transcrita como expresión, con su
> orden de operaciones y su redondeo. Ninguna se describe en prosa. Si una fórmula del código no
> coincide carácter por carácter con la de acá, el test correspondiente debe fallar.
>
> Prerrequisitos: [`04-dinero-fechas-y-tipos.md`](./04-dinero-fechas-y-tipos.md) (tipos `Money` y
> `Decimal4`, redondeo half-away-from-zero) y
> [`05-dominio-entidades.md`](./05-dominio-entidades.md) (entidades y enums).

## 1. Convenciones de este documento

- `M(x)` = `Money`, `D(x)` = `Decimal4`. `x.raw()` es el entero escalado.
- `⊗` = `checked_mul` (multiplicación con reescalado y redondeo half-away-from-zero).
- `⊕` / `⊖` = `checked_add` / `checked_sub`.
- `pct(p)` = `p.as_fraction()`, es decir `p / 100`, en `Decimal4`.
- «congelar» = copiar el valor en la fila persistida para que no cambie si la fuente cambia después.
- Todo caso de uso devuelve `AppResult<T>`; los errores están en
  [`02-arquitectura.md`](./02-arquitectura.md) §6.

## 2. Estructura común de los casos de uso CRUD

**[LEGADO]** Todos los servicios del sistema anterior heredaban de un `BaseCrudService<TEntity,
TDto>` con esta secuencia. Se reproduce como plantilla porque la mayoría de los 60+ casos de uso son
exactamente esto.

### 2.1 `create`

1. Validar el DTO. Si falla, registrar `warn` con la lista de errores y devolver
   `AppError::Validation`.
2. Verificar precondiciones que necesitan la base (existencia de FK, unicidad).
3. Registrar `info`: «creando `<Entidad>`».
4. Construir la entidad: `id = IdGenerator::new_id()`, `audit = Audit::new(Clock::now_utc())`.
5. Insertar.
6. Si el insert afecta 0 filas, devolver `AppError::Persistence`.
7. Registrar `info` con el `id` resultante.
8. Devolver el DTO de salida.

### 2.2 `update`

1. Validar el DTO.
2. Leer la entidad por `id`. Si no existe → `AppError::NotFound`.
3. Aplicar los cambios campo por campo (nunca un `Adapt` ciego que pise `id`, `created_at` o
   `row_version`).
4. `audit.touch(now)` → setea `updated_at` e incrementa `row_version`.
5. `UPDATE … WHERE id = ? AND row_version = ? AND is_deleted = 0`.
6. 0 filas afectadas → `AppError::Concurrency`.

**[BUG-LEGADO]** El sistema anterior hacía `dto.Adapt(entity)` y confiaba en Mapster para no pisar
campos de auditoría, y no comparaba `row_version` en el `WHERE`: dos ediciones simultáneas se
sobreescribían en silencio.

### 2.3 `delete` (borrado lógico)

1. Leer la entidad. Si no existe → `AppError::NotFound`.
2. **Verificar dependencias vivas.** Si hay, devolver `AppError::DependencyInUse` con su clave i18n.
   La matriz de dependencias está en §12.
3. `audit.soft_delete(now)`.
4. Marcar como borrados los hijos con cascada lógica (doc 04 §4.4), **en la misma transacción**.
5. `UPDATE … WHERE id = ? AND row_version = ?`.

**[BUG-LEGADO]** El sistema anterior no verificaba dependencias ni cascadeaba: borrar un cliente
dejaba sus contactos vivos y visibles en consultas que no filtraban por cliente.

### 2.4 `get_by_id` y `list`

- Siempre con el filtro `is_deleted = 0`.
- `get_by_id` devuelve `Option<T>`; el comando Tauri lo traduce a `AppError::NotFound` si es `None`.
- Los listados cargan las relaciones que la pantalla necesita y nada más, para no traer el grafo
  completo.

## 3. Movimientos

### 3.1 Total de un movimiento — INV-01

```
total = monto ⊗ cantidad
```

```rust
pub fn total(monto: Money, cantidad: Decimal4) -> Result<Money, DomainError> {
    monto.checked_mul(cantidad)
}
```

Ejemplos verificables (usar tal cual como tests):

| `monto` | `cantidad` | `total` |
| --- | --- | --- |
| `40000.0000` | `1.0000` | `40000.0000` |
| `1500.5000` | `2.0000` | `3001.0000` |
| `1200.0000` | `5.0000` | `6000.0000` |
| `333.3333` | `3.0000` | `999.9999` |
| `0.0001` | `0.5000` | `0.0001` (half-away-from-zero sobre `0.00005`) |

**Nunca** se persiste `total`. Toda agregación lo recalcula como
`SUM(monto * cantidad / 10000)` en SQL, cuidando el reescalado.

### 3.2 Agregación de ingresos y gastos de un período

El signo **no** está en el movimiento: está en `tipos_movimiento.es_ingreso`. Hay que unir siempre.

```
ingresos = Σ (m.monto ⊗ m.cantidad)  para m con tm.es_ingreso = true
gastos   = Σ (m.monto ⊗ m.cantidad)  para m con tm.es_ingreso = false
balance  = ingresos ⊖ gastos
```

SQL de referencia (el reescalado se hace en Rust sobre el `SUM` en bruto para no perder precisión
dentro de SQLite):

```sql
SELECT tm.es_ingreso,
       SUM(m.monto * m.cantidad) AS suma_bruta_e8
  FROM movimientos m
  JOIN tipos_movimiento tm ON tm.id = m.tipo_movimiento_id
 WHERE m.is_deleted = 0
   AND m.fecha >= :desde AND m.fecha <= :hasta
 GROUP BY tm.es_ingreso;
```

`suma_bruta_e8` está escalada ×10⁸ (producto de dos valores ×10⁴). En Rust:

```rust
// Reescalar de 1e8 a 1e4 con half-away-from-zero, en i128.
let total = Money::from_raw(rescale_e8_to_e4(suma_bruta_e8)?);
```

Atención: `SUM` en SQLite sobre enteros grandes puede desbordar `i64`. Si el número de filas es
grande, hacer la suma en Rust iterando, o partir la consulta. Con el volumen esperado (miles de
filas, montos de hasta 10⁷) el producto máximo por fila es ~10¹⁵ y no desborda, pero el test debe
cubrir el caso.

**[LEGADO]** El sistema anterior calculaba `Amount = m.Monto * m.Cantidad` en LINQ y agregaba en
memoria tras un `GroupBy(_ => 1)`, devolviendo `0` cuando no había filas.

### 3.3 Filtrado y paginación de movimientos

Es el **único** listado con filtrado y paginación **del lado del servidor**.

Filtros y su predicado exacto:

| Filtro | Tipo | Predicado |
| --- | --- | --- |
| `concepto` | `Option<String>` | `LOWER(concepto) LIKE '%' || LOWER(:concepto) || '%'` — sólo si no está vacío ni es sólo espacios |
| `tipo_movimiento_id` | `Option<Uuid>` | `tipo_movimiento_id = :tipo` |
| `fecha_desde` | `Option<NaiveDate>` | `fecha >= :desde` a las `00:00:00.000Z` |
| `fecha_hasta` | `Option<NaiveDate>` | `fecha <= :hasta` a las `23:59:59.999Z` |
| `monto_min` | `Option<Money>` | `monto >= :min` — compara **`monto`**, no el total |
| `monto_max` | `Option<Money>` | `monto <= :max` — compara **`monto`**, no el total |

Los filtros se combinan con `AND`. Un filtro ausente no agrega predicado.

Orden por defecto: `ORDER BY fecha DESC`. **[NUEVO]** se agrega `, id DESC` como desempate, porque
sin él el orden de dos movimientos del mismo instante es indefinido y la paginación puede repetir o
saltear filas.

Relaciones cargadas en el listado: `tipo_movimiento` y `categoria`. Nada más.

Paginación:

```
offset = (page - 1) * size          -- page es 1-based
limit  = if size == 0 { sin límite } else { size }
```

Tamaños permitidos: `10 / 30 / 50 / 100 / 0`, default **30**.

**[BUG-LEGADO]** El DTO de filtro del sistema anterior tenía `PageSize = 10` por defecto mientras la
interfaz mostraba 30 seleccionado: la primera carga traía 10 filas y el selector mentía. El default
del backend y el del frontend **deben** ser el mismo valor y salir de la misma constante
(`PageRequest::DEFAULT_SIZE`).

El debounce de 300 ms del campo de texto es responsabilidad del frontend
([`09-modulos-funcionales.md`](./09-modulos-funcionales.md) §1).

### 3.4 Crear movimiento

Además de la plantilla de §2.1:

1. Verificar que `tipo_movimiento_id` existe y no está borrado.
2. Verificar que `categoria_id`, si viene, existe y no está borrada.
3. Verificar que `cliente_id`, `trabajo_id`, `empleado_id`, `factura_id`,
   `tipo_concepto_pago_id`, si vienen, existen.
4. Si `moneda == Usd`, `cotizacion_aplicada` es obligatoria (validación V-04).
5. Registrar `info` con concepto y monto.

**[NUEVO]** Si `tipo_movimiento_id == ADELANTO`, `empleado_id` es **obligatorio**: un adelanto sin
empleado no se puede liquidar (RC-05). El sistema anterior lo permitía y generaba adelantos
huérfanos que nunca se descontaban.

### 3.5 Borrar movimiento

Precondición **[NUEVO]**: si el movimiento está vinculado a una liquidación viva a través de
`liquidacion_adelantos`, devolver `AppError::DependencyInUse` con
`Validation.Movimiento.AdelantoLiquidado`. La FK `RESTRICT` lo respalda.

## 4. Facturas y pagos

### 4.1 Total de la factura

```
total = subtotal ⊕ iva
```

Se recalcula y se **sobreescribe** en cada `create` y en cada `update`, antes de validar. El valor
que el usuario mande en `total` se ignora.

```rust
dto.total = dto.subtotal.checked_add(dto.iva)?;
```

**No hay cálculo automático de IVA.** El sistema no conoce ninguna tasa: el usuario copia el monto
de IVA del comprobante. Si en el futuro se quisiera sugerir el 21 %, sería una ayuda de la interfaz
que precarga el campo, nunca una regla del backend.

### 4.2 Total pagado y saldo pendiente

```
total_pagado    = Σ p.monto   para p en pagos vivos de la factura
saldo_pendiente = factura.total ⊖ total_pagado
```

```sql
SELECT IFNULL(SUM(monto), 0) FROM pagos_factura
 WHERE factura_id = :id AND is_deleted = 0;
```

`saldo_pendiente` puede ser **negativo** si se cargó un pago de más. No se trunca a cero: la deuda
negativa se muestra como crédito y es información útil.

### 4.3 Registrar un pago

Caso de uso `pagos_factura::create`. **Transaccional.**

1. Validar el DTO (V-11).
2. Leer la factura con sus pagos. Si no existe → `NotFound`.
3. Si `factura.estado == Anulada` → `Conflict { code: "FACTURA_ANULADA",
   message_key: "Validation.PagoFactura.FacturaAnulada" }`.
4. **[NUEVO]** Verificar INV-09:
   `total_pagado_actual ⊕ nuevo_monto <= factura.total ⊕ tolerancia`, con
   `tolerancia = Business.ToleranciaSobrepagoFactura` (default `0.0000`). Si se excede →
   `Conflict { code: "SOBREPAGO", message_key: "Validation.PagoFactura.ExcedeTotal" }`.
5. Insertar el pago.
6. **[NUEVO]** Recalcular el estado de la factura (§4.4) y actualizarla en la misma transacción.
7. Confirmar.
8. Registrar `info` con el `id` de la factura, el monto y el nuevo saldo.

**[BUG-LEGADO]** El sistema anterior insertaba el pago y **no tocaba el estado de la factura**: una
factura totalmente pagada seguía en `Emitida` para siempre, y por eso aparecía en la deuda y en el
conteo de vencidas. Es el defecto funcional más visible del sistema actual.

### 4.4 Derivación del estado de la factura **[NUEVO]**

Se ejecuta después de todo alta, baja o modificación de pago, y al abrir el listado.

```
si estado ∈ {Borrador, Anulada}        → no cambia
si saldo_pendiente <= 0                → Pagada
si no y vencida(hoy)                   → Vencida
si no                                  → Emitida
```

donde

```
vencida(hoy) = fecha_vencimiento.is_some() && hoy > fecha_vencimiento
```

Ver [`08-maquinas-de-estado.md`](./08-maquinas-de-estado.md) §2 para las transiciones permitidas.

### 4.5 Cuenta corriente de un cliente

Caso de uso `comercial::cuenta_corriente_cliente`.

1. Leer el cliente. Si no existe, devolver un resultado vacío con el `cliente_id` (**no** un error:
   así lo hacía el sistema anterior y la pantalla lo espera).
2. Traer las facturas del cliente con `estado ∈ {Emitida, Vencida}`, con sus pagos, ordenadas por
   `fecha DESC`.
3. Por cada factura calcular:

```
pagado       = Σ pagos.monto
saldo        = factura.total ⊖ pagado
dias_vencido = if saldo > 0 { max(0, (hoy - factura.fecha).dias) } else { 0 }
```

4. Descartar las filas con `saldo <= 0`.
5. `total_deuda = Σ saldo` de las filas que quedaron.

`hoy` es la fecha civil en la zona del usuario (`Clock::today_civil`).

**[BUG-LEGADO]** El servicio anterior calculaba una variable `dias` restando el umbral de 30 días y
después la descartaba, devolviendo en `DiasVencido` los días completos desde la emisión. El código
muerto se elimina: `dias_vencido` son los días transcurridos desde `factura.fecha`.

**[NUEVO]** Cuando `fecha_vencimiento` esté cargada, `dias_vencido` se calcula desde el
vencimiento, no desde la emisión:

```
dias_vencido = if saldo > 0 {
    max(0, (hoy - factura.fecha_vencimiento.unwrap_or(factura.fecha)).dias)
} else { 0 }
```

### 4.6 Antigüedad de deuda (aging)

Caso de uso `comercial::antiguedad_deuda(cliente_id: Option<Uuid>)`.

1. Traer las facturas con `estado ∈ {Emitida, Vencida}`, opcionalmente filtradas por cliente, con
   sus pagos.
2. Por cada factura:

```
saldo = factura.total ⊖ Σ pagos.monto
si saldo <= 0 → saltear
dias  = (hoy - factura.fecha).dias        -- [NUEVO]: desde fecha_vencimiento si existe
total_deuda ⊕= saldo

si      dias <= 30 → bucket_0_30   ⊕= saldo
si no y dias <= 60 → bucket_31_60  ⊕= saldo
si no y dias <= 90 → bucket_61_90  ⊕= saldo
si no              → bucket_mas_90 ⊕= saldo
```

Los cortes son **inclusivos por arriba**: exactamente 30 días cae en `0-30`, exactamente 60 en
`31-60`, exactamente 90 en `61-90`, 91 en `+90`. Los cuatro umbrales salen de configuración
(`Business.BucketsAntiguedad`, default `[30, 60, 90]`).

Invariante verificable: `total_deuda == bucket_0_30 + bucket_31_60 + bucket_61_90 + bucket_mas_90`.

## 5. Órdenes de trabajo y certificados

### 5.1 Porcentaje acumulado de un ítem

```
porcentaje_acumulado = porcentaje_anterior ⊕ porcentaje_actual
```

INV-08 **[HUECO]**: el sistema anterior **no validaba** que esto fuera `<= 100`. Se agrega la
validación V-13 (doc 07).

### 5.2 Subtotales de un ítem

```
base               = precio_unitario ⊗ cantidad
subtotal_actual    = base ⊗ pct(porcentaje_actual)
subtotal_acumulado = base ⊗ pct(porcentaje_acumulado)
```

Transcripción literal del método del sistema anterior:

```csharp
var baseAmount = Cantidad * PrecioUnitario;
return (
    baseAmount * (PorcentajeActual / 100m),
    baseAmount * (PorcentajeAcumulado / 100m)
);
```

Orden de operaciones obligatorio: **primero** el producto `cantidad × precio_unitario` con su
redondeo, **después** la multiplicación por la fracción del porcentaje. Invertirlo cambia el
resultado en el último decimal.

Ejemplo verificable, tomado del caso real que el usuario mostró en la reunión (RC-09):

| Dato | Valor |
| --- | --- |
| `descripcion` | cableado |
| `cantidad` | `4200.0000` (metros) |
| `precio_unitario` | `1000.0000` |
| `porcentaje_anterior` | `0.0000` |
| `porcentaje_actual` | `60.0000` |
| `base` | `4200000.0000` |
| `porcentaje_acumulado` | `60.0000` |
| `subtotal_actual` | `2520000.0000` |
| `subtotal_acumulado` | `2520000.0000` |

### 5.3 Porcentaje pendiente

```
porcentaje_pendiente = 100 ⊖ porcentaje_acumulado
```

Es lo que responde a RC-11: «¿por qué me quedó pendiente?». Se muestra junto a la `nota` del ítem.

### 5.4 Totales de una orden de trabajo / certificado

```
total_certificado = Σ item.subtotal_actual        para los ítems de la orden
ajuste_uocra      = total_certificado ⊗ pct(orden.ajuste_uocra_porcentaje)
total_neto        = total_certificado ⊖ ajuste_uocra ⊖ orden.otros_descuentos
```

Notas obligatorias:

- `ajuste_uocra_porcentaje` es un **porcentaje** (el `8` de la planilla real se guarda como
  `80000`); `ajuste_uocra` es el **monto** que resulta de aplicarlo.
- El ajuste UOCRA **resta**. Es un descuento sobre lo certificado.
- `otros_descuentos` ya viene como monto y también resta.
- **[HUECO]** El sistema anterior tenía las columnas `AjusteUocraPorcentaje` y `OtrosDescuentos` en
  `OrdenesTrabajo` pero **no implementaba ninguna fórmula que las usara**: sólo aparecían en el PDF
  del certificado. La fórmula de arriba es la definición nueva y normativa.
- Estos totales se pueden calcular «en vivo» sobre la orden (para la pantalla de edición) y se
  **congelan** en `certificados` al emitir.

### 5.5 Emitir un certificado **[NUEVO]** — RC-10

Caso de uso `certificados::emitir(orden_trabajo_id)`. **Transaccional.** Incrementa la
`row_version` de la orden (raíz del agregado) aunque no cambien sus campos.

1. Leer la orden con sus ítems y sus certificados. Si no existe → `NotFound`.
2. Si ningún ítem tiene `porcentaje_actual > 0` →
   `Conflict { code: "CERTIFICADO_VACIO", message_key: "Validation.Certificado.SinAvance" }`.
3. Verificar V-13 para **todos** los ítems: `porcentaje_acumulado <= 100`.
4. `numero = (max(certificados.numero de esa orden) o 0) + 1`.
5. Calcular los totales de §5.4 con los valores actuales.
6. Insertar la fila de `certificados` con los cuatro totales congelados.
7. Por cada ítem con `porcentaje_actual > 0`, insertar un `certificado_items` congelando
   `cantidad`, `precio_unitario`, `porcentaje_anterior`, `porcentaje_actual`, `subtotal_actual` y
   `subtotal_acumulado`.
8. Por cada ítem incluido, actualizar el ítem:

```
item.porcentaje_anterior = item.porcentaje_anterior ⊕ item.porcentaje_actual
item.porcentaje_actual   = 0
item.ejecutado           = item.porcentaje_anterior >= 100
```

9. Actualizar `orden.numero_certificado = numero.to_string()`.
10. `orden.audit.touch(now)`.
11. Confirmar y registrar `info` con el número de certificado y `total_neto`.

Invariante verificable después de emitir: para cada ítem,
`item.porcentaje_anterior == Σ certificado_items.porcentaje_actual` de esa orden.

### 5.6 Anular un certificado **[NUEVO]**

Sólo se puede anular el **último** certificado de la orden. Al anularlo se revierte el paso 8:

```
item.porcentaje_actual   = cert_item.porcentaje_actual
item.porcentaje_anterior = item.porcentaje_anterior ⊖ cert_item.porcentaje_actual
item.ejecutado           = item.porcentaje_anterior >= 100
```

Si se intenta anular uno que no es el último →
`Conflict { code: "CERTIFICADO_NO_ULTIMO", message_key: "Validation.Certificado.NoEsUltimo" }`.
El `numero` **no** se reutiliza (INV-15).

## 6. Liquidaciones — el algoritmo completo

Es la funcionalidad que el usuario marcó como más urgente (RC-01). Se documenta en detalle porque
tiene tres ramas y un orden de prioridad que no es obvio.

### 6.1 Configuración que consume

Seis valores que salen del servicio de configuración del usuario (doc 14, sección
`Business.Liquidacion`):

| Clave | Tipo | Default |
| --- | --- | --- |
| `Business.Liquidacion.IncluirSabados` | `bool` | `false` |
| `Business.Liquidacion.IncluirDomingos` | `bool` | `false` |
| `Business.Liquidacion.IncluirFeriados` | `bool` | `false` |
| `Business.Liquidacion.MultiplicadorSabado` | `Decimal4` | `1.0` |
| `Business.Liquidacion.MultiplicadorDomingo` | `Decimal4` | `1.0` |
| `Business.Liquidacion.MultiplicadorFeriado` | `Decimal4` | `1.0` |

Los seis se **copian** al DTO sugerido y después a la fila de `liquidaciones`: la liquidación
congela con qué reglas se calculó.

### 6.2 Tarifa diaria sugerida

```
tarifa_diaria_sugerida = empleado.sueldo_base ⊘ dias_por_periodo(empleado.pago_frecuencia)
```

con los divisores de [`05-dominio-entidades.md`](./05-dominio-entidades.md) §3.5:
`Diario = 1`, `Semanal = 6`, `Quincenal = 15`, `Mensual = 30`.

Es sólo una **sugerencia** que la interfaz precarga al crear un empleado. La liquidación usa
`empleado.tarifa_diaria`, nunca esta derivación.

### 6.3 Multiplicador de un día del calendario

Prioridad estricta: **feriado > domingo > sábado > día hábil**. Un domingo que además es feriado
usa el multiplicador de **feriado**.

```rust
fn multiplicador_dia(
    fecha: NaiveDate,
    feriados: &HashSet<NaiveDate>,
    cfg: &LiquidacionConfig,
) -> Decimal4 {
    let es_sabado  = fecha.weekday() == Weekday::Sat;
    let es_domingo = fecha.weekday() == Weekday::Sun;
    let es_feriado = feriados.contains(&fecha);

    if es_feriado {
        return if cfg.incluir_feriados { cfg.multiplicador_feriado } else { Decimal4::ZERO };
    }
    if es_domingo {
        return if cfg.incluir_domingos { cfg.multiplicador_domingo } else { Decimal4::ZERO };
    }
    if es_sabado {
        return if cfg.incluir_sabados { cfg.multiplicador_sabado } else { Decimal4::ZERO };
    }
    Decimal4::ONE
}
```

Transcripción literal del sistema anterior:

```csharp
if (esFeriado)  return incluirFeriados ? multiplicadorFeriado : 0.0m;
if (esDomingo)  return incluirDomingos ? multiplicadorDomingo : 0.0m;
if (esSabado)   return incluirSabados  ? multiplicadorSabado  : 0.0m;
return 1.0m;
```

Un multiplicador `<= 0` significa **día no computable**: no suma días ni bruto.

### 6.4 Suma de adelantos del período

```
adelantos = Σ (m.monto ⊗ m.cantidad)
            para m con  m.empleado_id        = :empleado_id
                   and  m.tipo_movimiento_id = ADELANTO      -- …0003
                   and  m.fecha             >= :inicio
                   and  m.fecha             <= :fin
                   and  m.is_deleted         = 0
```

El GUID es la constante `constants::tipos_movimiento::ADELANTO` =
`00000000-0000-0000-0000-000000000003`. La consulta usa el índice compuesto
`ix_movimientos_empleado_tipo_fecha`.

Se suma el **total** del movimiento (`monto × cantidad`), no `monto`. Transcripción literal:
`movimientos.Sum(m => m.Total)`.

**[NUEVO]** Se excluyen los adelantos que ya están vinculados a otra liquidación viva:

```
   and  m.id NOT IN (SELECT movimiento_id FROM liquidacion_adelantos WHERE is_deleted = 0)
```

Esto implementa INV-05. Sin esta cláusula, dos liquidaciones con períodos solapados descuentan el
mismo adelanto dos veces, y el empleado cobra de menos.

### 6.5 Totales de la liquidación

```
total_neto = total_bruto ⊖ total_adelantos
```

**No se persiste** `total_neto`.

### 6.6 `sugerir_liquidacion(empleado_id, inicio, fin, dias_trabajados)` — algoritmo completo

Este es el corazón de RC-01. El parámetro `dias_trabajados` actúa como **selector de rama**: si
viene en `0`, el sistema calcula; si viene con un valor, el usuario manda.

```
ENTRADA: empleado_id, inicio (fecha civil), fin (fecha civil), dias_trabajados (Decimal4)
SALIDA:  LiquidacionSugeridaDto

 1. empleado ← repo.empleados.find(empleado_id)
    si es None → AppError::NotFound { entity: "Empleado" }

 2. cfg ← leer los 6 valores de configuración de §6.1

 3. feriados ← conjunto de fechas civiles de feriado, unión de todos los años
    desde inicio.year() hasta fin.year() inclusive, obtenidos del HolidayProvider.
    Si el proveedor falla, devuelve conjunto VACÍO y se registra `warn`
    (degradación silenciosa, doc 13 §2).

 4. SI dias_trabajados == 0 ENTONCES          -- ramas automáticas
      4.1 RAMA A — desde asistencia:
          asistencias ← repo.asistencias.find(empleado_id, inicio..=fin)   -- fechas civiles

          SI asistencias NO está vacía ENTONCES
              total_dias  ← 0
              total_bruto ← M(0)
              PARA CADA a EN asistencias:
                  factor ← a.tipo_jornada.factor()
                  SI factor <= 0 → continuar          -- Falta y FaltaJustificada quedan fuera

                  SI a.tipo_jornada == Feriado ENTONCES
                      mult ← si cfg.incluir_feriados { cfg.multiplicador_feriado } si no { 0 }
                  SI NO
                      mult ← multiplicador_dia(a.fecha, feriados, cfg)
                  FIN SI

                  SI mult <= 0 → continuar

                  total_dias  ⊕= factor
                  total_bruto ⊕= empleado.tarifa_diaria ⊗ factor ⊗ mult
              FIN PARA
              IR A 5
          FIN SI

      4.2 RAMA B — iteración de calendario (sólo si NO hay ninguna asistencia cargada):
          total_dias  ← 0
          total_bruto ← M(0)
          PARA fecha DESDE inicio HASTA fin, paso 1 día:
              mult ← multiplicador_dia(fecha, feriados, cfg)
              SI mult <= 0 → continuar
              total_dias  ⊕= 1
              total_bruto ⊕= empleado.tarifa_diaria ⊗ mult
          FIN PARA

    SI NO                                      -- RAMA C — manual
      total_dias  ← dias_trabajados
      total_bruto ← total_dias ⊗ empleado.tarifa_diaria
    FIN SI

 5. adelantos ← suma de §6.4 para (empleado_id, inicio, fin)

 6. registrar `info`: empleado, total_dias, total_bruto, adelantos, cantidad de feriados

 7. devolver LiquidacionSugeridaDto {
        empleado_id, empleado_nombre: empleado.nombre,
        fecha_inicio: inicio, fecha_fin: fin,
        dias_trabajados: total_dias,
        tarifa_aplicada: empleado.tarifa_diaria,
        total_bruto, total_adelantos: adelantos,
        total_neto: total_bruto ⊖ adelantos,
        incluir_sabados: cfg.incluir_sabados,
        incluir_domingos: cfg.incluir_domingos,
        incluir_feriados: cfg.incluir_feriados,
        multiplicador_sabado: cfg.multiplicador_sabado,
        multiplicador_domingo: cfg.multiplicador_domingo,
        multiplicador_feriado: cfg.multiplicador_feriado,
        adelantos_detalle: Vec<AdelantoDetalleDto>,   -- [NUEVO], RC-02
    }
```

Puntos donde es fácil equivocarse, en orden de gravedad:

1. **La rama A gana sobre la rama B.** Si existe **al menos un** registro de asistencia en el
   período, la rama de calendario **no se ejecuta**. No se mezclan: no se completan los días sin
   asistencia con la iteración de calendario. Es intencional y es lo que hace el sistema anterior
   (`CalcularDesdeAsistenciaAsync` devuelve `null` sólo si `registros.Count == 0`).
2. **En la rama A, un día de asistencia `Falta` o `FaltaJustificada` no suma nada**, ni siquiera
   como día. `total_dias` no cuenta las faltas.
3. **En la rama A, la asistencia de tipo `Feriado` no consulta la lista de feriados**: usa
   directamente el multiplicador de feriado según la marca `incluir_feriados`. Es decir, el usuario
   puede marcar un día cualquiera como «feriado trabajado» y se paga con el multiplicador de
   feriado. Transcripción literal:

```csharp
var multiplicador = asistencia.TipoJornada == TipoJornada.Feriado
    ? (incluirFeriados ? multiplicadorFeriado : 0.0m)
    : ObtenerMultiplicador(asistencia.Fecha, feriados, /* … */);
```

4. **En la rama A, el orden de la multiplicación es `tarifa ⊗ factor ⊗ mult`**, en ese orden
   exacto, con redondeo en cada paso. Transcripción literal:
   `totalBruto += empleado.TarifaDiaria * factor * multiplicador;`
5. **En la rama B, cada día computable suma exactamente `1.0`** a `total_dias`, sin importar el
   multiplicador. Un feriado con multiplicador `2.0` suma 1 día y `tarifa × 2` de bruto.
   Transcripción literal: `totalDias += 1.0m; totalBruto += empleado.TarifaDiaria * multiplicador;`
6. **En la rama C, el orden es `total_dias ⊗ tarifa`**, y **no** se aplica ningún multiplicador ni
   se consultan feriados: el usuario dijo cuántos días y punto. Transcripción literal:
   `totalBruto = totalDias * empleado.TarifaDiaria;`
7. **Los adelantos se suman igual en las tres ramas**, después del `if`.
8. **La comparación `dias_trabajados == 0` es exacta**, no `<= 0`. Un valor negativo entra por la
   rama C y produce un bruto negativo. **[NUEVO]** La validación V-10 lo rechaza antes.

### 6.7 Ejemplo verificable de punta a punta — el caso del usuario (RC-01)

Datos, tomados literalmente de la reunión:

| Dato | Valor |
| --- | --- |
| `empleado.tarifa_diaria` | `40000.0000` |
| `dias_trabajados` (manual, rama C) | `10.0000` |
| Adelantos del período | `30000` + `40000` + `40000` + `50000` + `100000` = `260000.0000` |

Con la aritmética del sistema:

```
total_bruto     = 10.0000 ⊗ 40000.0000 = 400000.0000
total_adelantos = 260000.0000
total_neto      = 400000.0000 ⊖ 260000.0000 = 140000.0000
```

El ejemplo que el usuario dio de memoria en la reunión («400 menos 240, total 160») usa cifras
redondeadas en miles y adelantos de 240 000; con los cinco comprobantes que enumeró da 260 000. El
test usa los **cinco adelantos enumerados**, que es el caso que hay que reproducir en el PDF.

Test obligatorio adicional, rama A, para cubrir medias jornadas y feriados:

| Día | `tipo_jornada` | `factor` | Es feriado en la API | `mult` | Aporte a días | Aporte a bruto |
| --- | --- | --- | --- | --- | --- | --- |
| lun 3 | `Completa` | `1.0` | no | `1.0` | `1.0` | `40000.0000` |
| mar 4 | `Media` | `0.5` | no | `1.0` | `0.5` | `20000.0000` |
| mié 5 | `Falta` | `0.0` | no | — | `0.0` | `0.0000` |
| jue 6 | `FaltaJustificada` | `0.0` | no | — | `0.0` | `0.0000` |
| vie 7 | `Feriado` | `1.0` | no | `2.0` (`incluir_feriados = true`, `multiplicador_feriado = 2.0`) | `1.0` | `80000.0000` |
| sáb 8 | `Completa` | `1.0` | no | `0.0` (`incluir_sabados = false`) | `0.0` | `0.0000` |
| **Total** | | | | | **`2.5000`** | **`140000.0000`** |

### 6.8 Crear la liquidación (confirmar la sugerencia)

Caso de uso `liquidaciones::create`. **Transaccional.**

1. Validar el DTO (V-10).
2. **[NUEVO]** Verificar que no exista otra liquidación viva del mismo empleado cuyo rango se
   solape con `[fecha_inicio, fecha_fin]`. Si existe →
   `Conflict { code: "PERIODO_SOLAPADO", message_key: "Validation.Liquidacion.PeriodoSolapado" }`.
   Sin esto, INV-05 se puede violar por otro camino.
3. Insertar la fila de `liquidaciones` con `total_bruto` y `total_adelantos` **congelados** tal como
   los confirmó el usuario (pudo editarlos en el paso 3 del asistente, doc 09 §3.11).
4. **[NUEVO]** Insertar una fila de `liquidacion_adelantos` por cada adelanto incluido, congelando
   `monto`, `fecha` y `concepto`. Si el índice único de `movimiento_id` falla, es que otro proceso
   liquidó ese adelanto: devolver `Conflict` con
   `Validation.Liquidacion.AdelantoYaLiquidado`.
5. Verificar que `Σ liquidacion_adelantos.monto == liquidaciones.total_adelantos`. Si no coincide es
   un error de programación: `AppError::Unexpected`.
6. Confirmar y registrar `info` con bruto, adelantos y neto.

### 6.9 Crear liquidaciones en lote

Caso de uso `liquidaciones::create_batch(Vec<LiquidacionDto>)`. **Una sola transacción para todo el
lote.**

1. Si la lista está vacía → `AppError::Validation` con
   `Validation.Liquidacion.BatchVacio`.
2. Abrir transacción.
3. Para cada DTO: validar y ejecutar los pasos de §6.8. **Ante el primer error, revertir todo el
   lote** y devolver ese error. Es todo o nada.
4. Confirmar.

**[LEGADO]** El sistema anterior ya hacía exactamente esto, con `BeginTransaction` /
`Rollback` en el primer fallo de validación.

### 6.10 Anular una liquidación

1. Borrado lógico de la fila de `liquidaciones`.
2. Borrado lógico de sus `liquidacion_adelantos` en la misma transacción. Como el índice único de
   `movimiento_id` filtra por `is_deleted`, los adelantos quedan **liberados** y se pueden volver a
   incluir en una liquidación nueva.

## 7. Rentabilidad

### 7.1 Rentabilidad por obra

Caso de uso `comercial::rentabilidad_por_obra`.

La imputación es indirecta: `movimiento → trabajo → obra`. Un movimiento sin `trabajo_id` **no** se
imputa a ninguna obra.

```
para cada obra:
    movs      = movimientos con trabajo_id ≠ NULL cuyo trabajo.obra_id = obra.id
    ingresos  = Σ (m.monto ⊗ m.cantidad)  para m con tm.es_ingreso = true
    gastos    = Σ (m.monto ⊗ m.cantidad)  para m con tm.es_ingreso = false
    rentabilidad = ingresos ⊖ gastos
    margen_porcentaje = si ingresos > 0 { round((rentabilidad ⊘ ingresos) ⊗ 100, 2) } si no { 0 }
orden: rentabilidad DESC
```

Transcripción literal del margen:

```csharp
MargenPorcentaje = Ingresos > 0
    ? Math.Round((Rentabilidad / Ingresos) * 100m, 2)
    : 0m;
```

Notas:

- El redondeo es a **2 decimales**, half-away-from-zero.
- Si `ingresos == 0` el margen es `0`, **no** es `null` ni `-100`. Aunque haya gastos.
- La condición es `> 0`, no `!= 0`: un ingreso negativo también da margen `0`.
- El orden de operaciones es: dividir primero, multiplicar por 100 después, redondear al final.

### 7.2 Ranking de obras del dashboard

Idéntico a §7.1 pero limitado a las **5** primeras por `rentabilidad DESC`. El límite sale de
configuración (`Dashboard.TopObras`, default `5`).

### 7.3 Rentabilidad por trabajo

**[HUECO]** El sistema anterior no la implementaba, aunque la documentación de negocio la promete
(«Ranking: saber qué trabajos son más rentables»). Misma fórmula que §7.1 agrupando por
`movimiento.trabajo_id` en lugar de por obra.

### 7.4 Rentabilidad por orden de trabajo — RC-17 **[HUECO]**

```
certificado = Σ certificados.total_neto de la orden
gastos      = Σ (m.monto ⊗ m.cantidad) para los movimientos de gasto del trabajo de esa orden
margen      = certificado ⊖ gastos
```

Aproximación reconocida: los gastos se imputan al **trabajo**, no a la orden, así que si un trabajo
tiene varias órdenes los gastos no se pueden repartir. Se muestra el dato con esa advertencia en la
interfaz. Repartir gastos por orden requeriría un `orden_trabajo_id` en `movimientos`, que se deja
como extensión futura.

## 8. Asistencia

### 8.1 Upsert de un día

Caso de uso `asistencias::upsert(dto)`. Es la operación que dispara el ciclo de clic de la grilla
(doc 09 §3.7).

```
1. normalizar dto.fecha a fecha civil
2. existente ← repo.asistencias.find_by_empleado_fecha(dto.empleado_id, dto.fecha)
3. si existente es None:
       validar y crear
   si no:
       dto.id ← existente.id
       validar y actualizar (con row_version)
4. devolver la fila resultante releída de la base
```

La clave del upsert es `(empleado_id, fecha)` y está respaldada por el índice único
`ux_asistencias_empleado_empleado_fecha`, que **no** filtra por `is_deleted` (doc 04 §4.3).

**[BUG-LEGADO]** El sistema anterior comparaba `a.Fecha.Date == dto.Fecha.Date` en LINQ, lo que EF
Core traduce a una función de fecha sobre la columna y **no usa el índice**. Con la normalización a
medianoche del sistema nuevo la comparación es de igualdad exacta y el índice se aprovecha.

### 8.2 Consulta de un período

```
asistencias del período = todas las filas con fecha >= inicio AND fecha <= fin
```

Ambos extremos son fechas civiles inclusivas. La grilla mensual pide
`[primer día del mes, último día del mes]` para **todos** los empleados activos de una sola vez, no
una consulta por empleado.

### 8.3 Resumen de asistencia de un empleado en un período **[NUEVO]**

```
dias_completos          = conteo de filas con tipo_jornada = Completa
dias_medios             = conteo de filas con tipo_jornada = Media
faltas                  = conteo de filas con tipo_jornada = Falta
faltas_justificadas     = conteo de filas con tipo_jornada = FaltaJustificada
feriados_trabajados     = conteo de filas con tipo_jornada = Feriado
dias_computables_total  = Σ tipo_jornada.factor()
```

Alimenta la vista de resumen por empleado de RC-06.

## 9. Dashboard

Caso de uso `dashboard::stats(periodo: PeriodoDashboard)`.

### 9.1 Rangos de período

`PeriodoDashboard` es un enum **[NUEVO]**; el sistema anterior recibía un `string` y comparaba
contra literales `"Mensual"` y `"Anual"`.

| Período | Rango actual | Rango anterior |
| --- | --- | --- |
| `Mensual` | `[ahora − 1 mes, ahora]` | `[ahora − 2 meses, ahora − 1 mes]` |
| `Anual` | `[ahora − 1 año, ahora]` | `[ahora − 2 años, ahora − 1 año]` |
| `Historico` | `[el comienzo de los tiempos, ahora]` | no hay: la comparación queda vacía |

Transcripción literal:

```csharp
"Mensual" => (now.AddMonths(-1), now),
"Anual"   => (now.AddYears(-1), now),
_         => (DateTime.MinValue, now)
```

**[BUG-LEGADO]** `now` era `DateTime.Now` (hora local) mientras los datos de auditoría estaban en
UTC. En el sistema nuevo `ahora` es `Clock::now_utc()` y los rangos se construyen en UTC.

Son **ventanas móviles**, no meses calendario: «Mensual» significa «los últimos 30/31 días», no
«agosto». Es contraintuitivo pero es lo que hace el sistema y lo que el usuario ya interpretó. Si se
quisiera cambiar a mes calendario, es una decisión de negocio a documentar acá primero.

### 9.2 KPIs

| KPI | Fórmula / consulta |
| --- | --- |
| `total_ingresos` | agregación de §3.2 en el rango actual, `es_ingreso = true` |
| `total_gastos` | ídem con `es_ingreso = false` |
| `balance` | `total_ingresos ⊖ total_gastos` — **derivado, no se persiste** |
| `rentabilidad` | `si total_ingresos > 0 { round((balance ⊘ total_ingresos) ⊗ 100, 2) } si no { 0 }` |
| `clientes_activos` | conteo de clientes con al menos un movimiento **de ingreso** en el rango actual |
| `trabajos_pendientes` | conteo de trabajos con `estado ∉ {Finalizado, Cancelado}` |
| `obras_pausadas_count` | conteo de obras con `estado = Pausada` |
| `facturas_vencidas_count` | ver §9.3 |
| `liquidaciones_pendientes` | ver §9.4 |
| `previous_periodo_ingresos` | agregación de ingresos en el rango anterior |
| `previous_periodo_gastos` | agregación de gastos en el rango anterior |
| `ingresos_change_percent` | ver §9.5 |
| `gastos_change_percent` | ver §9.5 |

Transcripción literal de `rentabilidad`:

```csharp
stats.Rentabilidad = stats.TotalIngresos > 0
    ? Math.Round((stats.Balance / stats.TotalIngresos) * 100m, 2)
    : 0m;
```

### 9.3 Facturas vencidas

```
umbral = hoy − Business.DiasVencimientoFacturaPorDefecto        -- default 30 días

facturas_vencidas = conteo de facturas donde
      estado = Vencida
   OR (estado = Emitida AND fecha <= umbral)
```

Transcripción literal:

```csharp
var overdueThreshold = DateTime.Today.AddDays(-30);
stats.FacturasVencidasCount = await _context.Facturas.CountAsync(f =>
    f.Estado == EstadoFactura.Vencida ||
    (f.Estado == EstadoFactura.Emitida && f.Fecha <= overdueThreshold));
```

- El umbral de 30 días estaba **hardcodeado** como `private const int OverdueInvoiceDays = 30`.
  **[NUEVO]** pasa a configuración.
- **[BUG-LEGADO]** El conteo **no descuenta los pagos**: una factura `Emitida` de hace 40 días
  totalmente cobrada se cuenta como vencida, porque el estado nunca se actualizó (§4.3). Con la
  derivación de estado de §4.4 el problema desaparece, pero el conteo igualmente debe agregar
  `AND saldo_pendiente > 0` como red de seguridad.
- **[NUEVO]** Cuando `fecha_vencimiento` está cargada, la condición es
  `estado = Emitida AND fecha_vencimiento < hoy AND saldo_pendiente > 0`.

### 9.4 Liquidaciones pendientes

```
liquidaciones_pendientes = conteo de empleados con activo = true
    que NO tienen ninguna liquidación cuya fecha_fin caiga
    en el mes y año actuales
```

Transcripción literal:

```csharp
var mesActual = DateTime.Now.Month;
var añoActual = DateTime.Now.Year;
stats.LiquidacionesPendientes = await _context.Empleados
    .Where(e => e.Activo)
    .CountAsync(e => !_context.Liquidaciones.Any(l =>
        l.EmpleadoId == e.Id &&
        l.FechaFin.Month == mesActual &&
        l.FechaFin.Year == añoActual));
```

Es un KPI de **mes calendario**, aunque el resto del dashboard use ventanas móviles. Es
intencional: el usuario liquida a fin de mes.

### 9.5 Variación contra el período anterior

```
change_percent(anterior, actual):
    si anterior == 0:
        si actual == 0 → 0
        si no          → NULL      -- «sin base de comparación»
    si no:
        round(((actual ⊖ anterior) ⊘ anterior) ⊗ 100, 1)
```

Transcripción literal:

```csharp
if (previous == 0m)
    return current == 0m ? 0m : null;
return Math.Round(((current - previous) / previous) * 100m, 1);
```

- Redondeo a **1 decimal** (no 2, a diferencia del margen de rentabilidad).
- `null` es un valor válido y significativo: la interfaz muestra «—», no «0 %» ni «∞».
- Si no hay rango anterior (período `Historico`), los cuatro campos de comparación quedan sin
  calcular.

### 9.6 Top 3 de clientes

```
para los movimientos del rango actual con cliente_id ≠ NULL y tm.es_ingreso = true:
    agrupar por cliente.nombre
    total = Σ (m.monto ⊗ m.cantidad)
orden: total DESC
límite: 3
```

Agrupa por **nombre**, no por `id`: dos clientes homónimos se suman juntos.
**[NUEVO]** agrupar por `cliente_id` y devolver también el `id`, para que la interfaz pueda navegar
al cliente. El límite sale de `Dashboard.TopClientes` (default `3`).

### 9.7 Movimientos recientes

Los **5** últimos movimientos por `fecha DESC`, con `tipo_movimiento`, `categoria` y `cliente`
cargados. **[NUEVO]** desempate `, id DESC`. El límite sale de
`Dashboard.MovimientosRecientes` (default `5`).

### 9.8 Series mensuales del año en curso

Dos arreglos de 12 posiciones, indexados por `mes − 1`:

```
para los movimientos con fecha.year = año_actual:
    idx = fecha.month - 1
    si tm.es_ingreso → monthly_income[idx]   ⊕= m.monto ⊗ m.cantidad
    si no            → monthly_expenses[idx] ⊕= m.monto ⊗ m.cantidad
```

**[BUG-LEGADO]** El sistema anterior acumulaba en `double[]` con un cast `(double)item.Amount`:
convertía dinero a punto flotante justo para el gráfico. **[NUEVO]** los arreglos son de `Money` y
la conversión a número para el gráfico ocurre en el frontend, en el momento de dibujar, sobre un
valor ya redondeado a 2 decimales.

«Año en curso» es el año de `Clock::now_utc()`, no del período seleccionado. Es intencional: el
gráfico anual es siempre del año calendario.

### 9.9 Gastos por categoría

```
para los movimientos con tm.es_ingreso = false y categoria_id ≠ NULL:
    agrupar por categoria.nombre
    valor = Σ (m.monto ⊗ m.cantidad)
orden: valor DESC
límite: 5
```

**Sin filtro de período**: esta consulta abarca **toda** la historia, a diferencia del resto del
dashboard. Está así en el sistema anterior. **[NUEVO]** se le aplica el rango del período
seleccionado, que es lo que el usuario espera; el cambio se documenta acá para que no parezca un
error de transcripción. Límite en `Dashboard.TopCategorias` (default `5`).

### 9.10 Salud de la base

```
database_healthy = la conexión responde a una consulta trivial (SELECT 1)
database_status  = clave i18n, no texto
```

**[BUG-LEGADO]** El sistema anterior devolvía los literales `"Saludable"` y `"Error de conexión"`
desde el backend, sin traducir. **[NUEVO]** devuelve `Dashboard.Estado.Saludable` /
`Dashboard.Estado.Error` como clave i18n.

### 9.11 Alertas del dashboard

Se generan a partir de los KPI, con estos umbrales (todos configurables, sección `Dashboard.Alertas`
del doc 14):

| Alerta | Condición | Severidad | Clave i18n |
| --- | --- | --- | --- |
| Facturas vencidas | `facturas_vencidas_count > 0` | `warning` | `Dashboard.Alerta.FacturasVencidas` |
| Balance negativo | `balance < 0` | `error` | `Dashboard.Alerta.BalanceNegativo` |
| Obras pausadas | `obras_pausadas_count > 0` | `info` | `Dashboard.Alerta.ObrasPausadas` |
| Liquidaciones pendientes | `liquidaciones_pendientes > 0` | `warning` | `Dashboard.Alerta.LiquidacionesPendientes` |
| Caída de ingresos | `ingresos_change_percent < -Dashboard.Alertas.CaidaIngresosPct` (default `20`) | `warning` | `Dashboard.Alerta.CaidaIngresos` |

Cada alerta lleva su conteo o monto como parámetro de la clave i18n.

## 10. Categorías y tipos de movimiento

### 10.1 CRUD de categorías

Plantilla de §2 más:

- Al borrar: si hay movimientos vivos con esa `categoria_id` →
  `DependencyInUse { message_key: "Validation.Categoria.EnUso" }`.
- **[NUEVO]** si tiene subcategorías vivas →
  `DependencyInUse { message_key: "Validation.Categoria.TieneHijas" }`.
- **[NUEVO]** al asignar `categoria_padre_id`: rechazar si el padre ya tiene padre (profundidad
  máxima 2) y rechazar si el padre es la propia categoría o una descendiente (ciclo).

### 10.2 CRUD de tipos de movimiento — INV-04

- Al borrar: si `es_sistema` o si el `id` está en `constants::tipos_movimiento::TODOS` →
  `Conflict { code: "TIPO_SISTEMA", message_key: "Validation.TipoMovimiento.EsSistema" }`.
- Al borrar: si hay movimientos vivos con ese tipo →
  `DependencyInUse { message_key: "Validation.TipoMovimiento.EnUso" }`.
- Al actualizar un tipo de sistema: se permite cambiar `descripcion`; se **rechaza** cambiar
  `nombre` y `es_ingreso`.

**[BUG-LEGADO]** El sistema anterior no protegía los tipos de sistema en absoluto: se podían borrar
los cuatro, y con eso quedaba inutilizable la liquidación (que depende del GUID `…0003`).

## 11. Uso de transacciones

Casos de uso que **obligatoriamente** abren una transacción:

| Caso de uso | Por qué |
| --- | --- |
| `liquidaciones::create` | `liquidaciones` + N `liquidacion_adelantos` |
| `liquidaciones::create_batch` | todo el lote es atómico |
| `liquidaciones::anular` | liquidación + sus adelantos |
| `certificados::emitir` | `certificados` + N `certificado_items` + N ítems + la orden |
| `certificados::anular` | revierte todo lo anterior |
| `pagos_factura::create` / `update` / `delete` | pago + recálculo de estado de la factura |
| `ordenes_trabajo::save` | orden + sus ítems (alta, edición y baja en un solo guardado) |
| `clientes::save` | cliente + sus contactos |
| cualquier borrado lógico con cascada | doc 04 §4.4 |
| `backup::import_json` | limpia e inserta todas las tablas |

Los CRUD simples de una sola tabla **no** necesitan transacción explícita.

## 12. Matriz de dependencias para el borrado

Antes de borrar lógicamente, verificar que **no** existan filas vivas en:

| Entidad a borrar | Verificar | Clave i18n del error |
| --- | --- | --- |
| `Cliente` | obras, facturas, movimientos | `Validation.Cliente.EnUso` |
| `Obra` | trabajos | `Validation.Obra.EnUso` |
| `Trabajo` | órdenes de trabajo, movimientos, asistencias | `Validation.Trabajo.EnUso` |
| `OrdenTrabajo` | certificados emitidos | `Validation.OrdenTrabajo.TieneCertificados` |
| `OrdenTrabajoItem` | `certificado_items` | `Validation.OrdenTrabajoItem.Certificado` |
| `Factura` | pagos, movimientos imputados | `Validation.Factura.EnUso` |
| `Empleado` | liquidaciones, asistencias, movimientos | `Validation.Empleado.EnUso` |
| `Categoria` | movimientos, subcategorías | `Validation.Categoria.EnUso` / `.TieneHijas` |
| `TipoMovimiento` | movimientos; `es_sistema` | `Validation.TipoMovimiento.EnUso` / `.EsSistema` |
| `TipoConceptoPago` | movimientos; `es_sistema` | `Validation.TipoConceptoPago.EnUso` / `.EsSistema` |
| `Movimiento` | `liquidacion_adelantos` | `Validation.Movimiento.AdelantoLiquidado` |
| `Liquidacion` | — (se anula, y libera sus adelantos) | — |
| `ClienteContacto` | — | — |
| `PagoFactura` | — (recalcula el estado de la factura) | — |

## 13. Checklist de tests obligatorios de este documento

Un caso de uso sin su test correspondiente está incompleto.

- [ ] §3.1 total del movimiento: las 5 filas de la tabla de ejemplos.
- [ ] §3.2 agregación con conjunto vacío devuelve `0`, no error.
- [ ] §3.3 paginación: `size = 0` devuelve todo; `page = 1, size = 30` devuelve 30; el desempate por
      `id` hace estable el orden con dos movimientos del mismo instante.
- [ ] §4.1 `total = subtotal + iva`, y el `total` que manda el cliente se ignora.
- [ ] §4.2 saldo negativo cuando se paga de más.
- [ ] §4.3 registrar un pago que salda la factura la deja en `Pagada`.
- [ ] §4.4 los cuatro caminos de la derivación de estado.
- [ ] §4.6 aging: los bordes exactos en 30, 31, 60, 61, 90 y 91 días; y la invariante de la suma.
- [ ] §5.2 el ejemplo de los 4 200 metros al 60 %.
- [ ] §5.2 orden de operaciones: comparar `(cantidad × precio) × pct` contra
      `cantidad × (precio × pct)` y verificar que se usa el primero.
- [ ] §5.5 emitir dos certificados seguidos acumula bien y numera 1 y 2.
- [ ] §5.5 rechazo cuando el acumulado pasaría de 100.
- [ ] §5.6 anular el último certificado revierte los porcentajes; anular uno anterior falla.
- [ ] §6.3 los cuatro casos del multiplicador, incluido «domingo que es feriado».
- [ ] §6.4 la suma excluye adelantos ya liquidados.
- [ ] §6.6 rama A gana sobre rama B cuando hay **una sola** asistencia en el período.
- [ ] §6.6 rama A: falta y falta justificada aportan cero días y cero pesos.
- [ ] §6.6 rama A: la asistencia `Feriado` no consulta la lista de feriados.
- [ ] §6.6 rama B: cada día computable suma exactamente 1 día.
- [ ] §6.6 rama C: no aplica multiplicadores.
- [ ] §6.7 los dos ejemplos completos, con el valor exacto esperado.
- [ ] §6.8 rechazo por período solapado.
- [ ] §6.9 el lote revierte entero ante un fallo en el último elemento.
- [ ] §6.10 anular libera los adelantos y permite reliquidarlos.
- [ ] §7.1 margen con `ingresos = 0` da `0`; con `ingresos > 0` redondea a 2 decimales.
- [ ] §8.1 el upsert no crea una segunda fila para el mismo empleado y día.
- [ ] §9.5 `change_percent` devuelve `null` con `anterior = 0` y `actual ≠ 0`, y `0` con ambos en 0.
- [ ] §9.5 redondeo a 1 decimal.
- [ ] §10.2 no se puede borrar ninguno de los cuatro tipos de sistema.
