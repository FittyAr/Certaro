# 12 — Reportes y exportaciones

> Define `crates/eo-infrastructure/src/reporting/`. Cada layout está transcripto al detalle: tamaño de
> página, columnas, anchos, alineaciones y formatos. El implementador no debe inventar ni una fuente.

## 1. Stack y reglas

| Formato | Crate | Notas |
| --- | --- | --- |
| PDF | `printpdf` + `genpdf`, o `typst` como generador | ver §1.1 |
| XLSX | `rust_xlsxwriter` | soporta formatos numéricos, congelado y autofiltro |
| DOCX | `docx-rs` | |
| CSV | `csv` | |
| JSON | `serde_json` | |

### 1.1 Decisión sobre PDF

El sistema anterior usaba **QuestPDF**, con una API declarativa de filas, columnas y celdas. En Rust
no hay un equivalente directo. Se usa `genpdf` sobre `printpdf` con un módulo propio de tablas
(`reporting/pdf/table.rs`) que provea:

- columnas de ancho **relativo** (proporción) y **constante** (puntos),
- combinación de celdas horizontal y vertical,
- estilo por celda: fondo, bordes con grosor y color, relleno, alineación,
- encabezado que se repite en cada página,
- pie con número de página.

Es la pieza de infraestructura más costosa del rewrite y hay que construirla antes de los reportes.
Si `genpdf` resulta insuficiente para las celdas combinadas del certificado (§4), la alternativa es
generar el documento con `typst` como motor de composición y plantillas `.typ` versionadas en
`assets/templates/`. La decisión se toma al implementar §4 y se registra acá.

### 1.2 Reglas comunes

1. **Ningún texto del reporte es un literal en el código.** Todos los rótulos son claves i18n
   resueltas con el idioma configurado. **[FIX]** El sistema anterior tenía todos los rótulos
   hardcodeados en español dentro del generador.
2. **Ningún dato de la empresa es un literal.** Nombre, contratista, logo, dirección y CUIT salen de
   configuración (doc 14, sección Negocio). **[FIX]** El certificado tenía hardcodeados
   `"PABLO BAEZ"`, `"GENERCON"` y `"ENERGIA CONTROLADA"`; la liquidación tenía `"Cuentas Claras"` y
   `"Software de Gestión Profesional"`.
3. **Todo importe se formatea con una única función**, `format_money(money, &LocaleConfig)`, que
   aplica el símbolo, el separador de miles y los decimales de la configuración regional.
   El formato `"C"` de .NET equivale a **símbolo + 2 decimales** con la cultura del sistema; en el
   nuevo sistema los decimales visibles son configurables (`Locale.DecimalesMoneda`, default 2) y
   la cultura es explícita, no la del sistema operativo.
4. **Toda fecha se formatea con `format_date`**, con el patrón de configuración
   (`Locale.FormatoFecha`, default `dd/MM/yyyy`).
5. La fuente es **una sola**, embebida en el binario, con soporte completo de latín extendido:
   **Inter** para pantalla y reportes. Se declara en `Report.Font` y sus tamaños en tokens, no
   dispersos por el layout. **[FIX]** El generador anterior no declaraba fuente y quedaba la de
   QuestPDF (Lato) en PDF, Calibri en Word y Calibri 11 en Excel: tres tipografías distintas para
   el mismo dato.
6. El alcance de los datos exportados es **siempre el filtro activo, sin paginación** (doc 09 §3.2).
7. El destino lo elige el usuario con el diálogo del sistema. **[FIX]** El PDF de liquidación se
   guardaba directamente en el Escritorio sin preguntar.
8. Toda exportación emite `export:progress` cada 500 filas y devuelve `ExportResult`.

### 1.3 Nombres de archivo propuestos

