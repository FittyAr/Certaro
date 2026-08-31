# 17 · Estrategia de testing

El sistema anterior tenía **174 métodos de test repartidos en 58 archivos** (unos 185 casos con los
`[Theory]` expandidos), concentrados casi por completo en servicios de aplicación y ViewModels. Este
documento define la suite del sistema nuevo: qué se testea en cada capa, con qué herramientas, y
cuáles son los tests que **no pueden faltar**.

Regla de arranque: un módulo sin tests no está terminado. Ver
[`AGENTS.md`](../.agents/AGENTS.md) y el criterio de terminado de cada fase en
[`19-roadmap.md`](./19-roadmap.md).

---

## 1. Pirámide por capa

| Capa | Tipo de test | Herramientas | Base de datos | Cantidad esperada |
| --- | --- | --- | --- | --- |
| `eo-domain` | unitario puro | `cargo test`, `rstest`, `proptest` | no | la mayoría |
| `eo-application` | unitario con dobles | `cargo test`, `mockall` | no | mucha |
| `eo-infrastructure` (repos) | integración | `cargo test`, SQLite en memoria | sí | media |
| `eo-infrastructure` (HTTP) | integración con servidor falso | `wiremock` | no | poca |
| `eo-infrastructure` (reportes) | caracterización | `insta` + hashes de bytes | no | poca |
| `eo-migration` | integración | `cargo test` sobre base temporal | sí | poca |
| `eo-import-legacy` | integración con fixtures | `cargo test` | sí | media (doc 15 §8) |
| `src-tauri` (comandos) | contrato | `cargo test` | sí | una por comando |
| Frontend (composables, utilidades) | unitario | `vitest` | no | mucha |
| Frontend (componentes) | componente | `vitest` + `@vue/test-utils` + `jsdom` | no | media |
| Frontend (arquitectura) | estático | `vitest` leyendo archivos | no | los de doc 16 §8 |

No hay tests end-to-end con la aplicación empaquetada. El costo de mantenerlos en un proyecto de un
solo desarrollador no se paga; el equivalente es el checklist manual de §9.

---

## 2. Dominio: `eo-domain`

Es la capa con más densidad de tests porque es la que contiene la aritmética y no tiene dependencias.

### 2.1 `Money` y `Decimal4`

Son los tipos de los que depende **toda** cifra del sistema. Se testean de manera exhaustiva.

| Test | Verifica |
| --- | --- |
| `money_parse_y_display_son_inversos` | `parse(display(m)) == m` para valores de referencia |
| `money_display_siempre_cuatro_decimales` | `"0.0000"`, `"-1.5000"`, `"12345.6700"` |
| `money_suma_es_exacta` | `0.1 + 0.2 == 0.3` en `Money`, que es el caso que falla en `f64` |
| `money_suma_detecta_overflow` | `checked_add` devuelve `None` en el borde de `i64` |
| `money_multiplicacion_redondea_half_away_from_zero` | `2.5 → 3`, `-2.5 → -3`, `1.00005 → 1.0001` |
| `money_multiplicacion_por_cero_es_cero` | y no `-0` |
| `money_division_por_cero_es_error` | devuelve `DomainError`, no panic |
| `money_negativo_conserva_signo_en_display` | el `-` va antes del símbolo |
| `money_serializa_como_string` | el JSON es `"12345.6700"`, no un número |
| `money_deserializa_rechaza_mas_de_cuatro_decimales` | `"1.00005"` es un error, no se trunca en silencio |
| `decimal4_porcentaje_de_money_es_exacto` | `1000.0000 × 33.3333%` da el valor esperado al centésimo de centavo |

Además, tres tests de propiedad con `proptest`:

