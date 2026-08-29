# 11 — Contratos Tauri (IPC)

> Define `src-tauri/src/commands/` y `src/api/`. Es el contrato entre el frontend y el backend: si
> algo no está acá, el frontend no puede hacerlo.

## 1. Reglas del borde

1. Un comando Tauri es una **capa fina**: valida que llegaron los tipos, resuelve el caso de uso del
   estado de la aplicación, lo invoca y traduce el error. **Cero lógica de negocio.** Un comando de
   más de 15 líneas es sospechoso.
2. Los comandos son `async` y devuelven `Result<T, ApiError>`.
3. Los parámetros van en un **único struct** por comando cuando hay más de dos, para que agregar un
   campo no rompa la firma.
4. Serialización: `#[serde(rename_all = "camelCase")]` **en todos** los DTO. El frontend nunca ve
   `snake_case`.
5. Los `Uuid` viajan como string en formato canónico con guiones.
6. Los importes viajan como **string decimal** con 4 decimales exactos (doc 04 §1.4). Nunca como
   `number`: `0.1 + 0.2` en JavaScript no es `0.3` y una caja no se lleva con eso.
7. Las fechas con hora viajan como **RFC 3339 en UTC** (`2026-08-29T12:34:56Z`). Las fechas civiles
   viajan como `YYYY-MM-DD` sin hora ni zona (doc 04 §3).
8. Los enums viajan como el **nombre de la variante** en PascalCase (`"Emitida"`,
   `"FaltaJustificada"`), no como número. El número es un detalle de la base.
9. Ningún comando devuelve una entidad de dominio: siempre un DTO de `eo-application`.
10. Ningún comando recibe SQL, nombres de columna libres ni rutas de archivo sin validar.

## 2. Envoltura de error

Un solo tipo de error cruza el borde:

```rust
// src-tauri/src/error.rs
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    /// Categoría, para que el frontend decida cómo mostrarlo.
    pub kind: ApiErrorKind,
    /// Clave i18n del mensaje principal.
    pub key: String,
    /// Parámetros a interpolar en el mensaje.
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub params: std::collections::BTreeMap<String, String>,
    /// Errores por campo. Sólo en `kind == "validation"`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldError>,
    /// Identificador de correlación para cruzar con el log. Nunca contiene datos del usuario.
    pub trace_id: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ApiErrorKind {
    Validation,   // el formulario tiene errores; se pintan los campos
    NotFound,     // el registro no existe o está borrado
    Conflict,     // regla de negocio o transición de estado inválida
    Concurrency,  // row_version desactualizado
    External,     // falló un servicio externo
    Io,           // falló el sistema de archivos
    Internal,     // bug; se loguea completo y al usuario se le muestra un mensaje genérico
}
```

`Internal` **nunca** expone el mensaje original: el frontend recibe
`key: "Error.Internal"` y el `trace_id`. El detalle va al log (doc 02 §6).

Mapeo desde `AppError`:

| `AppError` | `kind` | Notas |
| --- | --- | --- |
| `Validation(errs)` | `validation` | `key = "Error.Validation"`, `fields` poblado |
| `NotFound { entity, id }` | `notFound` | `key = "Validation.Common.EntityNotFound"` |
| `Conflict { key, params }` | `conflict` | la clave viene del dominio (docs 07 §5 y 08 §6) |
| `Concurrency` | `concurrency` | `key = "Validation.Common.ConcurrencyConflict"` |
| `Domain(InvalidStateTransition{..})` | `conflict` | `key = "State.InvalidTransition"` |
| `External { service }` | `external` | |
| `Io(_)` | `io` | |
| resto | `internal` | |

## 3. Convención de nombres

```
<modulo>_<accion>
```

`modulo` en plural y `accion` del conjunto cerrado: `list`, `get`, `create`, `update`, `delete`,
`restore`, `count`, `export`, más los verbos propios del módulo (`emit`, `settle`, `suggest`, …).
Todo en `snake_case`, porque es el nombre de la función Rust.

Ejemplos: `movimientos_list`, `facturas_emit`, `liquidaciones_suggest`.

Un archivo por módulo en `src-tauri/src/commands/`, y un módulo espejo en `src/api/`.

## 4. Tipos compartidos

### 4.1 Genéricos