| Reporte | Patrón |
| --- | --- |
| Movimientos | `Movimientos_{yyyyMMdd_HHmmss}.{ext}` |
| Caja por período | `Caja_{desde:yyyyMMdd}_{hasta:yyyyMMdd}.{ext}` |
| Rentabilidad | `Rentabilidad_{desde:yyyyMMdd}_{hasta:yyyyMMdd}.{ext}` |
| Cuenta corriente | `CuentaCorriente_{cliente}_{yyyyMMdd}.{ext}` |
| Antigüedad de deuda | `AntiguedadDeuda_{corte:yyyyMMdd}.{ext}` |
| Liquidación | `Liquidacion_{empleado}_{hasta:yyyyMMdd}.pdf` |
| Certificado | `Certificado_{obra}_{numero}_{fecha:yyyyMMdd}.pdf` |
| Asistencia | `Asistencia_{yyyy}-{MM}.{ext}` |
| Base completa | `Certaro_export_{yyyyMMdd_HHmmss}.json` |

Los componentes que vienen de datos del usuario se sanean: se reemplaza todo carácter inválido para
nombre de archivo por `_`, se colapsan los `_` repetidos y se recorta a 60 caracteres. Si queda
vacío, se usa el identificador.

## 2. Reporte de movimientos

### 2.1 PDF

| Propiedad | Valor |
| --- | --- |
| Tamaño | A4 |
| Orientación | vertical |
| Márgenes | 1 cm en los cuatro lados |
| Fondo | blanco |
| Cuerpo | 10 pt |

**Encabezado**, repetido en cada página:

| Línea | Contenido | Formato |
| --- | --- | --- |
| 1 | `Report.Movimientos.Title` + nombre de la empresa | 20 pt, semibold, color primario |
| 2 | rango de fechas y filtros activos, en prosa | 8 pt, gris |
| 3 | cantidad de registros | 8 pt, gris |

**[FIX]** El encabezado anterior era una sola línea (`"Listado de Movimientos - Certaro"`) sin
indicar qué filtros se habían aplicado, así que un PDF impreso no decía qué mostraba.

**Tabla**, 7 columnas. El sistema anterior tenía **4** (`Fecha`, `Concepto`, `Tipo`, `Monto`, donde
la columna rotulada «Monto» mostraba en realidad el **total**), lo que ocultaba el desglose:

| # | Ancho | Rótulo (clave) | Alineación | Formato |
| --- | --- | --- | --- | --- |
| 1 | relativo 2 | `Report.Col.Fecha` | izquierda | `dd/MM/yyyy` |
| 2 | relativo 5 | `Report.Col.Concepto` | izquierda | texto |
| 3 | relativo 2 | `Report.Col.Tipo` | izquierda | texto |
| 4 | relativo 2 | `Report.Col.Categoria` | izquierda | texto |
| 5 | relativo 2 | `Report.Col.Monto` | derecha | moneda |
| 6 | relativo 1 | `Report.Col.Cantidad` | derecha | número, 4 decimales significativos |
| 7 | relativo 2 | `Report.Col.Total` | derecha | moneda, semibold |

Estilo del encabezado de la tabla: semibold, relleno vertical 5 pt, borde inferior de 1 pt negro.
Estilo de las celdas: relleno vertical 5 pt, borde inferior de 1 pt gris claro. Filas alternadas con
fondo gris muy claro.

**Pie de tabla**, en la última página: `Report.Total.Ingresos`, `Report.Total.Gastos` y
`Report.Total.Balance`, cada uno con su importe alineado a la derecha, semibold, con el balance en
verde si es positivo y rojo si es negativo. **[NUEVO]**: el PDF anterior no totalizaba nada.

**Pie de página**, en todas: a la izquierda la fecha y hora de generación
(`dd/MM/yyyy HH:mm`), al centro el nombre de la empresa, a la derecha
`Report.Footer.Page` con `{actual}` y `{total}`.

**[FIX]** El pie anterior era `"Página {n}"` sin el total, y no incluía la fecha de generación.

### 2.2 XLSX