```rust
proptest! {
    /// La suma es asociativa. Con f64 no lo es.
    #[test]
    fn money_suma_asociativa(a: i64, b: i64, c: i64) { … }

    /// El display y el parse son inversos para todo valor representable.
    #[test]
    fn money_roundtrip(raw: i64) { … }

    /// Multiplicar por 100% devuelve el mismo valor.
    #[test]
    fn money_por_cien_por_ciento_es_identidad(raw: i64) { … }
}
```

El tercero parece trivial y no lo es: con una implementación ingenua de la multiplicación
(`a * b / 10_000` sin redondeo correcto), multiplicar por `100%` pierde una unidad en la mitad de los
valores.

### 2.2 Fechas

| Test | Verifica |
| --- | --- |
| `civil_to_utc_es_medianoche` | la hora resultante es `00:00:00.000` |
| `civil_roundtrip` | `utc_to_civil(civil_to_utc(d)) == d` para todo `d` |
| `civil_no_cambia_de_dia_cerca_de_medianoche` | el caso que rompía al sistema anterior |
| `rango_de_dia_incluye_el_ultimo_milisegundo` | el filtro `hasta` incluye `23:59:59.999` |
| `formato_de_almacenamiento_tiene_24_caracteres` | el orden lexicográfico coincide con el cronológico |

### 2.3 Propiedades calculadas

Cada método calculado de doc 05 tiene su test, con el caso normal y los bordes:

| Método | Casos |
| --- | --- |
| `Movimiento::total()` | normal, `cantidad = 1`, monto negativo |
| `Liquidacion::total_neto()` | adelantos menores, iguales y **mayores** al bruto (da negativo, y está bien) |
| `OrdenTrabajoItem::porcentaje_acumulado()` | suma exacta a 100, y el caso que la excede |
| `OrdenTrabajoItem::subtotal_actual()` | porcentaje 0, 50, 100 |
| `Factura::total()` | con y sin IVA |
| `Factura::saldo()` | sin pagos, pago parcial, pago exacto, sobrepago |
| `Empleado::dias_por_periodo()` | las cuatro variantes de `PaymentFrequency` |

`Liquidacion::total_neto()` con adelantos mayores al bruto merece una mención: el resultado es
negativo y el sistema **no** lo recorta a cero. Un empleado que pidió más adelantos que lo que
generó en el período tiene saldo en contra, y esconderlo sería falsear la caja.

### 2.4 Máquinas de estado

Para cada una de las tres máquinas de doc 08, dos tests generados por tabla:

```rust
#[rstest]
#[case(EstadoFactura::Borrador,  EstadoFactura::Emitida,  true)]
#[case(EstadoFactura::Borrador,  EstadoFactura::Pagada,   false)]
#[case(EstadoFactura::Pagada,    EstadoFactura::Borrador, false)]
// … la tabla completa de doc 08 §2, transición por transición
fn factura_transicion(#[case] from: EstadoFactura, #[case] to: EstadoFactura, #[case] ok: bool) { … }
```

Más un test de completitud: `toda_combinacion_de_estados_esta_cubierta` recorre el producto cartesiano
de las variantes y verifica que la tabla de casos las contemple todas. Sin ese test, agregar una
variante al enum deja transiciones sin probar.

---

## 3. Aplicación: `eo-application`

Los casos de uso se testean con los puertos sustituidos por dobles. No hay base de datos.

### 3.1 Dobles

`mockall` sobre los traits de repositorio. Un caso de uso recibe sus dependencias por parámetro, así
que el test las construye sin contenedor de inyección.

```rust
#[tokio::test]
async fn crear_movimiento_rechaza_monto_cero() {
    let mut repo = MockMovimientoRepository::new();
    repo.expect_add().never();                       // la aserción importante

    let uc = CrearMovimiento::new(Arc::new(repo), fixed_clock(), Arc::new(MockUnitOfWork::new()));

    let err = uc.execute(dto_con_monto_cero()).await.unwrap_err();

    assert_matches!(err, AppError::Validation(v) if v.has_field("monto"));
}
```

