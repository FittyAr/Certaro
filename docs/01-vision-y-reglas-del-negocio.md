# 01 — Visión y reglas del negocio

> Leer antes que cualquier otro documento técnico. Define **para qué** existe el sistema. Todas
> las decisiones de los documentos 03 a 19 se justifican acá.

## 1. Propósito

Certaro es la herramienta de administración diaria de una **empresa pequeña de instalaciones
y mantenimiento eléctrico** en Argentina. Su propósito no es la contabilidad fiscal: es el
**control operativo y el flujo de caja real**.

Consecuencias directas de esa definición:

- No hay libro diario, ni asientos por partida doble, ni cierre de ejercicio, ni presentación
  impositiva. Hay movimientos de dinero, saldos y deudas.
- El IVA existe como un campo de la factura para que el total coincida con el papel, **no** como
  un cálculo automático ni como una obligación a liquidar. Ver
  [`06-casos-de-uso-y-formulas.md`](./06-casos-de-uso-y-formulas.md) §4.
- La precisión monetaria importa porque el usuario compara la app contra sus comprobantes reales.
  De ahí la representación en entero escalado descrita en
  [`04-dinero-fechas-y-tipos.md`](./04-dinero-fechas-y-tipos.md).
- El sistema **reemplaza planillas de Excel dispersas**. Todo dato que el usuario hoy anota en una
  planilla tiene que tener un lugar en la app.

## 2. Usuario

Un solo usuario operativo (el dueño de la empresa), en una máquina de escritorio, sin conexión
garantizada. No hay autenticación, no hay roles, no hay multiusuario en esta versión.

**[NUEVO]** El diseño de datos igualmente conserva `created_at` / `updated_at` / `row_version` en
todas las tablas para poder introducir sincronización o multiusuario más adelante sin migración
destructiva.

## 3. Filosofía de diseño

| Principio | Qué implica en la implementación |
| --- | --- |
| **Minimalista pero profesional** | Interfaz limpia; el usuario muestra la app a sus clientes. Los PDF que emite son documentos de cara al cliente y al empleado. |
| **Flexibilidad real** | El sistema tiene que soportar pagos parciales, adelantos, faltas justificadas y deudas viejas sin bloquear al usuario. Nada de flujos rígidos que impidan registrar la realidad. |
| **Todo personalizable** | Categorías y tipos de movimiento los crea el usuario. Nombre de la empresa, contratista y logo son configuración, no constantes. |
| **Escalabilidad** | Arranca local con SQLite, pero el dominio no asume la base local: los repositorios son puertos. |
| **Memoria visual del usuario** | El usuario transcribe a mano los certificados a propósito («si yo tipeo todo esto me queda como un backup en el cerebro»). No automatizar la carga de certificados con OCR ni importaciones: la carga manual es un requerimiento, no una carencia. |

## 4. Requerimientos explícitos del cliente

Extraídos de la reunión grabada con el usuario (`Docs/transcripcion_audio1.md` del repositorio
legado). Cada requerimiento se enumera con su cita textual, la interpretación funcional y dónde
se implementa.

### RC-01 — PDF de liquidación con detalle de días, tarifa, bruto y adelantos

> «cuando yo hago todos los meses pagarle a la persona "X", tener la cantidad de días, el bruto
> el total, o sea, días, 40.000 pesos por día, son 400 […] pero tengo que restarle el adelanto.
> El adelanto, por ejemplo, acá son de 240. Total al final, 160.»

**Funcional**: la liquidación de un empleado por período debe mostrar días trabajados, valor por
día, bruto, total de adelantos y neto final, y ser exportable a PDF para entregarlo al empleado.

**Prioridad**: el propio usuario lo marca como lo más urgente («Eso es lo que más me urge para
fin de mes»).

**Dónde**: [`06`](./06-casos-de-uso-y-formulas.md) §6, [`12`](./12-reportes-y-exportaciones.md) §3.

### RC-02 — Cada adelanto listado con su fecha en el PDF

> «yo tengo los comprobantes de cada adelanto que le puedo poner fecha. Entonces que le quede
> como: adelanto fecha tal.»
> «acá hay 30 tal día, 40 tal día, 40… 50 el otro, 100 antes. Entonces si yo le pongo la fecha
> ellos van a tener más… y con el comprobante que le mando todos los días van a poder certificar
> que realmente sea cada uno.»

**Funcional**: el PDF de liquidación no muestra el total de adelantos agregado: muestra **una
línea por adelanto**, con fecha, concepto y monto, y luego el total. El objetivo declarado es
«cuentas claras»: que el empleado pueda cruzar cada línea contra el comprobante que recibió.