| Propiedad | Valor |
| --- | --- |
| Hoja | `Report.Sheet.Movimientos` |
| Fila 1 | título del reporte, combinada sobre todas las columnas, 14 pt negrita |
| Fila 2 | filtros aplicados, 9 pt gris |
| Fila 3 | vacía |
| Fila 4 | encabezados |
| Fila 5+ | datos |
| Congelado | en `A5` |
| Autofiltro | sobre el rango de encabezados y datos |
| Ancho | ajustado al contenido, con mínimo 10 y máximo 60 caracteres |

Columnas, 13 — el sistema anterior tenía 6:

| # | Rótulo | Tipo de celda | Formato numérico |
| --- | --- | --- | --- |
| 1 | Fecha | fecha | `dd/mm/yyyy` |
| 2 | Concepto | texto | — |
| 3 | Tipo | texto | — |
| 4 | Categoría | texto | — |
| 5 | Cliente | texto | — |
| 6 | Obra | texto | — |
| 7 | Trabajo | texto | — |
| 8 | Moneda | texto | — |
| 9 | Monto | número | `#,##0.00` |
| 10 | Cantidad | número | `#,##0.####` |
| 11 | Unidad | texto | — |
| 12 | Total | número | `#,##0.00`, negrita |
| 13 | Observaciones | texto | — |

**[FIX]** El XLSX anterior escribía los importes como números sin formato, así que Excel los mostraba
con los decimales que le parecía. Ahora cada columna monetaria declara su formato.

Última fila: totales con `SUBTOTAL(109; rango)` en las columnas 9 y 12, para que el total responda al
autofiltro. **[NUEVO]**

Se agrega una segunda hoja `Report.Sheet.Resumen` **[NUEVO]** con el total de ingresos, gastos y
balance, y un desglose por tipo y por categoría.

### 2.3 DOCX

| Propiedad | Valor |
| --- | --- |
| Tamaño | A4 vertical |
| Márgenes | 2 cm |
| Título | centrado, 20 pt, negrita |
| Subtítulo | centrado, 10 pt, gris: rango y filtros |

Tabla con las **mismas 7 columnas del PDF** y los mismos formatos, con bordes finos y encabezado con
fondo gris claro repetido en cada página. Al final, el bloque de totales.

**[FIX]** El DOCX anterior tenía 4 columnas (distintas de las 6 del XLSX y de las 4 del PDF: tres
formatos con tres conjuntos de columnas para el mismo reporte), un ancho de tabla fijo de 5000 twips
que no se adapta a la página, y ningún pie.

Pie de página con la fecha de generación y la numeración.

### 2.4 CSV

| Propiedad | Valor |
| --- | --- |
| Separador | `,` |
| Encoding | UTF-8 **con BOM** |
| Fin de línea | `\r\n` |
| Encabezado | sí, con los rótulos traducidos |
| Comillas | se entrecomilla si el valor contiene `,`, `"`, `\n` o `\r`; las comillas internas se duplican |
| Fechas | `yyyy-MM-dd` (ISO) |
| Números | punto decimal, sin separador de miles, 4 decimales para cantidades y 2 para importes |

**[FIX]** Tres cambios respecto del sistema anterior:

- **BOM**: sin BOM, Excel en Windows abre el CSV en la página de códigos local y rompe todo acento.
  Es el problema más reportado de cualquier CSV exportado en español.
- **Fin de línea fijo `\r\n`**: antes usaba el del sistema operativo, así que el mismo export daba
  archivos distintos según la máquina.
- **Fechas ISO**: antes usaba `dd/MM/yyyy`, que Excel interpreta como mes/día según la
  configuración regional y da fechas equivocadas en silencio.

Las mismas 13 columnas del XLSX.

### 2.5 JSON

