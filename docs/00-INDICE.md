# 00 — Índice y guía de lectura

Esta carpeta es **la especificación normativa** de ElectroObra. Está escrita para que quien
implemente el sistema en Rust + Tauri + Vue 3 no necesite leer el código C# anterior ni inferir
reglas de negocio. Si un documento y el código difieren, **manda el documento**. Si el documento
no dice algo que necesitás, **preguntá**: no inventes fórmulas financieras.

## Convenciones de esta documentación

| Marca | Significado |
| --- | --- |
| **[LEGADO]** | Cómo lo hacía la app C# anterior. Contexto histórico, no necesariamente a replicar. |
| **[NUEVO]** | Decisión tomada para la reescritura. Es lo que hay que implementar. |
| **[BUG-LEGADO]** | Defecto conocido del sistema anterior que el nuevo debe corregir. |
| **[HUECO]** | Regla que el sistema anterior no tenía y que ahora hay que definir/implementar. |
| `código` | Identificador literal: nombre de tabla, columna, clave i18n, comando o clave de configuración. |

El idioma del **dominio es español** (obra, trabajo, certificado, liquidación, adelanto). Los
nombres de tablas, columnas, entidades y claves i18n van en español `snake_case` /
`PascalCase` según corresponda. El idioma del **código de infraestructura y los commits es
inglés**.

## Mapa de lectura por rol

### Si vas a crear la base de datos
1. [`03-modelo-de-datos.md`](./03-modelo-de-datos.md) — DDL literal, índices, claves foráneas, semilla.
2. [`04-dinero-fechas-y-tipos.md`](./04-dinero-fechas-y-tipos.md) — cómo se representan importes, porcentajes y fechas.

### Si vas a implementar el dominio y los casos de uso
1. [`05-dominio-entidades.md`](./05-dominio-entidades.md) — entidades y enums, propiedad por propiedad.
2. [`06-casos-de-uso-y-formulas.md`](./06-casos-de-uso-y-formulas.md) — todas las fórmulas transcritas.
3. [`07-validaciones.md`](./07-validaciones.md) — reglas de validación con su clave i18n.
4. [`08-maquinas-de-estado.md`](./08-maquinas-de-estado.md) — transiciones válidas de estado.

### Si vas a implementar los comandos Tauri
1. [`11-contratos-tauri.md`](./11-contratos-tauri.md) — firma de cada comando y tipos TypeScript espejo.
2. [`02-arquitectura.md`](./02-arquitectura.md) — manejo de errores y mapeo `AppError` → payload.

### Si vas a implementar la interfaz
1. [`09-modulos-funcionales.md`](./09-modulos-funcionales.md) — comportamiento de cada pantalla.
2. [`10-navegacion-y-atajos.md`](./10-navegacion-y-atajos.md) — rutas, menú y atajos de teclado.
3. [`16-frontend.md`](./16-frontend.md) — reparto PrimeVue/Shadcn, stores, composables, tokens.
4. [`14-configuracion-e-i18n.md`](./14-configuracion-e-i18n.md) — catálogo de claves i18n.

### Si vas a implementar reportes, servicios externos o migración de datos
1. [`12-reportes-y-exportaciones.md`](./12-reportes-y-exportaciones.md) — layout exacto de cada export.
2. [`13-servicios-externos-y-archivos.md`](./13-servicios-externos-y-archivos.md) — APIs, adjuntos, backup.
3. [`15-migracion-de-datos.md`](./15-migracion-de-datos.md) — importador one-shot desde la base vieja.

### Si vas a montar CI o testear
1. [`17-testing.md`](./17-testing.md) — estrategia por capa.
2. [`18-devops.md`](./18-devops.md) — pipelines, versionado, instaladores.

### Antes de empezar cualquier fase
[`19-roadmap.md`](./19-roadmap.md) — orden de implementación y criterio de terminado por fase.

## Índice completo