**Dónde**: [`12`](./12-reportes-y-exportaciones.md) §3, sección «Detalle de adelantos».

### RC-03 — Movimientos con concepto, monto, cantidad, tipo, categoría y fecha

> «ponés el concepto, lo que quieras, el monto, le ponés la cantidad. Si es combustible le vas a
> poner "uno", porque es una carga […] podés poner dos lámparas, ya te va a hacer la suma.»

**Funcional**: el movimiento tiene `Monto` **y** `Cantidad`; el total es el producto. La cantidad
por defecto es 1. Sirve tanto para «una carga de combustible» como para «5 lámparas a $X».

**Dónde**: [`05`](./05-dominio-entidades.md) entidad `Movimiento`, [`06`](./06-casos-de-uso-y-formulas.md) §3.

### RC-04 — Categorías personalizables por el usuario

> «Estas son categorías que se pueden personalizar. Yo puse estas por genérico, pero vos me
> podés decir "No, sacame todo esto, agregame esto otro". Incluso te puedo agregar una opción de
> configuración y que vos las puedas personalizar, las categorías.»

**Funcional**: CRUD completo de categorías desde la aplicación, con jerarquía padre/hijo. Lo
mismo para tipos de movimiento, salvo los cuatro de sistema, que no se pueden eliminar.

**Dónde**: [`03`](./03-modelo-de-datos.md) tablas `Categorias` y `TiposMovimiento`,
[`09`](./09-modulos-funcionales.md) §3.13 y §3.14.

### RC-05 — El movimiento debe identificar a qué empleado se le paga y con qué concepto

> «Lo que no estoy viendo que hayamos hecho es agregar… no tengo los pagos. Pero no me dice a
> quién le estoy haciendo el pago acá. ¿Vés? Acá hay un error […] cuando generás un nuevo
> movimiento tengo que poder seleccionar a quién le estás […] a quién le estás haciendo el pago y
> qué tipo de concepto es el pago que le hacés a esa persona.»

**Funcional**: el formulario de movimiento debe permitir seleccionar `Empleado` y
`TipoConceptoPago` (adelanto, quincena, liquidación, viático…). Sin esto, un adelanto no se puede
imputar a nadie y RC-01/RC-02 son imposibles.

**Dónde**: [`05`](./05-dominio-entidades.md) entidad `Movimiento` (FKs `EmpleadoId`,
`TipoConceptoPagoId`), [`09`](./09-modulos-funcionales.md) §3.2.

### RC-06 — Resumen por empleado

> «A cada empleado, por ejemplo, agarrar y poder tener el resumen de cada empleado.»

**Funcional**: vista de detalle de empleado con su histórico de liquidaciones, adelantos y
asistencia.

**Dónde**: [`09`](./09-modulos-funcionales.md) §3.9.

### RC-07 — Jerarquía Cliente → Obra → Trabajo → Orden de trabajo → Ítems

> «primero cargás un cliente, lo registrás como cliente […] después cuando vas a trabajo, podés
> agregar un trabajo porque un cliente tiene varios trabajos.»
> «lo que me gustaría entonces es obra. La obra por ejemplo dice "Obra 1892" que es Mercedes,
> Tecnocas de Mercedes. Esa es el lugar físico.»
> «podemos mejorar y crear el concepto de orden de trabajo y dentro de la orden de trabajo que
> haya varios subtrabajos.»

**Funcional**: cuatro niveles. El **cliente** es la empresa; la **obra** es el lugar físico con su
número; el **trabajo** es la tarea contratada; la **orden de trabajo** es el documento con ítems.
La obra tiene su propio número, CUIT y datos.

**Dónde**: [`03`](./03-modelo-de-datos.md) §2, [`05`](./05-dominio-entidades.md).

### RC-08 — Listar los trabajos de un cliente con su estado

> «La idea es que vos puedas listar todos los trabajos que hizo un solo cliente, si están
> finalizados o no, si están pausados, si están en proceso.»

**Funcional**: filtro por cliente y por estado en el listado de trabajos. Estados: pendiente, en
proceso, pausado, finalizado, cancelado.

**Dónde**: [`08`](./08-maquinas-de-estado.md) §3, [`09`](./09-modulos-funcionales.md) §3.5.

### RC-09 — Certificados con porcentaje de avance por ítem y total certificado

> «ellos en la orden de compra me dicen el monto y el porcentaje que tiene cada cosa de
> terminación.»
> «Este cableado, que son 4.200 metros, la cantidad de metros, acá me dice que hice el 60%. Acá
> me da el total de lo que se certificó.»

**Funcional**: cada ítem de la orden tiene cantidad, precio unitario y porcentaje ejecutado; el
sistema calcula el subtotal certificado del ítem y el total del certificado.

