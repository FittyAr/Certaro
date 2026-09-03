export interface HelpTopic {
  id: string
  title: string
  subtitle: string
  purpose: string
  workflow: string[]
  strengths: string[]
  limitations: string[]
  tips: string[]
}

export const HELP_REGISTRY: Record<string, HelpTopic> = {
  'kanban-overview': {
    id: 'kanban-overview',
    title: 'Tableros Kanban',
    subtitle: 'Gestión visual de flujos de trabajo en obras y operaciones',
    purpose:
      'El módulo Kanban permite visualizar, organizar y hacer seguimiento continuo del avance de tareas, obras y órdenes de trabajo mediante columnas de estado interactivas y tarjetas dinámicas.',
    workflow: [
      'Selecciona el tablero deseado en la barra superior (Presets del sistema o personalizados).',
      'Filtra por texto, nivel de prioridad o por un Proyecto específico si necesitas enfocarte en una sola obra.',
      'Arrastra las tarjetas entre columnas para actualizar su estado de avance.',
      'Haz clic derecho sobre cualquier tarjeta o columna para acceder a opciones avanzadas como checklist, etiquetas o mover de columna.',
      'Crea nuevas tarjetas o columnas según las necesidades operativas de tu equipo.',
    ],
    strengths: [
      'Arrastre fluido con Pointer Drag & Drop inmune a bloqueos del sistema operativo.',
      'Sincronización bidireccional automática con Trabajos y Órdenes de Trabajo reales.',
      'Menús contextuales completos (clic derecho) para todas las acciones rápidas.',
      'Colores distintivos por columna y por tablero para identificar prioridades al instante.',
      'Filtro instantáneo por Proyecto para aislar tareas de clientes y obras complejas.',
    ],
    limitations: [
      'Los tableros presets del sistema ("Trabajos", "Órdenes de Trabajo") no se pueden eliminar para preservar la integridad contable y operativa, pero sí se pueden ocultar.',
      'Las columnas mapeadas a estados de dominio no se pueden borrar mientras mantengan dicha sincronización obligatoria.',
      'Para eliminar un tablero que contiene tarjetas, el sistema te exigirá escribir su nombre exacto como medida de seguridad contra pérdidas accidentales.',
    ],
    tips: [
      'Puedes usar las checklists dentro de cada tarjeta para desglosar entregas técnicas o materiales.',
      'Si prefieres no arrastrar con el mouse, puedes activar los botones manuales ◀ y ▶ desde Configuración > General.',
      'En la vista de Proyectos, puedes hacer clic derecho en cualquier fila y seleccionar "Ver en Kanban" para abrir directamente las tarjetas de esa obra.',
    ],
  },

  'kanban-sync': {
    id: 'kanban-sync',
    title: 'Sincronización de Entidades y Presets',
    subtitle: 'Cómo se conectan los tableros con los Trabajos y Órdenes del sistema',
    purpose:
      'Los tableros de tipo "Preset" (como Trabajos y Órdenes de Trabajo) están vinculados directamente a la base de datos operativa y financiera de Certaro.',
    workflow: [
      'Al crear un Trabajo en cualquier parte del sistema (en Proyectos o Trabajos), se crea y sincroniza automáticamente su tarjeta en la columna inicial.',
      'Al mover la tarjeta de columna en el Kanban, el sistema actualiza automáticamente el estado del Trabajo en el resto de la aplicación (ej: de Pendiente a En Progreso o Finalizado).',
      'Si en algún momento realizas importaciones masivas o sincronizaciones externas, puedes pulsar el botón "↻ Sincronizar" para forzar una reconciliación completa.',
    ],
    strengths: [
      'Auto-sincronización en segundo plano: cada vez que abres el tablero, el sistema concilia cualquier trabajo nuevo automáticamente.',
      'Consistencia de datos: el estado contable y el estado visual nunca quedan desfasados.',
      'Cálculo de presupuestos y fechas límite integrados directamente en la tarjeta.',
    ],
    limitations: [
      'Mover una tarjeta en un tablero preset solo puede actualizar estados válidos según la máquina de estados del dominio.',
      'Las tarjetas que representan Trabajos del sistema no pueden ser eliminadas directamente desde el Kanban si el Trabajo tiene certificados o movimientos asociados.',
    ],
    tips: [
      'El botón "↻ Sincronizar" solo es necesario como respaldo en caso de que múltiples usuarios editen en red simultáneamente.',
    ],
  },

  'kanban-columns': {
    id: 'kanban-columns',
    title: 'Columnas y Límites WIP',
    subtitle: 'Organización de etapas y control del trabajo en proceso',
    purpose:
      'Las columnas representan las distintas etapas de tu proceso productivo. Permiten definir límites WIP (Work In Progress) y colores temáticos para controlar cuellos de botella.',
    workflow: [
      'Para reordenar columnas, haz clic sostenido en la cabecera (en el ícono ⋮⋮) y arrastra la columna horizontalmente a la posición deseada.',
      'Para cambiar nombre, color o límite WIP, haz clic en el ícono de lápiz ✎ o haz clic derecho sobre la cabecera.',
      'Si configuras un límite WIP, el contador te mostrará la cantidad actual vs el límite sugerido (ej: 3 / 5).',
    ],
    strengths: [
      'Reordenamiento atómico en una sola transacción: sin bloqueos de base de datos.',
      'Límites WIP visuales para detectar saturación de personal o demoras de materiales.',
      'Colores personalizables con paleta predeterminada para rápida distinción.',
    ],
    limitations: [
      'En tableros presets, las columnas principales corresponden a los estados del sistema (Pendiente, En Progreso, Pausado, Finalizado, etc.).',
      'Al eliminar una columna que contiene tarjetas, se solicitará confirmación ya que las tarjetas huérfanas serán removidas.',
    ],
    tips: [
      'Usa colores cálidos para etapas de alta atención o revisión y colores fríos o neutros para etapas iniciales.',
    ],
  },

  'kanban-boards-management': {
    id: 'kanban-boards-management',
    title: 'Gestión y Ciclo de Vida de Tableros',
    subtitle: 'Diferencias entre tableros presets y personalizados, ocultamiento y eliminación',
    purpose:
      'Permite administrar la visibilidad de los tableros en la barra superior y eliminar tableros que ya no estén en uso.',
    workflow: [
      'Haz clic en el botón "⚙ Tableros" en la barra superior para abrir el panel de gestión.',
      'Usa el botón "Ocultar / Mostrar" para ocultar tableros de la barra sin perder su información.',
      'Para tableros personalizados que ya no necesites, haz clic en el botón "✕" para eliminarlos.',
      'Si el tablero tiene tarjetas dentro, se abrirá un modal de seguridad pidiéndote confirmar escribiendo el nombre exacto del tablero.',
    ],
    strengths: [
      'Interfaz limpia: puedes ocultar tableros secundarios y mantener a la vista solo los activos.',
      'Protección contra borrado accidental con confirmación tipográfica obligatoria.',
      'Creación ilimitada de tableros temáticos (por equipo, por mes o por tipo de tarea).',
    ],
    limitations: [
      'Los tableros presets no pueden eliminarse físicamente, únicamente ocultarse.',
      'La eliminación de un tablero personalizado con tarjetas es permanente e irreversible.',
    ],
    tips: [
      'Si terminaste una obra o proyecto especial, en lugar de borrar el tablero puedes simplemente ocultarlo para conservar el historial.',
    ],
  },

  'kanban-buttons-config': {
    id: 'kanban-buttons-config',
    title: 'Botones Manuales de Reordenamiento (◀ / ▶)',
    subtitle: 'Preferencia de accesibilidad para mover columnas',
    purpose:
      'Permite elegir si deseas mostrar botones en las cabeceras de columnas para moverlas a la izquierda o derecha con un clic.',
    workflow: [
      'Por defecto, esta opción está desactivada para ofrecer una interfaz moderna y limpia basada en arrastre (Pointer Drag & Drop).',
      'Si utilizas pantalla táctil, touchpad sensible o prefieres hacer clic en lugar de arrastrar, puedes activar esta opción en Configuración > General > Tablero Kanban.',
    ],
    strengths: [
      'Accesibilidad mejorada para usuarios que prefieren navegación por clic.',
      'Preferencia guardada localmente de manera permanente.',
    ],
    limitations: [
      'Mover columnas con botones ejecuta la misma transacción atómica que el arrastre, pero requiere más clics repetidos.',
    ],
    tips: [
      'Incluso con los botones desactivados, siempre puedes mover columnas a la izquierda o derecha haciendo clic derecho sobre la cabecera de la columna.',
    ],
  },

  'dashboard-overview': {
    id: 'dashboard-overview',
    title: 'Panel de Control y Resumen General',
    subtitle: 'Visión panorámica financiera y operativa de la empresa',
    purpose:
      'El Panel de Control consolida en una sola pantalla los indicadores clave (KPIs) de ingresos, egresos, saldo operativo, facturación pendiente, cotizaciones cambiarias y alertas prioritarias.',
    workflow: [
      'Selecciona el período de análisis deseado (Mes actual, Mes anterior, Trimestre, Año o Histórico completo) con los selectores de la barra superior.',
      'Observa la variación porcentual con respecto al período anterior para detectar tendencias de crecimiento o desvíos.',
      'Revisa las alertas automáticas en la parte superior y haz clic sobre cualquiera de ellas para ir directamente a los registros afectados.',
      'Utiliza el modo privacidad (ícono de ojo) cuando estés compartiendo pantalla o trabajando frente a terceros.',
    ],
    strengths: [
      'Cálculo 100% nativo y transaccional desde el backend sin desfase contable.',
      'Modo privacidad instantáneo para ocultar cifras confidenciales con un solo clic.',
      'Alertas inteligentes interactivas con filtros preaplicados al navegar.',
      'Cotizaciones de divisas en tiempo real integradas sin bloquear el rendimiento.',
    ],
    limitations: [
      'Los montos corresponden a transacciones registradas o devengadas en el sistema.',
      'Si el servicio externo de cotizaciones no responde, el panel sigue funcionando con normalidad omitiendo dicho bloque accesorio.',
    ],
    tips: [
      'Haz clic en "Actualizar" si acabas de registrar cobros o movimientos en otra terminal.',
      'Puedes presionar Ctrl+Shift+P para alternar rápidamente el modo privacidad.',
    ],
  },

  'dashboard-alertas': {
    id: 'dashboard-alertas',
    title: 'Alertas Automáticas de Gestión',
    subtitle: 'Notificaciones tempranas de cobranzas, vencimientos y saldos',
    purpose:
      'Detecta automáticamente situaciones críticas que requieren atención inmediata: facturas impagas vencidas, clientes con saldo deudor excedido o certificados pendientes.',
    workflow: [
      'Al iniciar el sistema o cambiar de período, las alertas se evalúan y muestran agrupadas por severidad.',
      'Haz clic en la tarjeta de cualquier alerta para acceder directamente al módulo correspondiente con los filtros adecuados preseleccionados.',
      'Gestiona o regulariza el cobro/pago para que la alerta se resuelva automáticamente en el próximo cálculo.',
    ],
    strengths: [
      'Navegación contextual con un clic: elimina la necesidad de buscar manualmente qué facturas vencieron.',
      'Clasificación por severidad visual (amarillo para advertencias, rojo para urgencias contables).',
    ],
    limitations: [
      'Las alertas son informativas y de control preventivo; no bloquean la carga habitual del resto del sistema.',
    ],
    tips: [
      'Comienza tu jornada revisando las alertas rojas para ejecutar acciones de cobranza proactivas.',
    ],
  },

  'dashboard-cotizaciones': {
    id: 'dashboard-cotizaciones',
    title: 'Cotizaciones de Divisas y Moneda Extranjera',
    subtitle: 'Seguimiento cambiario en tiempo real para presupuestos y acopios',
    purpose:
      'Muestra la cotización del dólar (oficial, blue, tarjeta, MEP) para facilitar el cálculo de presupuestos, compras de insumos en divisa extranjera y fijación de precios.',
    workflow: [
      'El sistema consulta en segundo plano la API de cotizaciones públicas configurada.',
      'Al presupuestar o registrar movimientos en moneda extranjera, puedes consultar la cotización del día directamente en este panel.',
    ],
    strengths: [
      'Caché local tolerante a fallos: si la conexión a internet es inestable, muestra la última cotización conocida informando la fecha de actualización.',
      'Arquitectura no bloqueante: la pantalla carga de inmediato sin esperar la respuesta del servidor externo.',
    ],
    limitations: [
      'Depende de la disponibilidad del proveedor externo de cotizaciones.',
    ],
    tips: [
      'En Configuración > Integraciones puedes personalizar qué tipos de dólar deseas consultar.',
    ],
  },

  'movimientos-overview': {
    id: 'movimientos-overview',
    title: 'Libro de Caja y Movimientos Diarios',
    subtitle: 'Registro de ingresos, egresos y adelantos con imputación analítica',
    purpose:
      'Permite asentar cada movimiento de dinero (efectivo, transferencias o divisas), imputándolo a su tipo, categoría jerárquica y, opcionalmente, a una obra o proyecto para control de costos.',
    workflow: [
      'Haz clic en "+ Nuevo" o presiona Ctrl+N para desplegar el formulario de carga rápida.',
      'Elige la fecha, el concepto descriptivo y el Tipo de Movimiento (Ingreso o Egreso).',
      'Asigna la Categoría correspondiente para alimentar los reportes y desgloses de gastos.',
      'Si el gasto o ingreso pertenece a una obra específica, selecciónala en la imputación de Proyecto/Trabajo.',
      'Si el movimiento es en Dólares (USD), ingresa el importe y la cotización aplicada.',
      'Para filtrar el historial, utiliza los selectores de fechas, categorías y buscador de texto.',
      'Exporta la vista filtrada en cualquier momento usando el menú "Exportar" (CSV o DOCX).',
    ],
    strengths: [
      'Cálculo de totales del servidor: la barra inferior muestra el balance exacto de todo el filtro seleccionado, sin importar la paginación.',
      'Imputación directa a obras para conocer la rentabilidad neta real de cada trabajo.',
      'Soporte bimonetario nativo (Pesos / Dólares con cotización congelada al asentar).',
      'Paginación de alto rendimiento optimizada para bases con decenas de miles de registros.',
    ],
    limitations: [
      'Los movimientos que correspondan a adelantos de sueldo ya liquidados quedan protegidos para preservar la consistencia laboral.',
      'La eliminación de un movimiento es permanente y descuenta su valor inmediatamente del saldo de caja.',
    ],
    tips: [
      'Para revisar exclusivamente la caja de una obra, ve a Proyectos, haz clic derecho sobre la fila y elige "Ver Caja".',
      'Puedes usar las teclas de flecha para recorrer las páginas de la grilla velozmente.',
    ],
  },

  'clientes-overview': {
    id: 'clientes-overview',
    title: 'Gestión de Clientes y Directorio Comercial',
    subtitle: 'Administración integral de clientes, contactos y saldo consolidado',
    purpose:
      'Permite gestionar la información fiscal, comercial y de contacto de tus clientes, así como auditar en tiempo real su saldo pendiente y proyectos contratados.',
    workflow: [
      'Crea un cliente nuevo con "+ Nuevo" (o Ctrl+N) completando su razón social, CUIT y datos de contacto.',
      'Añade múltiples contactos por cliente (administración, compras, jefes de obra) especificando el contacto principal.',
      'Filtra por nombre o activa el interruptor "Solo con deuda" para visualizar quiénes tienen saldos pendientes.',
      'Haz clic en el saldo o en el botón de cuenta para acceder al historial de comprobantes del cliente.',
    ],
    strengths: [
      'Validación automática de formato CUIT y coherencia de contactos.',
      'Soporte para múltiples personas de contacto dentro de la misma ficha comercial.',
      'Cálculo en tiempo real de deuda y saldo acumulado.',
      'Exportación rápida del padrón de clientes a hojas de cálculo o Word.',
    ],
    limitations: [
      'Un cliente que tenga proyectos, trabajos o facturas asociadas no puede ser eliminado del sistema.',
    ],
    tips: [
      'Carga con exactitud la condición de IVA y dirección fiscal para agilizar la emisión posterior de comprobantes.',
    ],
  },

  'clientes-cuenta-corriente': {
    id: 'clientes-cuenta-corriente',
    title: 'Cuenta Corriente y Estado de Deuda',
    subtitle: 'Auditoría de facturación, cobros imputados y días de mora',
    purpose:
      'Brinda un desglose cronológico de todas las facturas emitidas al cliente, los pagos recibidos (totales o parciales) y los días de mora de comprobantes impagos.',
    workflow: [
      'Observa las métricas principales: Total Facturado, Total Cobrado y Saldo Pendiente de cobro.',
      'Por defecto se listan solo las facturas con saldo deudor; activa "Incluir pagadas" para ver el historial completo.',
      'Identifica las facturas en rojo con mora vencida para coordinar reclamos de pago.',
      'Registra nuevos pagos directamente sobre los comprobantes pendientes.',
    ],
    strengths: [
      'Monitoreo exacto de atrasos: cálculo diario de mora por comprobante.',
      'Trazabilidad de pagos parciales con saldo remanente individual.',
      'Conciliación contable inmediata con el libro de caja.',
    ],
    limitations: [
      'El saldo se deriva exclusivamente de facturas emitidas y sus respectivos pagos registrados en el sistema.',
    ],
    tips: [
      'Revisa la cuenta corriente antes de autorizar certificaciones o despachos a clientes con atrasos reiterados.',
    ],
  },

  'proyectos-overview': {
    id: 'proyectos-overview',
    title: 'Proyectos, Obras y Estructura Jerárquica',
    subtitle: 'Gestión global de contratos, estados y rentabilidad por cliente',
    purpose:
      'Un Proyecto agrupa todas las obras o frentes contratados con un cliente. Permite supervisar el estado general, la rentabilidad global acumulada y acceder velozmente a sus trabajos y tableros Kanban.',
    workflow: [
      'Crea un proyecto con "+ Nuevo Proyecto" seleccionando el cliente e ingresando nombre y ubicación.',
      'Crea los Trabajos presupuestados dentro de cada obra usando "+ Nuevo Trabajo" o desde el menú contextual de la fila.',
      'Haz clic en la flecha de la fila para desplegar los trabajos que pertenecen al proyecto.',
      'Gestiona los estados del proyecto (Activa, Pausada, Finalizada o Cancelada) desde las acciones directas.',
      'Haz clic derecho para saltar a la "Caja" del proyecto o para abrir su tablero en el "Kanban".',
    ],
    strengths: [
      'Árbol jerárquico (Proyecto ➔ Trabajos ➔ Órdenes) con cómputo consolidado de rentabilidad y montos.',
      'Cierre y transición en cascada respetando las reglas contables y de avance físico.',
      'Filtros rápidos por cliente, estado y texto en tiempo real.',
      'Columna "Número" configurable y centrada según preferencias del usuario.',
    ],
    limitations: [
      'Un proyecto que tiene trabajos asociados no puede eliminarse sin antes dar de baja sus obras hijas.',
      'Para finalizar un proyecto con trabajos abiertos se requiere resolución de dichos trabajos.',
    ],
    tips: [
      'Puedes ocultar o activar la columna "Número" en cualquier momento desde Configuración > General.',
      'Aprovecha el botón "+ Trabajo" en cada fila para cargar tareas sin salir de la vista general.',
    ],
  },

  'proyectos-caja': {
    id: 'proyectos-caja',
    title: 'Caja y Rentabilidad Específica de la Obra',
    subtitle: 'Flujo de fondos e imputación analítica del proyecto',
    purpose:
      'Permite aislar todos los ingresos (certificaciones cobradas) y egresos (materiales, combustible, viáticos) asignados exclusivamente a este proyecto.',
    workflow: [
      'Accede desde el menú contextual de cualquier proyecto seleccionando "Ver Caja".',
      'Revisa los movimientos imputados específicamente a los trabajos de esta obra.',
      'Controla que los gastos acumulados no desvíen el margen de rentabilidad previsto.',
    ],
    strengths: [
      'Visión analítica pura: aísla los números de la obra sin mezclar con la caja central de la empresa.',
      'Trazabilidad directa con el libro diario general.',
    ],
    limitations: [
      'Solo computa movimientos que hayan sido vinculados a este proyecto al registrarse en Caja.',
    ],
    tips: [
      'Recuerda seleccionar siempre el Proyecto en el formulario de Movimientos para mantener la caja de obra al día.',
    ],
  },

  'trabajos-overview': {
    id: 'trabajos-overview',
    title: 'Gestión de Trabajos y Tareas de Obra',
    subtitle: 'Contratos operativos, presupuestos y control de avance físico',
    purpose:
      'Un Trabajo representa la unidad técnica de ejecución dentro de un Proyecto (ej: tendido eléctrico, montaje de tableros). Define fechas de ejecución, presupuesto asignado y contiene las Órdenes de Trabajo.',
    workflow: [
      'Crea un nuevo trabajo pulsando "+ Nuevo" o directamente desde la vista de Proyectos.',
      'Asigna el Proyecto padre, la descripción técnica de la tarea y el presupuesto convenido.',
      'Define la fecha de inicio y la fecha estimada o real de finalización.',
      'Supervisa la transición de estados: Presupuestado ➔ En Proceso ➔ Pausado ➔ Finalizado.',
      'Haz clic en la fila para abrir la ficha integral del trabajo y gestionar sus órdenes asociadas.',
    ],
    strengths: [
      'Vinculación garantizada al Proyecto padre: previene imputaciones huérfanas sin cliente.',
      'Sincronización automática con la columna correspondiente del tablero Kanban.',
      'Control cruzado entre presupuesto estimado e ingresos reales de certificados.',
    ],
    limitations: [
      'Un trabajo presupuestado que nunca inició no puede saltar directo a "Finalizado" (debe cancelarse o pasar a "En Proceso").',
      'No es posible eliminar un trabajo si contiene órdenes de trabajo, certificados o movimientos de caja.',
    ],
    tips: [
      'Revisa el tablero Kanban de Trabajos para gestionar visualmente las prioridades del personal en obra.',
    ],
  },

  'trabajos-detalle': {
    id: 'trabajos-detalle',
    title: 'Ficha Integral del Trabajo',
    subtitle: 'Seguimiento de órdenes de trabajo, certificados y rentabilidad',
    purpose:
      'Muestra la información técnica, fechas, cliente, estado actual y métricas consolidadas del trabajo en ejecución.',
    workflow: [
      'Comprueba las fechas pactadas y el estado de avance del trabajo.',
      'Accede a las órdenes de trabajo vinculadas para auditar tareas de cuadrillas.',
      'Cambia el estado de avance conforme avance la ejecución en campo.',
    ],
    strengths: [
      'Visión integral de avance físico y contractual en un solo panel.',
      'Navegación fluida hacia el proyecto contenedor.',
    ],
    limitations: [
      'Los cambios de estado deben respetar la máquina de estados del dominio.',
    ],
    tips: [
      'Finaliza el trabajo una vez que todos los certificados de avance hayan sido aprobados por el cliente.',
    ],
  },

  'ordenes-overview': {
    id: 'ordenes-overview',
    title: 'Órdenes de Trabajo y Cómputo Métrico',
    subtitle: 'Desglose pormenorizado de tareas, ítems y contratos de obra',
    purpose:
      'Una Orden de Trabajo desglosa un contrato en ítems medibles (unidades, metros, bocas de conexión) con cantidades y precios unitarios. Es la base técnica sobre la cual se emitirán los Certificados de Avance.',
    workflow: [
      'Crea una orden con "+ Nuevo" (o Ctrl+N) dentro del trabajo correspondiente.',
      'Carga cada uno de los ítems cotizados: descripción, unidad de medida, cantidad y precio unitario.',
      'Aplica porcentaje de ajuste UOCRA o descuentos globales si corresponde según pliego.',
      'Guarda la orden y haz clic en su fila para ingresar al detalle de cómputo y certificación.',
    ],
    strengths: [
      'Cálculo financiero exacto de subtotales por ítem y monto total presupuestado.',
      'Protección contra sobre-certificación: no permite certificar más del 100% de la cantidad pactada.',
      'Editor amplio de planilla para manipular cómodamente cómputos extensos.',
    ],
    limitations: [
      'Una orden que posea certificados emitidos no puede modificarse en sus ítems ni eliminarse.',
    ],
    tips: [
      'Utiliza unidades claras (m, u, gl) y descripciones detalladas para evitar discrepancias con el comitente en obra.',
    ],
  },

  'ordenes-detalle': {
    id: 'ordenes-detalle',
    title: 'Planilla de Ítems y Certificación de Avance',
    subtitle: 'Control de porcentajes acumulados y emisión de certificados',
    purpose:
      'Permite inspeccionar cada ítem de la orden, su avance porcentual a la fecha, monto facturable remanente y emitir nuevos Certificados de Obra.',
    workflow: [
      'Revisa la planilla de ítems y las barras de avance porcentual de cada renglón.',
      'Haz clic en "Emitir Certificado" para abrir el asistente de certificación periódica.',
      'El sistema precarga el acumulado anterior; solo debes tipear el porcentaje ejecutado en la quincena/mes.',
      'Al guardar, se crea el Certificado oficial con numeración correlativa inmutable.',
    ],
    strengths: [
      'Validación en tiempo real: resalta en rojo si algún ítem sobrepasa el 100% antes de grabar.',
      'Barras gráficas de avance por ítem para una lectura visual instantánea.',
      'Listado histórico inferior con todos los certificados expedidos contra esta orden.',
    ],
    limitations: [
      'No es posible emitir un certificado si ningún ítem tiene avance en el período.',
    ],
    tips: [
      'Recuerda que solo se puede anular el último certificado emitido de una orden si requieres corregir una medición.',
    ],
  },
}