```json
{
  "version": 1,
  "generadoEn": "2026-08-29T12:34:56Z",
  "reporte": "Movimientos",
  "filtro": { "fechaDesde": "2026-08-01", "fechaHasta": "2026-08-31" },
  "resumen": {
    "totalIngresos": "1234567.8900",
    "totalGastos": "234567.1200",
    "balance": "1000000.7700",
    "cantidad": 342
  },
  "items": [
    {
      "id": "0192f3a0-1234-7abc-8def-0123456789ab",
      "fecha": "2026-08-14",
      "concepto": "Cable 2.5 mm",
      "monto": "1500.0000",
      "cantidad": "100.0000",
      "total": "150000.0000",
      "tipoMovimiento": "Gasto",
      "categoria": "Materiales",
      "moneda": "Ars"
    }
  ]
}
```

Reglas: `camelCase`, indentación de 2 espacios, importes como **string** con 4 decimales, fechas
civiles como `YYYY-MM-DD`, instantes en RFC 3339 UTC, enums por nombre, campos nulos omitidos.

**[FIX]** Cuatro problemas del JSON anterior: era PascalCase, los importes eran números de punto
flotante, los enums viajaban como entero sin diccionario, y había **dos** implementaciones — la del
servicio con indentación (que nadie llamaba) y la de los ViewModels sin indentación (la que se usaba
de verdad).

## 3. PDF de liquidación

Es el documento que el cliente pidió explícitamente (RC-02: cada adelanto con su fecha). Se conserva
la estructura del sistema anterior, que estaba bien pensada, cambiando los literales por claves y
configuración.

| Propiedad | Valor |
| --- | --- |
| Tamaño | A4 |
| Orientación | vertical |
| Márgenes | 1,5 cm |
| Cuerpo | 11 pt, gris oscuro |

### 3.1 Encabezado

Dos columnas.

**Izquierda:**

| Línea | Contenido | Formato |
| --- | --- | --- |
| 1 | `Report.Liquidacion.Title` | 24 pt, semibold, color primario |
| 2 | `Report.Liquidacion.Empleado` + nombre | 14 pt, semibold |
| 3 | `Report.Liquidacion.Periodo` con desde y hasta | 10 pt |
| 4 | **[NUEVO]** documento y cargo del empleado | 9 pt, gris |

**Derecha**, alineada a la derecha:

| Línea | Contenido | Formato |
| --- | --- | --- |
| 1 | nombre de la empresa, en mayúsculas | 12 pt, semibold |
| 2 | lema de la empresa, desde configuración | 10 pt, cursiva |
| 3 | fecha y hora de generación, `dd/MM/yyyy HH:mm` | 8 pt |

**[FIX]** El lema estaba hardcodeado como `"Cuentas Claras"`. Pasa a `Business.Lema`, con
`Report.DefaultLema` como valor inicial.

### 3.2 Sección «Resumen de trabajo»

Título: `Report.Liquidacion.SectionResumen`, semibold, subrayado, 10 pt de separación abajo.

Tabla de 4 columnas:

| # | Ancho | Rótulo | Alineación |
| --- | --- | --- | --- |
| 1 | relativo 3 | `Report.Col.Concepto` | izquierda |
| 2 | relativo 1 | `Report.Col.Dias` | derecha |
| 3 | relativo 2 | `Report.Col.Tarifa` | derecha |
| 4 | relativo 2 | `Report.Col.Subtotal` | derecha |

Filas:

| Concepto | Días | Tarifa | Subtotal |
| --- | --- | --- | --- |
| `Report.Liquidacion.DiasTrabajados` | días, 1 decimal | tarifa, moneda | bruto base, moneda, semibold |
| **[NUEVO]** `Report.Liquidacion.RecargoSabado` | días de sábado | multiplicador | importe del recargo |
| **[NUEVO]** `Report.Liquidacion.RecargoDomingo` | días de domingo | multiplicador | importe |
| **[NUEVO]** `Report.Liquidacion.RecargoFeriado` | días de feriado | multiplicador | importe |

Las filas de recargo sólo aparecen si su importe es distinto de cero.

**[FIX]** El PDF anterior tenía **una sola fila** con el total bruto, sin desglosar los recargos.
El empleado veía un número y no podía verificarlo, que es exactamente el problema que el cliente
describe en RC-02 sobre la confianza en los números.