**Dónde**: [`06`](./06-casos-de-uso-y-formulas.md) §5.

### RC-10 — Los certificados son acumulativos y numerados por orden de trabajo

> «según el certificado. Por ejemplo ahora me ponen certificado uno porque este es el primer
> certificado. Ahora la segunda va a decir certificado dos y me aumenta solamente el porcentaje
> del trabajo. Una vez que se termina, ya me mandan otro pero con otra orden de trabajo.»

**Funcional**: el número de certificado es secuencial **dentro de la orden de trabajo**. El
porcentaje que se carga en el certificado *N* es el avance **de ese período**; el acumulado es la
suma de los anteriores más el actual. El acumulado no puede pasar de 100 %.

**Dónde**: [`06`](./06-casos-de-uso-y-formulas.md) §5, [`07`](./07-validaciones.md) §V-13.

### RC-11 — Saber qué quedó pendiente y por qué, con una leyenda por ítem

> «hay un cable que por ejemplo no lo voy a tener, no me lo van a traer. O sea que eso yo lo tengo
> que descontar y me va a quedar pendiente. Entonces […] al día de mañana decís "¿por qué me quedó
> pendiente?" Vés y decís "ah sí, este es el cable…". […] si en todo el año se sigue trabajando se
> te va a olvidar.»
> «Y le puedo poner como una leyenda…»

**Funcional**: cada ítem de orden de trabajo tiene un campo de **observaciones/leyenda** libre, y
el sistema muestra el porcentaje pendiente (100 − acumulado) para explicar por qué la orden no
está cerrada. El criterio de avance es **si el trabajo se hizo**, no si el material se recibió
(el usuario lo aclara explícitamente: «No por lo que recibí, sino si el trabajo se hizo o no»).

**Dónde**: [`05`](./05-dominio-entidades.md) `OrdenTrabajoItem.Observaciones`,
[`09`](./09-modulos-funcionales.md) §3.6.

### RC-12 — Una orden de trabajo por trabajo, con múltiples certificados

> «cada orden de trabajo tiene una sola orden de trabajo mínima. Esa orden de trabajo tiene todos
> estos trabajos. Si yo agarro a cada orden de trabajo la abro y le puedo cargar el porcentaje de
> cada uno de estos ítems, entonces yo siempre voy a tener el porcentaje final de lo que tengo de
> trabajo o de lo que me quedó pendiente.»

**Funcional**: la relación es 1 trabajo → N órdenes de trabajo (sin límite: «Si son diez, diez;
si es uno, uno»), y 1 orden → N ítems, y 1 orden → N certificados.

**Dónde**: [`03`](./03-modelo-de-datos.md) §2.

### RC-13 — Múltiples emails y contactos por cliente

> «Acá por ejemplo tengo ochocientos mails de cada persona. Estaría bueno que yo lo pueda… que
> estos mails yo pueda tener más de uno.»
> «tengo el de personal, el de López, el de Nahuel… todos los mails. Entonces cualquier cosa yo
> digo "ya no tengo que estar buscando" sino que directamente de ahí lo saco y hasta… que haga
> clic y te abra el correo y ya le mandás el mail.»

**Funcional**: tabla `ClienteContactos` con N contactos por cliente (nombre, cargo, email,
teléfono, marca de principal). En la interfaz, el email es accionable: al hacer clic abre el
cliente de correo con `mailto:`.

**Dónde**: [`03`](./03-modelo-de-datos.md) tabla `ClienteContactos`,
[`09`](./09-modulos-funcionales.md) §3.3.

### RC-14 — Adjuntos y documentos

> «Le invento un mail si quiero, le puedo poner la parte de documentos.»

**Funcional**: adjuntar comprobantes, fotos y PDFs a las entidades principales.

**Dónde**: [`13`](./13-servicios-externos-y-archivos.md) §3.

### RC-15 — Dashboard con gráficos, ampliable

> «acá la idea es seguir agregando más gráficos. Me podés pedir algún gráfico en particular o yo
> irte generando los que a mí se me vayan ocurriendo.»

**Funcional**: dashboard con KPIs y gráficos, diseñado como colección extensible de widgets, no
como una pantalla monolítica.

**Dónde**: [`06`](./06-casos-de-uso-y-formulas.md) §9, [`09`](./09-modulos-funcionales.md) §3.1.

### RC-16 — Exportación en varios formatos

> «vos necesitás que te genere un archivo PDF, o Word, o Excel, o lo que sea»

**Funcional**: PDF, XLSX, DOCX, CSV y JSON.

**Dónde**: [`12`](./12-reportes-y-exportaciones.md).

### RC-17 — Rentabilidad por orden de trabajo

