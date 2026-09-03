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
}