`repo.expect_add().never()` es la parte que hace útil el test: verifica que la validación corta
**antes** de tocar la persistencia. Un test que sólo mire el `Err` pasaría igual con una
implementación que escribe y después falla.

### 3.2 El reloj es un puerto

Ningún test depende de la hora real. `fixed_clock()` devuelve un `Clock` que siempre da el mismo
instante (doc 04 §3.6). Los tests de "factura vencida" fijan el reloj, no calculan contra `now()`.

Test explícito: `ningun_caso_de_uso_usa_utc_now` recorre el crate buscando `Utc::now()` fuera del
adaptador del reloj.

### 3.3 Cobertura por caso de uso

Cada caso de uso de doc 06 tiene como mínimo:

1. El camino feliz.
2. Un test por cada regla de validación que le aplica.
3. Un test por cada regla de negocio que puede rechazar la operación.
4. Un test de que la transacción se revierte si algo falla después de la primera escritura.

### 3.4 Los tests que no pueden faltar

Son las fórmulas de negocio. Si estas fallan, el sistema miente sobre el dinero.

| Test | Escenario |
| --- | --- |
| `sugerencia_liquidacion_manual` | rama 1 de doc 06 §6.6: días y tarifa dados a mano |
| `sugerencia_liquidacion_desde_asistencia` | rama 2: cuenta jornadas con los factores 1.0 / 0.5 / 0.0 |
| `sugerencia_liquidacion_por_calendario` | rama 3: itera el rango de fechas |
| `sugerencia_prioridad_feriado_sobre_domingo` | un domingo feriado usa el multiplicador de feriado |
| `sugerencia_prioridad_domingo_sobre_sabado` | verifica el orden completo de la cascada |
| `sugerencia_ignora_dias_excluidos` | `incluir_sabados = false` no cuenta los sábados |
| `sugerencia_suma_solo_adelantos` | filtra por el GUID `…0003` y no por nombre |
| `sugerencia_no_reusa_un_adelanto_ya_liquidado` | INV-05 |
| `sugerencia_media_jornada_cuenta_medio_dia` | `dias_trabajados` da `0.5` |
| `certificacion_subtotal_actual` | `cantidad × precio × (porcentaje / 100)` |
| `certificacion_subtotal_acumulado` | usa `anterior + actual` |
| `certificacion_ajuste_uocra_es_porcentaje` | y no un monto, que es el error fácil |
| `certificacion_total_neto` | `total − ajuste − otros` |
| `certificacion_rechaza_acumulado_mayor_a_cien` | la validación que el sistema anterior no tenía |
| `rentabilidad_por_obra` | `ingresos − gastos`, margen a 2 decimales |
| `rentabilidad_con_ingresos_cero_no_divide_por_cero` | el margen es `0`, no `NaN` |
| `antiguedad_buckets` | 0-30 / 31-60 / 61-90 / +90, con los bordes exactos en 30, 31, 60, 61, 90, 91 |
| `cuenta_corriente_saldo` | facturado − cobrado por cliente |
| `dashboard_comparacion_periodo_anterior` | el período anterior tiene la misma cantidad de días |
| `dashboard_facturas_vencidas_a_treinta_dias` | el umbral configurable |
| `factura_total_es_subtotal_mas_iva` | sin IVA automático |
| `pago_parcial_cambia_estado_a_pagada_parcial` | doc 08 §2 |
| `pago_que_completa_cambia_estado_a_pagada` | |
| `asistencia_upsert_no_duplica` | dos cargas del mismo día y empleado dan una fila |

Los bordes de `antiguedad_buckets` se prueban valor por valor a propósito: un `<=` en lugar de un `<`
en un bucket es un error que no rompe nada visiblemente y desplaza plata de una columna a otra.

---

## 4. Infraestructura

### 4.1 Repositorios