```ts
// src/api/types/common.ts

/** Importe con 4 decimales exactos, como string. Ej. "1234.5000". */
export type Money = string;
/** Decimal con 4 decimales exactos, como string. Ej. "26.5000". */
export type Decimal4 = string;
/** UUID canónico con guiones. */
export type Uuid = string;
/** Instante en UTC, RFC 3339. Ej. "2026-08-29T12:34:56Z". */
export type Instant = string;
/** Fecha civil sin hora. Ej. "2026-08-29". */
export type CivilDate = string;
/** row_version como 8 bytes en hexadecimal. Ej. "0000000000000001". */
export type RowVersion = string;

export type SortDir = 'Asc' | 'Desc';

export interface ListQuery<F> {
  filtro: F;
  page: number;          // 1-based
  pageSize: number;      // 0 = todos
  sortBy?: string;
  sortDir?: SortDir;
}

export interface PagedResult<T> {
  items: T[];
  totalCount: number;
  page: number;
  pageSize: number;
  totalPages: number;    // 0 si pageSize === 0
}

export interface Audit {
  createdAt: Instant;
  updatedAt: Instant | null;
  rowVersion: RowVersion;
  isDeleted: boolean;
  deletedAt: Instant | null;
}

/** Opción de un selector. Devuelto por los comandos `*_lookup`. */
export interface LookupItem {
  id: Uuid;
  label: string;
  /** Datos extra que el selector necesite: color, tarifa, estado. */
  meta?: Record<string, string>;
}

export interface EstadoInfo {
  actual: string;
  clave: string;
  transicionesPermitidas: TransicionPermitida[];
  esTerminal: boolean;
}

export interface TransicionPermitida {
  destino: string;
  clave: string;
  accion: string;
  requiereConfirmacion: boolean;
  confirmacionClave?: string;
}

export interface FieldError {
  field: string;
  key: string;
  params?: Record<string, string>;
}

export interface ApiError {
  kind: 'validation' | 'notFound' | 'conflict' | 'concurrency' | 'external' | 'io' | 'internal';
  key: string;
  params?: Record<string, string>;
  fields?: FieldError[];
  traceId: string;
}
```

### 4.2 Generación de los tipos

Los tipos TypeScript **no se escriben a mano**: se generan desde Rust con `ts-rs`, que deriva
`TS` en cada DTO y escribe `src/api/types/generated.ts`. El comando `pnpm gen:types` los regenera y
el CI falla si el archivo generado difiere del comiteado.

**Ningún tipo del contrato se define dos veces.** Si un tipo aparece escrito a mano en el frontend y
también en Rust, es un bug.

Los tipos de §4.1 marcados como alias (`Money`, `Uuid`, …) sí se escriben a mano una vez, porque son
alias de `string` y `ts-rs` no puede inferir la intención.

## 5. Comandos por módulo

Notación: `nombre(params) -> retorno`. Los errores posibles se listan por `kind`; `internal` es
siempre posible y no se repite.

### 5.1 Movimientos

| Comando | Params | Retorno | Errores |
| --- | --- | --- | --- |
| `movimientos_list` | `ListQuery<MovimientoFiltro>` | `MovimientoListResult` | `validation` (sortBy no permitido) |
| `movimientos_get` | `{ id: Uuid }` | `MovimientoDetalle` | `notFound` |
| `movimientos_create` | `{ dto: MovimientoInput }` | `MovimientoDetalle` | `validation`, `notFound` (FK), `conflict` |
| `movimientos_update` | `{ id, dto, rowVersion }` | `MovimientoDetalle` | `validation`, `notFound`, `concurrency`, `conflict` |
| `movimientos_delete` | `{ id, rowVersion }` | `void` | `notFound`, `concurrency`, `conflict` |
| `movimientos_restore` | `{ id }` | `MovimientoDetalle` | `notFound` |
| `movimientos_resumen` | `{ filtro: MovimientoFiltro }` | `MovimientoResumen` | — |
| `movimientos_export` | `{ filtro, formato, destino }` | `ExportResult` | `io`, `validation` |

