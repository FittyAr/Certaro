import type { HelpTopic } from '../helpRegistry'

export const obrasHelp: Record<string, HelpTopic> = {
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

  'certificados-overview': {
    id: 'certificados-overview',
    title: 'Historial de Certificados de Obra',
    subtitle: 'Comprobantes oficiales de avance físico para facturación',
    purpose:
      'Registra todas las actas de avance emitidas contra órdenes de trabajo. Cada certificado congela la medición de los ítems de obra y fundamenta la emisión de las facturas de cobro al comitente.',
    workflow: [
      'Los certificados no se crean de cero en esta pantalla: se emiten desde el detalle de cada Orden de Trabajo.',
      'Filtra el historial por Cliente, Proyecto o fechas de emisión para auditorías.',
      'Haz clic en "Ver" sobre cualquier fila para acceder a la copia congelada del acta.',
      'Si existió un error de cómputo, el sistema permite anular únicamente el último certificado emitido de esa orden.',
    ],
    strengths: [
      'Inmutabilidad legal y contractual: guarda copia fiel de precios y porcentajes vigentes al emitir.',
      'Numeración correlativa automática por orden de trabajo.',
      'Vínculo directo con facturas y órdenes.',
    ],
    limitations: [
      'Para emitir un nuevo certificado se debe acceder desde Órdenes de Trabajo.',
      'Solo se puede anular el último certificado de una orden (principio de correlatividad física).',
    ],
    tips: [
      'Utiliza el número de certificado al facturar para que el cliente identifique rápidamente qué avance está pagando.',
    ],
  },

  'certificados-detalle': {
    id: 'certificados-detalle',
    title: 'Detalle y Acta del Certificado de Obra',
    subtitle: 'Medición porcentual congelada y montos certificados del período',
    purpose:
      'Muestra el desglose de los ítems certificados en esta acta, el monto neto devengado, el ajuste UOCRA aplicado y permite editar observaciones técnicas.',
    workflow: [
      'Revisa los importes certificados del período y el porcentaje acumulado.',
      'Modifica o agrega observaciones de inspección si es necesario.',
      'Haz clic en "Ver Orden" para inspeccionar la planilla madre de cómputo.',
      'Si se trata del último certificado de la orden y hubo un error, puedes presionar "Anular" para recuperar el cupo.',
    ],
    strengths: [
      'Cálculo inmutable protegido contra modificaciones posteriores de la orden de trabajo.',
      'Edición ágil de notas técnicas sin riesgo de corromper la contabilidad.',
    ],
    limitations: [
      'Los montos y porcentajes no se pueden alterar una vez emitido el documento.',
    ],
    tips: [
      'Guarda o imprime el detalle del certificado para acompañarlo con la firma del inspector técnico.',
    ],
  },
}