Estilo del encabezado: semibold, relleno vertical 5 pt, borde inferior 1 pt negro.
Estilo de las celdas: relleno vertical 5 pt, borde inferior 1 pt gris claro.

### 3.3 Sección «Detalle de adelantos»

Título: `Report.Liquidacion.SectionAdelantos`, semibold, subrayado, en rojo. Separación: 20 pt
arriba, 10 pt abajo.

Tabla de 4 columnas — el sistema anterior tenía 3:

| # | Ancho | Rótulo | Alineación | Formato |
| --- | --- | --- | --- | --- |
| 1 | relativo 2 | `Report.Col.Fecha` | izquierda | `dd/MM/yyyy` |
| 2 | relativo 4 | `Report.Col.Concepto` | izquierda | texto |
| 3 | relativo 2 | `Report.Col.Tipo` **[NUEVO]** | izquierda | concepto de pago |
| 4 | relativo 2 | `Report.Col.Monto` | derecha | moneda |

Sin adelantos: una fila combinada sobre las 4 columnas, centrada, en cursiva, con
`Report.Liquidacion.SinAdelantos`.

Pie de la tabla: `Report.Liquidacion.TotalAdelantos` combinado sobre las 3 primeras columnas,
alineado a la derecha y semibold; en la cuarta, el total en rojo y semibold.

Cada adelanto se lista con su fecha **exacta**, sin agrupar ni redondear. Es el requisito literal del
cliente.

### 3.4 Bloque de totales

Contenedor de 200 pt de ancho, alineado a la derecha, con fondo gris muy claro y 10 pt de relleno,
separado 30 pt del bloque anterior.

| Fila | Izquierda | Derecha |
| --- | --- | --- |
| 1 | `Report.Liquidacion.Subtotal` | bruto, moneda |
| 2 | `Report.Liquidacion.Adelantos` | `- {importe}`, en rojo |
| 3 (con borde superior) | `Report.Liquidacion.TotalAPagar`, 14 pt semibold | neto, 14 pt semibold, verde si es positivo, **rojo si es negativo** |

**[FIX]** El total se pintaba siempre en verde. Un neto negativo (el empleado retiró más de lo
devengado) es un caso real y se pintaba de color de cosa buena.

### 3.5 Observaciones y firmas

Si hay observaciones: 20 pt de separación, `Report.Liquidacion.Observaciones` en semibold seguido del
texto.

Firmas: 60 pt de separación, dos bloques con línea superior a 40 pt y texto centrado —
`Report.Liquidacion.FirmaRevision` a la izquierda y `Report.Liquidacion.FirmaAdministracion` a la
derecha, separados por 100 pt.

### 3.6 Pie de página

Centrado: nombre de la empresa, separador, `Report.Footer.Page` con página actual y total.

## 4. PDF de certificado

El documento de certificación de avance de obra (RC-10). Replica una planilla que el cliente ya usa
en papel, así que la estructura de columnas **no se cambia**.

| Propiedad | Valor |
| --- | --- |
| Tamaño | A4 |
| Orientación | **horizontal** — es lo que permite que entren las 9 columnas |
| Márgenes | 1 cm |
| Cuerpo | 9 pt, negro |

### 4.1 Encabezado

Recuadro con borde de 1 pt, dividido en dos columnas.

**Izquierda** (proporción 3), con relleno de 5 pt, tres filas de rótulo + valor. El rótulo va en
semibold con ancho constante:

| Rótulo (clave) | Ancho del rótulo | Valor |
| --- | --- | --- |
| `Report.Certificado.Obra` | 50 pt | nombre de la obra; si no tiene, `Report.Certificado.ObraGeneral` |
| `Report.Certificado.Ref` | 50 pt | título de la orden de trabajo |
| `Report.Certificado.Contratista` | 60 pt | contratista, **desde configuración** |
| **[NUEVO]** `Report.Certificado.Cliente` | 60 pt | nombre del cliente |