```ts
export interface MovimientoFiltro {
  concepto?: string;
  tipoMovimientoId?: Uuid;
  categoriaId?: Uuid;
  clienteId?: Uuid;
  obraId?: Uuid;
  trabajoId?: Uuid;
  facturaId?: Uuid;
  empleadoId?: Uuid;
  moneda?: Moneda;
  fechaDesde?: CivilDate;
  fechaHasta?: CivilDate;
  montoMin?: Money;
  montoMax?: Money;
  incluirBorrados?: boolean;   // default false
}

export interface MovimientoListItem {
  id: Uuid;
  fecha: CivilDate;
  concepto: string;
  monto: Money;
  cantidad: Decimal4;
  total: Money;               // calculado, monto * cantidad
  unidad: string | null;
  moneda: Moneda;
  cotizacionAplicada: Money | null;
  tipoMovimientoId: Uuid;
  tipoMovimientoNombre: string;
  tipoMovimientoColor: string | null;
  esIngreso: boolean;
  categoriaId: Uuid;
  categoriaNombre: string;
  categoriaColor: string | null;
  clienteId: Uuid | null;
  clienteNombre: string | null;
  obraId: Uuid | null;
  obraNombre: string | null;
  trabajoId: Uuid | null;
  trabajoDescripcion: string | null;
  facturaId: Uuid | null;
  facturaNumero: string | null;
  empleadoId: Uuid | null;
  empleadoNombre: string | null;
  tipoConceptoPagoId: Uuid | null;
  tipoConceptoPagoNombre: string | null;
  adjuntosCount: number;
  /** true si un adelanto ya fue consumido por una liquidación: no editable. */
  bloqueadoPorLiquidacion: boolean;
  rowVersion: RowVersion;
}

/** El resultado de `movimientos_list` incluye el resumen del filtro completo,
 *  no de la página. Doc 09 §3.2. */
export interface MovimientoListResult extends PagedResult<MovimientoListItem> {
  resumen: MovimientoResumen;
}

export interface MovimientoResumen {
  totalIngresos: Money;
  totalGastos: Money;
  balance: Money;
  cantidad: number;
}
```

`sortBy` permitido: `fecha`, `concepto`, `monto`, `total`, `tipoMovimientoNombre`,
`categoriaNombre`, `createdAt`. Default `fecha` `Desc`, desempate `createdAt` `Desc`.

`MovimientoInput` tiene los mismos campos editables que `MovimientoListItem` menos los derivados
(`total`, los `*Nombre`, `*Color`, `adjuntosCount`, `bloqueadoPorLiquidacion`, `rowVersion`).
`MovimientoDetalle` es `MovimientoListItem` más `observaciones`, `adjuntos: AdjuntoItem[]` y `audit`.

Esta estructura —filtro, item de lista, input, detalle, resumen— se repite en todos los módulos y no
se vuelve a transcribir campo por campo: se deriva de las entidades de
[`05-dominio-entidades.md`](./05-dominio-entidades.md) aplicando estas reglas:

- **`*Filtro`**: un campo opcional por criterio de la tabla de filtros del módulo en doc 09 §3.
- **`*ListItem`**: los campos que muestra la tabla del módulo, más los `id` que necesitan los
  enlaces, más los derivados que se muestran, más `rowVersion`.
- **`*Input`**: los campos editables del formulario, con los hijos anidados cuando corresponde.
- **`*Detalle`**: `*ListItem` + observaciones + colecciones hijas + `audit` + `estado?: EstadoInfo`.

### 5.2 Clientes

| Comando | Params | Retorno |
| --- | --- | --- |
| `clientes_list` | `ListQuery<ClienteFiltro>` | `PagedResult<ClienteListItem>` |
| `clientes_get` | `{ id }` | `ClienteDetalle` |
| `clientes_create` | `{ dto: ClienteInput }` | `ClienteDetalle` |
| `clientes_update` | `{ id, dto, rowVersion }` | `ClienteDetalle` |
| `clientes_delete` | `{ id, rowVersion }` | `void` |
| `clientes_lookup` | `{ texto?: string, limite?: number }` | `LookupItem[]` |
| `clientes_cuenta_corriente` | `{ clienteId, incluirPagadas: boolean }` | `CuentaCorriente` |
| `clientes_antiguedad_deuda` | `{ fechaCorte: CivilDate, clienteId?: Uuid }` | `AntiguedadDeuda` |

`ClienteInput` incluye `contactos: ClienteContactoInput[]`: los contactos se crean, editan y borran
**en la misma llamada** que el cliente, dentro de una transacción. No hay comandos sueltos de
contacto; son parte del agregado.

```ts
export interface CuentaCorriente {
  clienteId: Uuid;
  clienteNombre: string;
  totalFacturado: Money;
  totalPagado: Money;
  saldo: Money;
  facturas: CuentaCorrienteFactura[];
}

export interface CuentaCorrienteFactura {
  id: Uuid;
  numero: string;
  fecha: CivilDate;
  fechaVencimiento: CivilDate | null;
  estado: EstadoFactura;
  total: Money;
  pagado: Money;
  saldo: Money;
  diasMora: number;
}

export interface AntiguedadDeuda {
  fechaCorte: CivilDate;
  total: Money;
  bucket0a30: Money;
  bucket31a60: Money;
  bucket61a90: Money;
  bucketMas90: Money;
  detalle: AntiguedadDeudaCliente[];
}
```