Contra SQLite **en memoria** (`sqlite::memory:`), con las migraciones aplicadas al inicio de cada
test. Cada test tiene su propia base; no hay estado compartido.

```rust
async fn test_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    db
}
```

| Test | Verifica |
| --- | --- |
| `soft_delete_excluye_de_las_consultas` | un registro borrado no aparece en el listado |
| `soft_delete_no_borra_la_fila` | sigue estando con `is_deleted = 1` |
| `row_version_incrementa_al_actualizar` | |
| `update_con_row_version_vieja_falla` | el conflicto optimista |
| `paginacion_devuelve_el_total_correcto` | el `total` es el de la consulta sin paginar |
| `filtro_de_texto_es_case_insensitive` | |
| `filtro_por_rango_incluye_los_bordes` | |
| `orden_por_fecha_es_cronologico` | aprovecha el formato de 24 caracteres |
| `unit_of_work_revierte_todo` | dos escrituras, la segunda falla, ninguna queda |
| `unit_of_work_commitea_todo` | |

### 4.2 Migraciones

| Test | Verifica |
| --- | --- |
| `migraciones_aplican_desde_cero` | `Migrator::up` sobre una base vacía |
| `migraciones_son_reversibles` | `up` y después `down` deja el esquema vacío |
| `esquema_coincide_con_el_documento` | el `sqlite_master` resultante coincide con un snapshot de `insta`, que se compara a mano contra doc 03 |
| `seed_inserta_los_tipos_de_sistema` | los 4 GUID de `…0001` a `…0004`, con `es_ingreso` correcto |
| `seed_es_idempotente` | aplicar dos veces no duplica |
| `foreign_keys_estan_activas` | `PRAGMA foreign_keys` devuelve 1 |
| `todo_on_delete_coincide_con_el_documento` | lee el `sqlite_master` y compara cada FK contra la tabla de doc 03 §4 |

El último es el que atrapa el error más caro: un `CASCADE` donde debía ir `RESTRICT` borra datos en
silencio y no se nota hasta que falta algo.

### 4.3 Servicios HTTP

Con `wiremock`, que levanta un servidor local. **Ningún test toca la red real.**

| Test | Escenario |
| --- | --- |
| `dolar_parsea_la_respuesta_esperada` | el JSON documentado en doc 13 §2.2 |
| `dolar_timeout_devuelve_lista_vacia` | el servidor demora más que el timeout |
| `dolar_error_500_devuelve_lista_vacia` | degradación silenciosa |
| `dolar_json_malformado_devuelve_lista_vacia` | y loguea el error |
| `dolar_usa_la_cache_dentro_de_la_ventana` | dos llamadas seguidas, una sola petición |
| `dolar_refresca_al_expirar_la_cache` | |
| `feriados_parsea_la_respuesta_esperada` | |
| `feriados_de_la_api_no_sobreescriben_los_manuales` | el `INSERT OR IGNORE` de doc 03 §3.21 |
| `sin_conexion_la_app_funciona` | todas las operaciones de negocio andan con las dos APIs caídas |

El último es una declaración de diseño: las APIs externas son un adorno. Si el sistema no funciona
sin internet, está mal hecho.

### 4.4 Reportes

Los reportes se testean por caracterización, no comparando bytes contra un archivo de referencia:
QuestPDF y su equivalente en Rust cambian los bytes entre versiones sin cambiar el contenido.

| Formato | Cómo se testea |
| --- | --- |
| CSV | comparación de texto exacta contra un snapshot de `insta`. Es determinista. |
| JSON | ídem, con las claves ordenadas |
| XLSX | se abre el resultado, se leen las celdas, y se comparan valores y tipos celda por celda |
| DOCX | se extrae el texto y se comparan los párrafos |
| PDF | se extrae el texto con una librería de extracción y se verifica que aparezcan los valores esperados, en el orden esperado |

Para el PDF no se compara el layout visual. Lo que se verifica es lo que importa:

