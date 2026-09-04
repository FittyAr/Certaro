import type { HelpTopic } from '../helpRegistry'

export const personalHelp: Record<string, HelpTopic> = {
  'empleados-overview': {
    id: 'empleados-overview',
    title: 'Padrón de Empleados y Personal Operativo',
    subtitle: 'Fichas de personal, tarifas de jornal y reglas de liquidación',
    purpose:
      'Gestiona el legajo de operarios, electricistas, capataces y administrativos: datos personales, cargo, tarifa por jornada o mensual, y multiplicadores de horas extras o fines de semana.',
    workflow: [
      'Crea un empleado nuevo con "+ Nuevo" (o Ctrl+N) cargando nombre, DNI, cargo y tarifa vigente.',
      'Establece los multiplicadores para sábados, domingos y feriados según convenio o acuerdo laboral.',
      'Si un operario concluye su relación laboral, ingresa la fecha de egreso para cerrar su legajo.',
      'Filtra por nombre o cargo para consultar rápidamente la plantilla activa.',
    ],
    strengths: [
      'Congelamiento de tarifas históricas: aumentos posteriores no modifican liquidaciones pasadas.',
      'Control de fechas de ingreso y egreso con validaciones cronológicas.',
      'Integración inmediata con la grilla de Asistencia diaria y el cálculo de liquidaciones.',
    ],
    limitations: [
      'Un empleado con asistencias registradas o liquidaciones no puede eliminarse del sistema.',
    ],
    tips: [
      'Desmarca la casilla "Activo" para dar de baja provisoria a un operario sin borrar sus datos.',
    ],
  },

  'asistencia-overview': {
    id: 'asistencia-overview',
    title: 'Control Diario de Asistencia en Obra',
    subtitle: 'Grilla matricial interactiva y carga masiva de jornadas',
    purpose:
      'Permite asentar día por día la presencia, media jornada, ausencia o licencia de cada operario, reconociendo feriados nacionales y días no laborables.',
    workflow: [
      'Selecciona el período (quincena o mes) en las fechas de la cabecera.',
      'Haz clic directamente en cada celda para alternar el tipo de jornada: Presente (P), Ausente (A), Media Jornada (M), Vacaciones (V) o Vacío.',
      'Usa "Carga Masiva" para aplicar jornadas idénticas a un grupo de operarios durante un intervalo.',
      'Los feriados del calendario nacional aparecen destacados visualmente.',
    ],
    strengths: [
      'Guardado automático instantáneo: cada clic se asienta de inmediato en la base de datos de manera atómica.',
      'Base de cálculo de liquidaciones: el asistente de sueldos lee directamente los días computados aquí.',
      'Bloqueo de fechas fuera del período de contratación de cada empleado.',
    ],
    limitations: [
      'El rango de visualización tiene un límite técnico de días para garantizar rapidez y fluidez en la tabla.',
    ],
    tips: [
      'Lleva la asistencia al día para que liquidar sueldos al final de la quincena tome menos de un minuto.',
    ],
  },

  'liquidaciones-overview': {
    id: 'liquidaciones-overview',
    title: 'Liquidación de Sueldos y Jornales',
    subtitle: 'Cálculo por lote con lectura de asistencia y descuento de adelantos',
    purpose:
      'Liquida quincenas o meses: calcula días trabajados desde la asistencia, aplica tarifas y multiplicadores, deduce adelantos entregados en caja y genera los recibos.',
    workflow: [
      'Haz clic en "+ Nueva Liquidación" (o Ctrl+N) para iniciar el asistente de cálculo.',
      'Selecciona el período y marca los empleados que integran la liquidación.',
      'El sistema sugiere automáticamente días trabajados, tarifas y adelantos pendientes de caja.',
      'Revisa los números, ajusta conceptos si hace falta y confirma el lote.',
      'Haz clic en cualquier liquidación para auditar el recibo detallado o imprimirlo.',
    ],
    strengths: [
      'Deducción automática de adelantos de caja: evita pagar dos veces montos adelantados al personal.',
      'Procesamiento en lote atómico para liquidar a todo el plantel en un solo paso.',
      'Detección de solapamientos: previene liquidar dos veces el mismo período a un operario.',
    ],
    limitations: [
      'Una liquidación ya entregada queda bloqueada para salvaguardar la validez jurídica del recibo.',
    ],
    tips: [
      'Verifica que la asistencia de la quincena esté completa antes de abrir el asistente de liquidación.',
    ],
  },

  'liquidaciones-detalle': {
    id: 'liquidaciones-detalle',
    title: 'Recibo y Detalle de Liquidación',
    subtitle: 'Desglose de haberes, descuentos aplicados y reglas congeladas',
    purpose:
      'Muestra el recibo individual de un operario: días liquidados, tarifa aplicada, adicionales de fin de semana, adelantos deducidos y monto neto final.',
    workflow: [
      'Comprueba el total a cobrar y el desglose de importes.',
      'Agrega notas u observaciones complementarias en la sección inferior.',
      'Si se detecta un error de cálculo antes del pago, pulsa "Anular" para restablecer los adelantos y reliquidar.',
    ],
    strengths: [
      'Tarifas y multiplicadores congelados permanentemente en el momento de la liquidación.',
      'Restitución automática de adelantos de caja en caso de anulación.',
    ],
    limitations: [
      'Los importes no se editan a mano directamente: si hay un error debe anularse y generarse nuevamente.',
    ],
    tips: [
      'Imprime el recibo detallado para entregarlo al operario junto con el pago correspondiente.',
    ],
  },
}