### 5.3 Obras

| Comando | Params | Retorno |
| --- | --- | --- |
| `obras_list` | `ListQuery<ObraFiltro>` | `PagedResult<ObraListItem>` |
| `obras_get` | `{ id }` | `ObraDetalle` |
| `obras_create` | `{ dto: ObraInput }` | `ObraDetalle` |
| `obras_update` | `{ id, dto, rowVersion }` | `ObraDetalle` |
| `obras_delete` | `{ id, rowVersion }` | `void` |
| `obras_lookup` | `{ clienteId?, texto?, limite? }` | `LookupItem[]` |
| `obras_next_numero` | — | `{ numero: number }` |
| `obras_transition` | `{ id, destino: EstadoObra, rowVersion, cascada: boolean }` | `ObraDetalle` |
| `obras_rentabilidad` | `{ obraId }` | `RentabilidadObra` |

`obras_transition` es la **única** forma de cambiar el estado. `cascada` sólo se mira en las
transiciones a `Finalizada` y `Cancelada` (doc 08 §3.3); si hay trabajos abiertos y `cascada` es
`false`, devuelve `conflict` con `State.Obra.TieneTrabajosAbiertos` y `params.count`, y el frontend
pregunta.

### 5.4 Trabajos y órdenes de trabajo

| Comando | Params | Retorno |
| --- | --- | --- |
| `trabajos_list` | `ListQuery<TrabajoFiltro>` | `PagedResult<TrabajoListItem>` |
| `trabajos_get` | `{ id }` | `TrabajoDetalle` |
| `trabajos_create` | `{ dto: TrabajoInput }` | `TrabajoDetalle` |
| `trabajos_update` | `{ id, dto, rowVersion }` | `TrabajoDetalle` |
| `trabajos_delete` | `{ id, rowVersion }` | `void` |
| `trabajos_lookup` | `{ obraId?, texto?, limite? }` | `LookupItem[]` |
| `trabajos_transition` | `{ id, destino: EstadoTrabajo, rowVersion, forzar: boolean }` | `TrabajoDetalle` |
| `ordenes_trabajo_list` | `{ trabajoId }` | `OrdenTrabajoListItem[]` |
| `ordenes_trabajo_get` | `{ id }` | `OrdenTrabajoDetalle` |
| `ordenes_trabajo_create` | `{ dto: OrdenTrabajoInput }` | `OrdenTrabajoDetalle` |
| `ordenes_trabajo_update` | `{ id, dto, rowVersion }` | `OrdenTrabajoDetalle` |
| `ordenes_trabajo_delete` | `{ id, rowVersion }` | `void` |

`OrdenTrabajoInput` incluye `items: OrdenTrabajoItemInput[]`; los ítems son parte del agregado y se
guardan en la misma transacción, con borrado de los que ya no vienen en la lista (salvo los que
tienen certificación, doc 09 §3.6).

`ordenes_trabajo_list` no pagina: una orden tiene decenas de ítems, no miles.

### 5.5 Certificados

| Comando | Params | Retorno |
| --- | --- | --- |
| `certificados_list` | `ListQuery<CertificadoFiltro>` | `PagedResult<CertificadoListItem>` |
| `certificados_get` | `{ id }` | `CertificadoDetalle` |
| `certificados_preparar` | `{ ordenTrabajoId }` | `CertificadoBorrador` |
| `certificados_create` | `{ dto: CertificadoInput }` | `CertificadoDetalle` |
| `certificados_update_observaciones` | `{ id, observaciones, rowVersion }` | `CertificadoDetalle` |
| `certificados_delete` | `{ id, rowVersion }` | `void` |
| `certificados_export_pdf` | `{ id, destino }` | `ExportResult` |

`certificados_preparar` devuelve el borrador con cada ítem de la orden y su porcentaje acumulado
histórico ya calculado, más el número que corresponde. Es lo que puebla el formulario de alta.

No existe `certificados_update` completo: un certificado emitido es inmutable salvo observaciones
(doc 08 §5.1).