| Test | Verifica |
| --- | --- |
| `pdf_liquidacion_lista_cada_adelanto_con_su_fecha` | RC-02, el requerimiento explícito del cliente |
| `pdf_liquidacion_total_neto_coincide_con_el_dominio` | el número del PDF es el mismo que el de la base |
| `pdf_certificado_tiene_las_nueve_columnas` | |
| `pdf_certificado_es_landscape` | |
| `pdf_usa_el_contratista_de_configuracion` | y no el literal `"PABLO BAEZ"` del sistema anterior |
| `pdf_sin_logo_configurado_no_falla` | |
| `reporte_vacio_genera_un_archivo_valido` | cero filas no produce un archivo corrupto |
| `reporte_con_mil_filas_no_excede_el_tiempo_limite` | |

`reporte_vacio` existe porque exportar una lista filtrada a nada es un caso real y frecuente.

### 4.5 Archivos y backup

Con `tempfile`, sobre un directorio temporal. Nunca sobre el directorio de datos real.

| Test | Verifica |
| --- | --- |
| `adjunto_se_guarda_en_la_ruta_convenida` | `{tipo}/{id}/{uuid}_{nombre}` |
| `adjunto_rechaza_mime_fuera_de_la_whitelist` | |
| `adjunto_rechaza_tamano_excesivo` | |
| `adjunto_sanea_el_nombre_de_archivo` | `../../etc/passwd` no escapa del directorio |
| `adjunto_borrado_va_a_la_papelera` | y no se elimina |
| `backup_produce_un_archivo_valido` | se abre y se consulta |
| `backup_verifica_integridad` | `PRAGMA integrity_check` da `ok` |
| `restore_cierra_la_conexion_antes_de_copiar` | el bug de doc 13 §4.3 |
| `restore_elimina_wal_y_shm` | |
| `export_json_incluye_version_y_timestamp` | |
| `import_json_valida_los_identificadores` | un nombre de tabla que no está en la allowlist se rechaza |
| `import_json_rechaza_inyeccion_en_el_nombre_de_columna` | el vector de ataque de doc 13 §5.2 |

`adjunto_sanea_el_nombre_de_archivo` y los dos de inyección son tests de seguridad, no de
funcionalidad. Van con casos hostiles explícitos, no con nombres válidos.

---

## 5. Comandos Tauri

Cada comando de doc 11 tiene un test de contrato. No se testea la lógica otra vez: se verifica que el
comando sea una capa fina que traduce bien.

| Test | Verifica |
| --- | --- |
| `comando_devuelve_el_dto_esperado` | la forma del JSON serializado |
| `comando_traduce_AppError_a_ApiError` | el `kind` y el `code` correctos |
| `comando_incluye_trace_id_en_el_error` | |
| `comando_no_filtra_detalle_interno` | un error de base no expone el SQL en `code` |

Más dos tests globales:

- `todo_comando_esta_registrado`: recorre las funciones marcadas con `#[tauri::command]` y verifica
  que estén en el `invoke_handler`. Un comando sin registrar falla en tiempo de ejecución con un
  mensaje poco claro; este test lo atrapa al compilar los tests.
- `todo_comando_esta_en_la_capa_api_del_frontend`: compara la lista de comandos de Rust con los
  nombres invocados en `src/api/`. Detecta las dos direcciones: un comando sin usar y una llamada a
  un comando que no existe.

---

## 6. Frontend

### 6.1 Composables y utilidades

Con `vitest`, sin montar componentes.