| # | Documento | Contenido |
| --- | --- | --- |
| 00 | `00-INDICE.md` | Este archivo: guía de lectura y glosario. |
| 01 | [`01-vision-y-reglas-del-negocio.md`](./01-vision-y-reglas-del-negocio.md) | Para qué existe el sistema, requerimientos del cliente, invariantes de negocio. |
| 02 | [`02-arquitectura.md`](./02-arquitectura.md) | Capas, crates, dependencias, flujo end-to-end, errores, logging. |
| 03 | [`03-modelo-de-datos.md`](./03-modelo-de-datos.md) | DDL SQL literal de las 21 tablas, índices, FK, semilla. |
| 04 | [`04-dinero-fechas-y-tipos.md`](./04-dinero-fechas-y-tipos.md) | `Money(i64)` escala 10 000, UTC, soft delete, `row_version`. |
| 05 | [`05-dominio-entidades.md`](./05-dominio-entidades.md) | 20 entidades, 7 enums, propiedades calculadas. |
| 06 | [`06-casos-de-uso-y-formulas.md`](./06-casos-de-uso-y-formulas.md) | Cada caso de uso y cada fórmula transcrita. |
| 07 | [`07-validaciones.md`](./07-validaciones.md) | Reglas de validación, claves i18n, huecos a corregir. |
| 08 | [`08-maquinas-de-estado.md`](./08-maquinas-de-estado.md) | Estados y transiciones de factura, obra y trabajo. |
| 09 | [`09-modulos-funcionales.md`](./09-modulos-funcionales.md) | Comportamiento por pantalla: filtros, paginación, acciones. |
| 10 | [`10-navegacion-y-atajos.md`](./10-navegacion-y-atajos.md) | Rutas, agrupación del menú, atajos de teclado. |
| 11 | [`11-contratos-tauri.md`](./11-contratos-tauri.md) | Comandos IPC y tipos TypeScript. |
| 12 | [`12-reportes-y-exportaciones.md`](./12-reportes-y-exportaciones.md) | Layouts PDF/XLSX/DOCX/CSV/JSON. |
| 13 | [`13-servicios-externos-y-archivos.md`](./13-servicios-externos-y-archivos.md) | Dólar, feriados, adjuntos, backup, export/import JSON. |
| 14 | [`14-configuracion-e-i18n.md`](./14-configuracion-e-i18n.md) | Claves de configuración y catálogo i18n canónico. |
| 15 | [`15-migracion-de-datos.md`](./15-migracion-de-datos.md) | Importador one-shot y verificación post-import. |
| 16 | [`16-frontend.md`](./16-frontend.md) | Vue 3, PrimeVue, Shadcn-Vue, Tailwind, Pinia. |
| 17 | [`17-testing.md`](./17-testing.md) | Estrategia y cobertura mínima por capa. |
| 18 | [`18-devops.md`](./18-devops.md) | CI, versionado, releases, instalador. |
| 19 | [`19-roadmap.md`](./19-roadmap.md) | Fases ordenadas por dependencia con criterio de terminado. |

## Glosario del dominio

Estos términos son los que usa el cliente. **No traducirlos** en el código ni en la base de datos.

| Término | Definición operativa |
| --- | --- |
| **Movimiento** | Cualquier entrada o salida de dinero registrada en el sistema. Es la unidad atómica del flujo de caja. Tiene un `TipoMovimiento`, una `Categoria`, un monto, una cantidad y una fecha. |
| **Tipo de movimiento** | Clasificación primaria del movimiento: Ingreso, Gasto, Adelanto, Ajuste. Es una **tabla** (no un enum) porque el usuario puede crear los suyos; cuatro filas son de sistema y no se pueden borrar. |
| **Categoría** | Clasificación secundaria y personalizable del movimiento (combustible, viáticos, alquiler, herramientas…). Jerárquica: una categoría puede tener categoría padre. |
| **Cliente** | Empresa o persona que contrata trabajos. Tiene datos fiscales (CUIT) y **múltiples contactos/emails**. |
| **Obra** | El **lugar físico** donde se trabaja, identificado por un número de obra único (p. ej. «Obra 1892 — Tecnocas de Mercedes»). Pertenece a un cliente. Agrupa órdenes de trabajo y movimientos. |
| **Trabajo** | Una tarea contratada dentro de una obra (p. ej. «canalización del primer piso»). Tiene estado, presupuesto y fechas. Un cliente tiene varios trabajos. |
| **Orden de trabajo** | El documento que el cliente envía y que enumera ítems a ejecutar con cantidad y precio unitario. Cada orden pertenece a un trabajo/obra y contiene una lista de **ítems**. |
| **Ítem de orden de trabajo** | Una línea de la orden: descripción, unidad, cantidad, precio unitario y **porcentaje de avance** certificado. |
| **Certificado** | La emisión periódica que declara el porcentaje de avance alcanzado por cada ítem de una orden de trabajo. «Certificado 1», «Certificado 2»… El porcentaje es **acumulativo** entre certificados sucesivos de la misma orden. |
| **Factura** | Comprobante emitido a un cliente, con subtotal, IVA, total, estado y vencimiento. Recibe uno o varios **pagos**. |
| **Pago de factura** | Cobro parcial o total imputado a una factura, con fecha y medio de pago. |
| **Empleado** | Persona que trabaja para la empresa. Tiene tarifa base y frecuencia de pago (diaria, semanal, quincenal, mensual). |
| **Asistencia** | Registro diario del estado de un empleado: jornada completa, media jornada, falta, falta justificada, feriado. Un registro por empleado y día. |
| **Liquidación** | El cálculo del pago de un empleado por un período: días trabajados × tarifa = bruto, menos adelantos = neto. Se entrega al empleado como PDF con el detalle. |
| **Adelanto** | Dinero entregado al empleado **antes** de la liquidación. Se registra como movimiento del tipo de sistema «Adelanto» y se descuenta del bruto al liquidar. |
| **UOCRA** | Sindicato de la construcción argentino. En el certificado aparece como un ajuste/descuento sobre el total certificado. |
| **Rentabilidad** | Por obra o por trabajo: ingresos imputados menos gastos imputados, con su margen porcentual. |
| **Antigüedad de deuda** | Clasificación del saldo pendiente de cada cliente en tramos de días: 0-30, 31-60, 61-90 y más de 90. |
| **Caja** | El saldo acumulado resultante de todos los movimientos. El sistema controla «caja real», no contabilidad fiscal. |