```ts
export interface CertificadoBorrador {
  ordenTrabajoId: Uuid;
  numeroSugerido: number;
  trabajoDescripcion: string;
  obraNombre: string;
  clienteNombre: string;
  items: CertificadoBorradorItem[];
}

export interface CertificadoBorradorItem {
  ordenTrabajoItemId: Uuid;
  descripcion: string;
  unidad: string | null;
  cantidad: Decimal4;
  precioUnitario: Money;
  /** Suma de los porcentajes de los certificados anteriores. */
  porcentajeAcumuladoAnterior: Decimal4;
  /** 100 - porcentajeAcumuladoAnterior. El máximo que se puede certificar ahora. */
  porcentajeDisponible: Decimal4;
  subtotalAcumuladoAnterior: Money;
}
```

### 5.6 Facturas y pagos

| Comando | Params | Retorno |
| --- | --- | --- |
| `facturas_list` | `ListQuery<FacturaFiltro>` | `PagedResult<FacturaListItem>` |
| `facturas_get` | `{ id }` | `FacturaDetalle` |
| `facturas_create` | `{ dto: FacturaInput }` | `FacturaDetalle` |
| `facturas_update` | `{ id, dto, rowVersion }` | `FacturaDetalle` |
| `facturas_delete` | `{ id, rowVersion }` | `void` |
| `facturas_lookup` | `{ clienteId?, soloImpagas?, texto?, limite? }` | `LookupItem[]` |
| `facturas_transition` | `{ id, destino: EstadoFactura, rowVersion }` | `FacturaDetalle` |
| `pagos_factura_list` | `{ facturaId }` | `PagoFacturaItem[]` |
| `pagos_factura_create` | `{ dto: PagoFacturaInput }` | `FacturaDetalle` |
| `pagos_factura_update` | `{ id, dto, rowVersion }` | `FacturaDetalle` |
| `pagos_factura_delete` | `{ id, rowVersion }` | `FacturaDetalle` |

Los tres comandos de pago devuelven la **factura completa**, no el pago: el estado y el saldo cambian
como efecto (doc 08 §2.4) y el frontend necesita ambos para refrescar sin una segunda llamada.

### 5.7 Empleados y asistencia

| Comando | Params | Retorno |
| --- | --- | --- |
| `empleados_list` | `ListQuery<EmpleadoFiltro>` | `PagedResult<EmpleadoListItem>` |
| `empleados_get` | `{ id }` | `EmpleadoDetalle` |
| `empleados_create` | `{ dto: EmpleadoInput }` | `EmpleadoDetalle` |
| `empleados_update` | `{ id, dto, rowVersion }` | `EmpleadoDetalle` |
| `empleados_delete` | `{ id, rowVersion }` | `void` |
| `empleados_lookup` | `{ soloActivos?, texto?, limite? }` | `LookupItem[]` |
| `asistencia_grilla` | `{ desde: CivilDate, hasta: CivilDate, empleadoIds?: Uuid[] }` | `AsistenciaGrilla` |
| `asistencia_upsert` | `{ empleadoId, fecha, tipoJornada, obraId?, observaciones? }` | `AsistenciaCelda` |
| `asistencia_delete` | `{ empleadoId, fecha }` | `void` |
| `asistencia_upsert_rango` | `{ empleadoId, desde, hasta, tipoJornada, soloDiasHabiles: boolean }` | `AsistenciaCelda[]` |

`asistencia_upsert` es idempotente: crea o actualiza según `(empleadoId, fecha)`. No recibe `id` ni
`rowVersion`: la clave natural es la identidad y el último click gana. Es una grilla de carga rápida,
no un formulario con concurrencia.

`asistencia_upsert_rango` cubre la carga masiva de doc 09 §3.10. Con `soloDiasHabiles = true`
excluye sábados, domingos y feriados del calendario.

```ts
export interface AsistenciaGrilla {
  desde: CivilDate;
  hasta: CivilDate;
  dias: AsistenciaDia[];
  filas: AsistenciaFila[];
}

export interface AsistenciaDia {
  fecha: CivilDate;
  /** 1 = lunes … 7 = domingo. */
  diaSemana: number;
  esFinDeSemana: boolean;
  esFeriado: boolean;
  feriadoNombre: string | null;
}

export interface AsistenciaFila {
  empleadoId: Uuid;
  empleadoNombre: string;
  celdas: AsistenciaCelda[];      // una por cada día de `dias`, en el mismo orden
  resumen: AsistenciaResumen;
}

export interface AsistenciaCelda {
  fecha: CivilDate;
  tipoJornada: TipoJornada | null;   // null = sin registro
  obraId: Uuid | null;
  observaciones: string | null;
}

export interface AsistenciaResumen {
  completas: number;
  medias: number;
  faltas: number;
  faltasJustificadas: number;
  feriados: number;
  /** Suma de los factores de jornada. Doc 06 §6.3. */
  jornadasEquivalentes: Decimal4;
}
```