| Test | Verifica |
| --- | --- |
| `useMoney_format_respeta_la_configuracion` | separadores y decimales |
| `useMoney_roundtrip` | `parse(format(x))` conserva el valor |
| `useMoney_no_pierde_precision_en_importes_grandes` | el caso que justifica usar string y no número |
| `useServerTable_debounce_agrupa_las_teclas` | tres cambios rápidos producen una petición |
| `useServerTable_cancela_las_respuestas_viejas` | una respuesta fuera de orden se descarta |
| `useServerTable_resetea_la_pagina_al_filtrar` | |
| `useServerTable_no_resetea_la_pagina_al_ordenar` | |
| `useServerTable_conserva_las_filas_ante_un_error` | |
| `useServerTable_persiste_el_tamano_de_pagina` | |
| `useApiError_validacion_no_muestra_toast` | |
| `useApiError_clave_faltante_cae_al_mensaje_generico` | y loguea |
| `useCrudDrawer_no_cierra_ante_error_de_validacion` | |
| `useCrudDrawer_pide_confirmacion_si_hay_cambios` | |
| `useShortcuts_no_dispara_dentro_de_un_input` | |

Los cuatro de `useServerTable` sobre debounce, cancelación y reset son los que evitan los bugs de
lista más comunes.

### 6.2 Componentes

Con `@vue/test-utils` y `jsdom`. Se testean los componentes de `domain/`, no las vistas completas.

| Componente | Tests |
| --- | --- |
| `MoneyText` | formatea, aplica el color del signo, muestra cero sin signo |
| `MoneyInput` | emite el string de 4 decimales, no un número |
| `DateInput` | emite `YYYY-MM-DD` en modo civil |
| `StatePill` | la etiqueta sale de i18n, el color del token |
| `PercentBar` | un valor sobre 100 se marca y no se recorta |
| `ListState` | los cuatro estados, y la distinción vacío / sin resultados |
| `CrudDrawer` | el foco entra al abrir y vuelve al cerrar |
| `FilterBar` | el botón de limpiar emite el evento |
| `DataGrid` | conecta los eventos de página y orden |

Las vistas no se testean componente por componente: son composición de piezas ya testeadas más
llamadas al store. Testearlas duplica esfuerzo y se rompe con cada cambio de layout. Lo que las cubre
es el checklist manual de §9.

### 6.3 Tests de arquitectura

Los 14 tests de doc 16 §8. Son los que impiden la degradación: colores literales, textos sin
traducir, aritmética de importes en la vista, dependencias en el sentido equivocado.

---

## 7. Datos de prueba

### 7.1 Constructores, no fixtures globales

Nada de una base de datos de prueba compartida. Cada test construye lo que necesita con
constructores encadenables:

```rust
let empleado = EmpleadoBuilder::new()
    .tarifa_diaria(Money::from_decimal_str("40000.0000").unwrap())
    .pago_frecuencia(PaymentFrequency::Quincenal)
    .build();

let asistencias = AsistenciaBuilder::rango(fecha(2026, 8, 1), fecha(2026, 8, 15))
    .todos(TipoJornada::Completa)
    .excepto(fecha(2026, 8, 5), TipoJornada::Media)
    .excepto(fecha(2026, 8, 10), TipoJornada::Falta)
    .build();
```

El constructor tiene valores por defecto válidos para todo campo. Un test que sólo le importa la
tarifa escribe sólo la tarifa. Así un campo nuevo en una entidad no rompe cien tests.

### 7.2 Valores de referencia

Los tests de fórmulas usan un conjunto fijo de cifras, calculadas **a mano** y documentadas en el
test. No se generan con la misma función que se está probando.

```rust
/// Quincena de 15 días corridos, tarifa 40.000, 12 jornadas completas y 2 medias.
/// Un sábado trabajado con multiplicador 1,5.
///
/// Bruto = (12 × 40.000) + (2 × 0,5 × 40.000) + (1 × 1,5 × 40.000)
///       = 480.000 + 40.000 + 60.000 = 580.000
/// Adelantos = 150.000 + 80.000 = 230.000
/// Neto = 580.000 − 230.000 = 350.000
#[tokio::test]
async fn liquidacion_de_referencia() { … }
```