**Derecha** (proporción 2), con borde izquierdo de 1 pt y relleno de 5 pt:

| Elemento | Contenido | Formato |
| --- | --- | --- |
| Logo | imagen desde `Business.LogoPath` si existe; si no, el nombre comercial | centrado, 16 pt, bold |
| Bajada | lema comercial desde configuración | centrado, 8 pt |
| Fecha | `Report.Certificado.Fecha` en semibold + fecha alineada a la derecha | 5 pt de separación arriba |
| Número | `Report.Certificado.Numero` en semibold + número alineado a la derecha | |

**[FIX]** Los tres literales del encabezado estaban en el código: `"PABLO BAEZ"` como contratista y
`"GENERCON"` / `"ENERGIA CONTROLADA"` como logo textual. Pasan a `Business.Contratista`,
`Business.NombreComercial`, `Business.Lema` y `Business.LogoPath`. Un segundo cliente del software
no podía usar el certificado sin recompilar.

**[FIX]** El número de certificado caía al literal `"1"` cuando venía vacío. Ahora es un entero
obligatorio y secuencial (INV-15).

### 4.2 Tabla: 9 columnas

| # | Tipo de ancho | Valor | Contenido |
| --- | --- | --- | --- |
| 1 | relativo | 3 | ítem / descripción |
| 2 | constante | 30 pt | unidad |
| 3 | constante | 40 pt | cantidad |
| 4 | constante | 70 pt | precio unitario |
| 5 | constante | 50 pt | % anterior |
| 6 | constante | 50 pt | % actual |
| 7 | constante | 50 pt | % acumulado |
| 8 | constante | 80 pt | importe actual |
| 9 | constante | 80 pt | importe acumulado |

### 4.3 Encabezado de la tabla: dos filas con celdas combinadas

**Fila 1:**

| Celda | Combinación | Rótulo |
| --- | --- | --- |
| 1 | 2 filas | `Report.Certificado.ItemDescripcion` |
| 2 | 2 columnas (2–3) | `Report.Certificado.Computos`, centrado |
| 3 | 2 filas | `Report.Certificado.PU`, centrado |
| 4 | 3 columnas (5–7) | `Report.Certificado.Avance`, centrado |
| 5 | 2 columnas (8–9) | `Report.Certificado.Importe`, centrado |

**Fila 2**, los subencabezados de las columnas combinadas:

`Report.Certificado.Und`, `Report.Certificado.Cant`, `Report.Certificado.Ant`,
`Report.Certificado.Act`, `Report.Certificado.Acu`, `Report.Certificado.Actual`,
`Report.Certificado.Acumulado` — todos centrados.

**Estilo de la fila 1:** fondo verde oscuro, relleno vertical 2 pt, borde de 0,5 pt negro, centrado,
texto semibold blanco de 8 pt.

**Estilo de la fila 2:** fondo verde claro, relleno vertical 1 pt, borde de 0,5 pt negro, centrado,
texto de 7 pt.

Los verdes salen de tokens (`--eo-report-header` y `--eo-report-subheader`), no de literales.

Este encabezado se repite en cada página. **[FIX]** No se repetía: un certificado de más de una
página dejaba la segunda hoja sin encabezados de columna.

### 4.4 Filas de datos

Estilo de celda: borde de 0,5 pt negro, relleno horizontal 3 pt, vertical 2 pt.

| Col | Campo | Formato | Alineación |
| --- | --- | --- | --- |
| 1 | descripción | texto | izquierda |
| 2 | unidad | texto | centro |
| 3 | cantidad | número, 0 decimales | centro |
| 4 | precio unitario | moneda, 2 decimales | derecha |
| 5 | % anterior | número con 1 decimal + `%` | centro |
| 6 | % actual | número con 1 decimal + `%`, semibold | centro |
| 7 | % acumulado | número con 1 decimal + `%` | centro |
| 8 | subtotal actual | moneda, 2 decimales | derecha |
| 9 | subtotal acumulado | moneda, 2 decimales | derecha |