`celdas` tiene **siempre** la misma longitud que `dias`, con `tipoJornada: null` en los días sin
registro. Así el frontend no necesita buscar por fecha para dibujar la grilla.

### 5.8 Liquidaciones

| Comando | Params | Retorno |
| --- | --- | --- |
| `liquidaciones_list` | `ListQuery<LiquidacionFiltro>` | `PagedResult<LiquidacionListItem>` |
| `liquidaciones_get` | `{ id }` | `LiquidacionDetalle` |
| `liquidaciones_suggest` | `{ empleadoIds: Uuid[], desde, hasta, diasManuales?: Record<Uuid, Decimal4> }` | `LiquidacionSugerencia[]` |
| `liquidaciones_create_batch` | `{ dtos: LiquidacionInput[], generarPdf: boolean }` | `LiquidacionBatchResult` |
| `liquidaciones_update` | `{ id, dto, rowVersion }` | `LiquidacionDetalle` |
| `liquidaciones_delete` | `{ id, rowVersion }` | `void` |
| `liquidaciones_export_pdf` | `{ id, destino }` | `ExportResult` |

`liquidaciones_suggest` es **puro**: calcula y no persiste. Alimenta el paso 2 del asistente. Recibe
varios empleados en una sola llamada para no hacer N peticiones.

```ts
export interface LiquidacionSugerencia {
  empleadoId: Uuid;
  empleadoNombre: string;
  desde: CivilDate;
  hasta: CivilDate;
  diasTrabajados: Decimal4;
  tarifaAplicada: Money;
  totalBruto: Money;
  totalAdelantos: Money;
  totalNeto: Money;
  /** De dónde salieron los días: doc 06 §6. */
  origen: 'Manual' | 'Asistencia' | 'Calendario';
  desglose: LiquidacionDesglose;
  adelantos: LiquidacionAdelantoSugerido[];
}

export interface LiquidacionDesglose {
  jornadasCompletas: Decimal4;
  jornadasMedias: Decimal4;
  faltas: number;
  faltasJustificadas: number;
  diasSabado: Decimal4;
  diasDomingo: Decimal4;
  diasFeriado: Decimal4;
  multiplicadorSabado: Decimal4;
  multiplicadorDomingo: Decimal4;
  multiplicadorFeriado: Decimal4;
  recargos: Money;
}

export interface LiquidacionAdelantoSugerido {
  movimientoId: Uuid;
  fecha: CivilDate;
  concepto: string;
  monto: Money;
  /** true si ya lo consumió otra liquidación: se muestra tachado y no suma. */
  yaDescontado: boolean;
  liquidacionQueLoDesconto: Uuid | null;
  /** Lo decide el usuario en el paso 2. Default: !yaDescontado. */
  incluir: boolean;
}

export interface LiquidacionBatchResult {
  creadas: Uuid[];
  pdfsGenerados: string[];   // rutas absolutas
}
```

`liquidaciones_create_batch` es **atómico**: una sola transacción para todas las liquidaciones y
todas las filas de `liquidacion_adelantos`. Si una falla, ninguna se guarda y el error indica cuál
con `params.empleado`.

### 5.9 Administración

| Comando | Params | Retorno |
| --- | --- | --- |
| `categorias_list` | `ListQuery<CategoriaFiltro>` | `PagedResult<CategoriaListItem>` |
| `categorias_tree` | `{ texto?: string }` | `CategoriaNodo[]` |
| `categorias_get` | `{ id }` | `CategoriaDetalle` |
| `categorias_create` | `{ dto }` | `CategoriaDetalle` |
| `categorias_update` | `{ id, dto, rowVersion }` | `CategoriaDetalle` |
| `categorias_delete` | `{ id, rowVersion }` | `void` |
| `categorias_lookup` | `{ texto?, limite? }` | `LookupItem[]` |
| `tipos_movimiento_list` | `ListQuery<TipoMovimientoFiltro>` | `PagedResult<TipoMovimientoListItem>` |
| `tipos_movimiento_get` | `{ id }` | `TipoMovimientoDetalle` |
| `tipos_movimiento_create` | `{ dto }` | `TipoMovimientoDetalle` |
| `tipos_movimiento_update` | `{ id, dto, rowVersion }` | `TipoMovimientoDetalle` |
| `tipos_movimiento_delete` | `{ id, rowVersion }` | `void` |
| `tipos_movimiento_lookup` | `{ texto?, limite? }` | `LookupItem[]` |
| `tipos_concepto_pago_lookup` | `{ texto?, limite? }` | `LookupItem[]` |