El comentario con la cuenta desarrollada es obligatorio en todo test de fórmula. Sin él, cuando el
test falla nadie sabe si el error está en el código o en el número esperado.

### 7.3 Determinismo

| Fuente de indeterminismo | Cómo se elimina |
| --- | --- |
| Hora actual | `Clock` fijo (§3.2) |
| Generación de UUID | puerto `IdGenerator` con una implementación secuencial en los tests |
| Orden de un `HashMap` | se ordena antes de comparar, o se usa `BTreeMap` |
| Zona horaria de la máquina | todo es UTC; los tests no leen la zona del sistema |
| Red | `wiremock`, nunca la red real |
| Sistema de archivos | `tempfile`, nunca el directorio de datos real |
| Locale del sistema | el formateo lee la configuración, no `CultureInfo` |

Un test que falla una vez cada veinte corridas es peor que no tenerlo: entrena a ignorar los fallos.
Un test intermitente se arregla o se borra, nunca se reintenta.

---

## 8. Cobertura y ejecución

### 8.1 Comandos

```bash
cargo test --workspace                 # todo el backend
cargo test -p eo-domain                # una capa
cargo llvm-cov --workspace --lcov --output-path lcov.info

pnpm test                              # frontend
pnpm test:coverage
```

### 8.2 Umbrales

| Crate / área | Mínimo de líneas | Motivo |
| --- | --- | --- |
| `eo-domain` | 95 % | es aritmética pura, no hay excusa |
| `eo-application` | 85 % | los casos de uso son la lógica de negocio |
| `eo-infrastructure` | 70 % | tiene adaptadores con poco valor de test |
| `src-tauri` | 60 % | capa fina |
| `eo-import-legacy` | 85 % | corre una sola vez y no hay segunda oportunidad |
| Frontend `composables/` | 85 % | |
| Frontend `components/domain/` | 70 % | |

Los umbrales se aplican en CI (doc 18). Bajar un umbral requiere justificarlo en el commit.

Sobre la cobertura: es un piso, no un objetivo. Un módulo con 95 % de cobertura y sin los tests de
§3.4 está peor testeado que uno con 80 % y todas las fórmulas cubiertas. La lista de tests
obligatorios manda sobre el porcentaje.

### 8.3 Velocidad

| Suite | Límite |
| --- | --- |
| `cargo test -p eo-domain` | 5 s |
| `cargo test --workspace` | 60 s |
| `pnpm test` | 30 s |

Si la suite tarda más que eso deja de ejecutarse durante el desarrollo, y una suite que no se ejecuta
no sirve. Los tests de repositorio usan SQLite en memoria justamente por esto.

---

## 9. Checklist manual

Lo que no está automatizado, y hay que verificar a mano antes de una release. Es corto a propósito.

| # | Verificación |
| --- | --- |
| 1 | Arranque en frío con el directorio de datos vacío: crea la base, aplica migraciones y abre el dashboard |
| 2 | Arranque sin conexión: no hay errores visibles, la cotización muestra el estado no disponible |
| 3 | Recorrer las 15 rutas del menú: ninguna queda en blanco ni tira error |
| 4 | Alta, edición y borrado en cada módulo CRUD |
| 5 | Los dos temas, claro y oscuro, en cada pantalla: ningún texto ilegible |
| 6 | La aplicación en inglés: ninguna etiqueta en español |
| 7 | Redimensionar a 1024×768: ninguna pantalla con scroll horizontal |
| 8 | Generar cada uno de los exports y abrir el archivo resultante |
| 9 | Crear un backup, restaurarlo, verificar que los datos están |
| 10 | Recorrer los atajos de doc 10 §4 |
| 11 | Una liquidación completa de punta a punta, comparando el PDF contra la cuenta hecha a mano |
| 12 | Un certificado con avance parcial, verificando el acumulado contra el certificado anterior |

Los puntos 11 y 12 son los importantes: son los dos flujos donde un error cuesta dinero real.