Fórmulas de los subtotales en doc 06 §5. La cantidad con 0 decimales es lo que hace el sistema
anterior y se conserva: en obra las cantidades son enteras. Si la cantidad tiene parte decimal, se
muestra con los decimales necesarios en lugar de redondear a entero. **[FIX]**: el formato `N0`
mostraba `2` para una cantidad de `2,5`, ocultando media unidad.

### 4.5 Pie de la tabla

En este orden exacto, con el acumulador `total_actual` mutando entre filas:

**Fila «Sub-total»:**

| Celdas | Contenido |
| --- | --- |
| 1–7 combinadas | `Report.Certificado.SubTotal`, derecha, semibold |
| 8 | suma de subtotales actuales, moneda, semibold |
| 9 | suma de subtotales acumulados, moneda, semibold |

**Fila «Ajuste UOCRA»**, sólo si el porcentaje es distinto de cero:

```
ajuste = total_actual × (ajuste_uocra_porcentaje / 100)
```

| Celdas | Contenido |
| --- | --- |
| 1–7 combinadas | `Report.Certificado.AjusteUocra` con `{porcentaje}` sin decimales, derecha, cursiva |
| 8 | importe del ajuste, moneda |
| 9 | vacío |

Después: `total_actual += ajuste`.

**Fila «Otros descuentos»**, sólo si es distinto de cero:

| Celdas | Contenido |
| --- | --- |
| 1–7 combinadas | `Report.Certificado.OtrosDescuentos`, derecha, cursiva |
| 8 | `- {importe}`, moneda |
| 9 | vacío |

Después: `total_actual -= otros_descuentos`.

**Fila «Total a facturar»:**

| Celdas | Contenido |
| --- | --- |
| 1–7 combinadas | `Report.Certificado.TotalAFacturar`, derecha, 11 pt semibold, fondo verde muy claro |
| 8 | `total_actual`, moneda, 11 pt semibold, mismo fondo |
| 9 | vacío, mismo fondo |

El orden de las operaciones es **obligatorio**: el ajuste UOCRA se calcula sobre el subtotal actual
**antes** de restar otros descuentos. Invertirlo da un número distinto.

La columna 9 queda vacía en las tres últimas filas. Es intencional: el acumulado histórico no se
ajusta ni se descuenta, sólo el certificado en curso.

**[NUEVO]** Un ajuste UOCRA **negativo** también se muestra (una quita), con el rótulo en la clave
correspondiente. El sistema anterior sólo lo consideraba si era `> 0`.

### 4.6 Pie de página

Centrado: `Report.Certificado.Footer` con el nombre de la empresa y la numeración de página.

## 5. Reportes nuevos

Los cinco que siguen no existen en el sistema anterior. Comparten el mismo encabezado y pie que §2.1
y el mismo estilo de tabla.

### 5.1 Caja por período

Parámetros: desde, hasta, agrupación (día / semana / mes).

Columnas: período, ingresos, gastos, balance, balance acumulado. Al pie, los totales. En el PDF, un
gráfico de barras de ingresos contra gastos arriba de la tabla.

### 5.2 Rentabilidad por obra

Parámetros: desde, hasta, cliente (opcional).

Columnas: obra, cliente, estado, presupuesto, ingresos, gastos, rentabilidad, margen %. Ordenado por
rentabilidad descendente. Fórmulas en doc 06 §7. Filas con margen negativo en rojo.

### 5.3 Cuenta corriente

Parámetros: cliente, incluir pagadas.

Encabezado con los datos del cliente. Columnas: número, fecha, vencimiento, estado, total, pagado,
saldo, días de mora. Al pie: total facturado, total pagado y saldo. Después, el desglose por buckets
de antigüedad.

### 5.4 Antigüedad de deuda

Parámetros: fecha de corte.

Columnas: cliente, total, `0-30`, `31-60`, `61-90`, `+90`. Al pie, los totales por bucket. Fórmulas
en doc 06 §4.6.