Los `*_lookup` existen para poblar selectores sin traerse el listado completo con sus conteos y
derivados. Devuelven a lo sumo `limite` ítems, default 50, y siempre incluyen el ítem ya
seleccionado aunque no entre en el límite.

### 5.10 Dashboard

| Comando | Params | Retorno |
| --- | --- | --- |
| `dashboard_stats` | `{ periodo: PeriodoDashboard }` | `DashboardStats` |
| `dashboard_alertas` | — | `Alerta[]` |
| `cotizaciones_get` | — | `Cotizacion[]` |

```ts
export type PeriodoDashboard = 'Mensual' | 'Anual' | 'Total';

export interface DashboardStats {
  periodo: PeriodoDashboard;
  desde: CivilDate;
  hasta: CivilDate;
  totalIngresos: Money;
  totalGastos: Money;
  balance: Money;
  cantidadMovimientos: number;
  variacionIngresos: Decimal4 | null;   // % contra el período anterior; null si no hay base
  variacionGastos: Decimal4 | null;
  variacionBalance: Decimal4 | null;
  serieMensual: PuntoSerie[];
  topClientes: TopCliente[];
  rentabilidadObras: RentabilidadObra[];
  ultimosMovimientos: MovimientoListItem[];
  estadoSistema: EstadoSistema;
}

export interface Alerta {
  tipo: 'FacturasVencidas' | 'ObrasPausadas' | 'LiquidacionesPendientes';
  clave: string;              // clave i18n del mensaje
  cantidad: number;
  severidad: 'Info' | 'Warning' | 'Error';
  /** Ruta con los filtros ya aplicados. Doc 09 §3.1. */
  destino: string;
}
```

`dashboard_alertas` va separado de `dashboard_stats` porque no depende del período y se refresca con
otra frecuencia.

### 5.11 Reportes y exportación

| Comando | Params | Retorno |
| --- | --- | --- |
| `reportes_generar` | `{ reporte: TipoReporte, formato: FormatoExport, parametros, destino }` | `ExportResult` |

Un comando único, con `reporte` y `parametros` tipados por unión discriminada. Los layouts están en
[`12-reportes-y-exportaciones.md`](./12-reportes-y-exportaciones.md).

```ts
export type FormatoExport = 'Pdf' | 'Xlsx' | 'Docx' | 'Csv' | 'Json';

export type TipoReporte =
  | 'Movimientos' | 'CajaPorPeriodo' | 'RentabilidadObras'
  | 'CuentaCorriente' | 'AntiguedadDeuda' | 'Liquidacion'
  | 'Certificado' | 'AsistenciaMensual' | 'BaseCompleta';

export interface ExportResult {
  ruta: string;
  bytes: number;
  registros: number;
}
```

`destino` es una ruta absoluta que el frontend obtuvo del diálogo del sistema. El backend valida que
el directorio exista y sea escribible, y que la extensión coincida con el formato; **no** acepta una
ruta arbitraria sin esa validación.

### 5.12 Adjuntos

| Comando | Params | Retorno |
| --- | --- | --- |
| `adjuntos_list` | `{ entidadTipo: EntidadAdjunto, entidadId: Uuid }` | `AdjuntoItem[]` |
| `adjuntos_add` | `{ entidadTipo, entidadId, rutaOrigen: string, descripcion? }` | `AdjuntoItem` |
| `adjuntos_delete` | `{ id }` | `void` |
| `adjuntos_open` | `{ id }` | `void` |
| `adjuntos_reveal` | `{ id }` | `void` |

Detalle de rutas, tipos MIME y límites en
[`13-servicios-externos-y-archivos.md`](./13-servicios-externos-y-archivos.md) §1.

### 5.13 Configuración y sistema