> «con esto yo sé más o menos el dinero que me está quedando por orden de trabajo»

**Funcional**: además de la rentabilidad por obra y por trabajo, poder ver el dinero certificado
contra los gastos imputados a la orden.

**Dónde**: [`06`](./06-casos-de-uso-y-formulas.md) §7.

## 5. Módulos del sistema

| Módulo | Qué resuelve |
| --- | --- |
| **Movimientos** | Registro de toda entrada y salida de dinero. Es el corazón del flujo de caja. |
| **Categorías** y **Tipos de movimiento** | Clasificación personalizable de los movimientos. |
| **Clientes** | Datos fiscales y contactos múltiples. |
| **Obras** | Lugares físicos de trabajo, con número único. |
| **Trabajos** | Tareas contratadas dentro de una obra, con estado y presupuesto. |
| **Órdenes de trabajo y certificados** | Ítems con cantidad, precio y avance porcentual acumulativo. |
| **Facturas y pagos** | Lo facturado contra lo cobrado; deuda por cliente y antigüedad. |
| **Empleados** | Perfiles, tarifa base y frecuencia de pago. |
| **Asistencia** | Estado diario por empleado, con impacto en la liquidación. |
| **Liquidaciones** | Cálculo de bruto − adelantos = neto, y su PDF de detalle. |
| **Reportes** | Centro de exportación con filtros y formatos. |
| **Dashboard** | KPIs, alertas y gráficos. |
| **Configuración** | Datos de la empresa, rutas, moneda, umbrales, idioma, tema. |

## 6. Invariantes de negocio

Reglas que el sistema **no debe permitir violar nunca**. Cada una tiene su implementación en la
validación (doc 07), en el esquema (doc 03) o en el caso de uso (doc 06).

| ID | Invariante | Dónde se hace cumplir |
| --- | --- | --- |
| INV-01 | El total de un movimiento es siempre `monto × cantidad`; nunca se persiste el total, se calcula. | Doc 06 §3 |
| INV-02 | `Cantidad` de un movimiento es > 0. | Doc 07 |
| INV-03 | Un movimiento pertenece obligatoriamente a un `TipoMovimiento` y a una `Categoria`. | Doc 03 (FK `RESTRICT`) |
| INV-04 | Los cuatro tipos de movimiento de sistema no se pueden eliminar ni cambiar su marca `EsIngreso`. | Doc 06 §11 |
| INV-05 | Un adelanto (tipo de sistema `…0003`) descontado en una liquidación no puede descontarse dos veces. | Doc 06 §6 |
| INV-06 | El número de obra es único a nivel global. | Doc 03 (índice `UNIQUE`) |
| INV-07 | Existe a lo sumo un registro de asistencia por empleado y por fecha. | Doc 03 (índice `UNIQUE` compuesto) |
| INV-08 | El porcentaje acumulado de un ítem de orden de trabajo nunca supera 100. | Doc 07 §V-13 **[HUECO]** |
| INV-09 | La suma de pagos imputados a una factura no puede superar el total de la factura. | Doc 07 **[HUECO]** |
| INV-10 | El estado de una factura se deriva de sus pagos; no se edita a mano de forma inconsistente. | Doc 08 §2 **[BUG-LEGADO]** |
| INV-11 | Un cliente, obra, tipo de movimiento o categoría con dependencias no se puede borrar. | Doc 03 (FK `RESTRICT`) |
| INV-12 | Nada se borra físicamente: todo borrado es lógico (`is_deleted`). | Doc 04 §4 |
| INV-13 | Toda fecha/hora se almacena en UTC. | Doc 04 §3 |
| INV-14 | Todo importe se almacena como entero escalado; nunca como punto flotante. | Doc 04 §1 |
| INV-15 | El número de certificado es secuencial y único dentro de su orden de trabajo. | Doc 06 §5 |
| INV-16 | La fecha de vencimiento de una factura no puede ser anterior a su fecha de emisión. | Doc 07 |
| INV-17 | La fecha de fin de una liquidación no puede ser anterior a su fecha de inicio. | Doc 07 |

## 7. Fuera de alcance

Explícitamente **no** forma parte de este sistema:

- Contabilidad fiscal, libros contables, presentaciones ante AFIP/ARCA.
- Facturación electrónica (emisión con CAE). Las facturas se registran, no se emiten fiscalmente.
- Autenticación, usuarios, roles y permisos.
- Sincronización en la nube o modo multiusuario concurrente.
- OCR o carga automática de certificados (ver §3, es un requerimiento del usuario cargarlos a mano).
- Liquidación de cargas sociales y aportes sindicales calculados. El ajuste UOCRA del certificado
  es un monto que el usuario ingresa, no un cálculo del sistema.