### 5.5 Asistencia mensual

Parámetros: mes, año, empleados.

A4 **horizontal**. Una fila por empleado, una columna por día del mes con el símbolo del tipo de
jornada, y al final las columnas de resumen: completas, medias, faltas, faltas justificadas, feriados
y jornadas equivalentes. Los sábados y domingos con fondo diferenciado. Leyenda de símbolos al pie.

## 6. Estructura del código

```
crates/eo-infrastructure/src/reporting/
├── mod.rs                  # trait ReportGenerator y el registro de reportes
├── format.rs               # format_money, format_date, format_percent, format_number
├── filename.rs             # saneado y patrones de nombre
├── pdf/
│   ├── mod.rs
│   ├── table.rs            # el motor de tablas de §1.1
│   ├── theme.rs            # fuentes, tamaños y colores, desde tokens
│   ├── movimientos.rs
│   ├── liquidacion.rs
│   ├── certificado.rs
│   ├── caja.rs
│   ├── rentabilidad.rs
│   ├── cuenta_corriente.rs
│   ├── antiguedad.rs
│   └── asistencia.rs
├── xlsx/
├── docx/
├── csv/
└── json/
```

El trait:

```rust
pub trait ReportGenerator {
    type Params;
    /// Genera el documento en memoria. La escritura a disco es del llamador,
    /// para que los tests no toquen el sistema de archivos.
    fn generate(&self, params: &Self::Params, ctx: &ReportContext) -> Result<Vec<u8>, AppError>;
}

pub struct ReportContext {
    pub empresa: DatosEmpresa,     // desde configuración
    pub locale: LocaleConfig,
    pub i18n: Arc<dyn Translator>, // resuelve las claves
    pub generado_en: DateTime<Utc>,
}
```

`generate` devuelve bytes: ningún generador escribe archivos ni conoce rutas. Eso hace que todos los
tests de layout corran en memoria.

## 7. Tests obligatorios

| Test | Qué verifica |
| --- | --- |
| `formato_moneda_es_estable` | `format_money` con distintos locales da exactamente la cadena esperada |
| `formato_fecha_es_estable` | ídem para fechas |
| `csv_tiene_bom_y_crlf` | los primeros 3 bytes son el BOM y las líneas terminan en `\r\n` |
| `csv_escapa_comillas_y_comas` | un concepto con `"` y `,` sobrevive el ida y vuelta |
| `csv_fechas_iso` | la fecha sale como `2026-08-14` |
| `json_importes_son_string` | ningún importe es un número JSON |
| `json_es_camel_case` | ninguna clave arranca en mayúscula |
| `xlsx_congela_en_a5_y_tiene_autofiltro` | |
| `xlsx_formatos_numericos_por_columna` | las columnas monetarias declaran `#,##0.00` |
| `pdf_movimientos_no_falla_con_cero_filas` | y muestra el mensaje de vacío |
| `pdf_movimientos_pagina_correctamente` | 500 filas generan más de una página con encabezado repetido |
| `pdf_liquidacion_sin_adelantos` | muestra la fila de «sin adelantos» combinada |
| `pdf_liquidacion_neto_negativo_en_rojo` | |
| `pdf_liquidacion_lista_cada_adelanto_con_fecha` | 5 adelantos producen 5 filas con sus fechas |
| `pdf_certificado_orden_de_operaciones` | ajuste UOCRA antes de otros descuentos; el total coincide con el cálculo a mano |
| `pdf_certificado_ajuste_negativo` | se muestra |
| `pdf_certificado_columna_acumulada_vacia_en_totales` | |
| `pdf_certificado_repite_encabezado` | con 60 ítems, la página 2 tiene los encabezados |
| `todos_los_rotulos_tienen_clave` | ningún generador contiene un literal visible; se verifica por grep en el CI |
| `snapshot_de_layout` | cada reporte contra un PDF/XLSX de referencia versionado, comparando el texto extraído, no los bytes |