| Comando | Params | Retorno |
| --- | --- | --- |
| `config_get_all` | — | `AppConfig` |
| `config_set` | `{ cambios: Record<string, string> }` | `AppConfig` |
| `config_reset` | `{ claves: string[] }` | `AppConfig` |
| `feriados_list` | `{ anio: number }` | `Feriado[]` |
| `feriados_sync` | `{ anios: number[] }` | `{ agregados: number, total: number }` |
| `feriados_add` | `{ fecha, nombre }` | `Feriado[]` |
| `feriados_delete` | `{ fecha }` | `Feriado[]` |
| `sistema_info` | — | `EstadoSistema` |
| `sistema_migraciones` | — | `{ aplicadas: string[], pendientes: string[] }` |
| `backup_list` | — | `BackupItem[]` |
| `backup_create` | — | `BackupItem` |
| `backup_restore` | `{ nombre: string }` | `void` |
| `backup_verify` | `{ nombre: string }` | `{ ok: boolean, detalle: string }` |
| `backup_export_json` | `{ destino }` | `ExportResult` |
| `backup_import_json` | `{ origen }` | `{ tablas: number, filas: number }` |
| `busqueda_global` | `{ texto: string, limitePorEntidad?: number }` | `ResultadoBusqueda` |

`config_set` recibe sólo las claves que cambiaron y valida cada una contra su tipo y su rango
declarados en [`14-configuracion-e-i18n.md`](./14-configuracion-e-i18n.md). Una clave desconocida es
`validation`, no se ignora en silencio.

`backup_restore` y `backup_import_json` son destructivos: el frontend los confirma dos veces y el
backend crea un backup automático antes de ejecutarlos.

## 6. Eventos

Además de los comandos, el backend emite eventos para lo que no es petición-respuesta:

| Evento | Payload | Cuándo |
| --- | --- | --- |
| `db:ready` | `{ migracionesAplicadas: number }` | terminó la inicialización de la base al arrancar |
| `db:error` | `{ key, params }` | falló la inicialización; la aplicación queda en modo degradado |
| `export:progress` | `{ id, actual, total }` | durante una exportación larga |
| `backup:progress` | `{ fase, porcentaje }` | durante backup o restauración |
| `cotizaciones:updated` | `Cotizacion[]` | llegó la respuesta de la API del dólar |
| `config:changed` | `{ claves: string[] }` | cambió la configuración, para que las vistas abiertas reaccionen |

El frontend arranca mostrando una pantalla de carga hasta `db:ready`. Ningún comando de datos se
llama antes de ese evento.

## 7. Capa `src/api/`

Un módulo por grupo de comandos, con la misma partición que §5. Cada función:

```ts
// src/api/movimientos.ts
import { invoke } from '@tauri-apps/api/core';
import type { ListQuery, PagedResult } from './types/common';
import type { MovimientoFiltro, MovimientoListResult, MovimientoDetalle } from './types/generated';

export function list(query: ListQuery<MovimientoFiltro>): Promise<MovimientoListResult> {
  return invoke('movimientos_list', { query });
}

export function get(id: string): Promise<MovimientoDetalle> {
  return invoke('movimientos_get', { id });
}
```

Reglas:

1. **Ningún componente Vue llama a `invoke` directamente.** Siempre a través de `src/api/`.
2. La capa `api` no maneja errores: los deja propagar. El manejo está en los stores y en el
   interceptor global (doc 16).
3. La capa `api` no transforma datos: no formatea importes ni convierte fechas. Eso es de los
   componentes y sus utilidades.
4. Un helper `callCommand` envuelve `invoke` para normalizar el error de Tauri a `ApiError` y
   agregar el `trace_id` al log del frontend.

## 8. Checklist para agregar un comando

1. Definir el DTO de entrada y de salida en `eo-application/src/dtos/`, con `Serialize`,
   `Deserialize` y `TS`.
2. Escribir el caso de uso en `eo-application/src/use_cases/`, con su test.
3. Escribir el validador si hay entrada de usuario (doc 07).
4. Escribir el comando en `src-tauri/src/commands/<modulo>.rs` y registrarlo en el
   `invoke_handler`.
5. Regenerar los tipos: `pnpm gen:types`.
6. Escribir la función en `src/api/<modulo>.ts`.
7. Agregar las claves i18n de los errores nuevos a `es.json` y `en.json`.
8. Agregar el comando a la tabla de este documento.

Si el paso 8 se saltea, el documento deja de ser el contrato y se vuelve un archivo desactualizado.
El test `comandos_documentados` (doc 17) compara los nombres registrados en el `invoke_handler`
contra los que aparecen en este archivo y falla si no coinciden.
