import type { HelpTopic } from '../helpRegistry'

export const dashboardHelp: Record<string, HelpTopic> = {
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
