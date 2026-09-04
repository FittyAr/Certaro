import type { HelpTopic } from '../helpRegistry'

export const finanzasHelp: Record<string, HelpTopic> = {
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

  'facturas-overview': {
    id: 'facturas-overview',
    title: 'Facturación, Cobranzas y Registro de Pagos',
    subtitle: 'Control de cuentas a cobrar, medios de pago y cobranzas parciales',
    purpose:
      'Gestiona el ciclo de facturación comercial: emisión de facturas a clientes, vinculación con certificados de obra aprobados, control de mora y cobros parciales o totales.',
    workflow: [
      'Crea una factura con "+ Nuevo" (o Ctrl+N) seleccionando el cliente, número de comprobante y fechas.',
      'Carga los montos gravados, el IVA correspondiente y vincula los certificados facturados si aplica.',
      'Supervisa los estados: Borrador ➔ Emitida ➔ Pago Parcial ➔ Pagada (o Vencida si pasó el vencimiento).',
      'Registra pagos haciendo clic en el ícono de pago de la fila, eligiendo medio (Transferencia, Cheque, Efectivo).',
      'Si una factura debe dejarse sin efecto comercial, utiliza la acción de Anulación.',
    ],
    strengths: [
      'Cálculo automático de mora y actualización inmediata de la Cuenta Corriente.',
      'Filtros rápidos de alta visibilidad para aislar comprobantes impagos o vencidos.',
      'Trazabilidad completa de cobranzas fraccionadas en distintos medios de pago.',
    ],
    limitations: [
      'Una factura cancelada en su totalidad o anulada no admite nuevos pagos.',
      'La fecha de vencimiento debe ser igual o posterior a la fecha de emisión.',
    ],
    tips: [
      'Activa el filtro "Solo vencidas" a primera hora para priorizar las gestiones de cobranza del día.',
    ],
  },
}
