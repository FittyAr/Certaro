import type { HelpTopic } from '../helpRegistry'

export const sistemaHelp: Record<string, HelpTopic> = {
  'reportes-overview': {
    id: 'reportes-overview',
    title: 'Centro de Reportes y Exportación Documental',
    subtitle: 'Generación de planillas contables, recibos y actas en Excel y Word',
    purpose:
      'Permite consolidar la información del sistema y exportarla en formatos abiertos (CSV/Excel y DOCX/Word) para balances de caja, entrega a comitentes o archivo fiscal.',
    workflow: [
      'Ubica la tarjeta del reporte que necesitas (Movimientos de caja, Recibo de liquidación o Certificado).',
      'Configura los parámetros: intervalo de fechas o selección del comprobante en el desplegable.',
      'Haz clic en el botón "Exportar" y elige el formato de salida deseado.',
      'Elige en tu equipo la carpeta donde guardar el documento generado.',
    ],
    strengths: [
      'Generación nativa ultrarrápida sin requerir tener instalado Microsoft Office.',
      'Maquetación prolija y profesional lista para presentar o imprimir.',
      'Privacidad total: el archivo se descarga directo a tu almacenamiento local.',
    ],
    limitations: [
      'Para exportar un recibo o certificado específico, dicho comprobante debe estar emitido previamente en el sistema.',
    ],
    tips: [
      'Para auditorías contables o balances de fin de año, exporta el Libro de Movimientos fijando el ejercicio fiscal completo.',
    ],
  },

  'categorias-overview': {
    id: 'categorias-overview',
    title: 'Categorías de Gastos e Ingresos',
    subtitle: 'Estructura en árbol para imputación y análisis de costos',
    purpose:
      'Permite clasificar los movimientos de dinero en una jerarquía limpia de categorías madre y subcategorías (ej: Materiales ➔ Cables, Combustibles ➔ Gasoil) para auditar costos con precisión.',
    workflow: [
      'Crea una categoría nueva con "+ Nuevo" (o Ctrl+N).',
      'Si es una categoría principal (raíz), deja vacío el campo "Categoría Padre". Para una subcategoría, selecciona su padre.',
      'Elige un color identificatorio para visualizarla claramente en los gráficos y tablas.',
      'Filtra por nombre o marca "Solo raíz" para inspeccionar las familias principales de gastos.',
    ],
    strengths: [
      'Jerarquía ordenada de un nivel que simplifica la imputación diaria de los usuarios.',
      'Protección de integridad: una categoría con subcategorías o movimientos imputados no puede eliminarse.',
    ],
    limitations: [
      'No se permite seleccionar una categoría como padre de sí misma ni armar ciclos jerárquicos.',
    ],
    tips: [
      'Mantén pocas categorías madre y usa subcategorías específicas para agilizar los reportes mensuales.',
    ],
  },

  'tipos-movimiento-overview': {
    id: 'tipos-movimiento-overview',
    title: 'Tipos de Movimiento de Caja',
    subtitle: 'Naturaleza contable (ingreso o egreso) y conceptos fijos',
    purpose:
      'Define los tipos fundamentales del libro diario de caja (Cobranza, Pago de Gastos, Adelanto de Sueldo). Cada tipo establece el signo (+ o -) que impacta en el balance de fondos.',
    workflow: [
      'Revisa los tipos preconfigurados del sistema.',
      'Crea nuevos tipos personalizados con "+ Nuevo" si tu operativa comercial lo demanda.',
      'Indica claramente si el tipo suma fondos (Ingreso) o descuenta dinero (Egreso).',
    ],
    strengths: [
      'Blindaje del balance histórico: los tipos raíz del sistema no pueden eliminarse ni cambiar de signo.',
      'Consistencia matemática en el servidor de base de datos.',
    ],
    limitations: [
      'Una vez utilizado en movimientos, el signo no puede ser alterado.',
    ],
    tips: [
      'Evita duplicar tipos; utiliza preferentemente las Categorías para discriminar destinos de gastos.',
    ],
  },

  'feriados-overview': {
    id: 'feriados-overview',
    title: 'Calendario de Feriados Nacionales',
    subtitle: 'Sincronización oficial y reglas de recargo laboral',
    purpose:
      'Administra los feriados nacionales y días no laborables del año. Este calendario es leído por la Asistencia y el módulo de Liquidaciones para liquidar recargos correspondientes.',
    workflow: [
      'Presiona "Sincronizar Feriados" para traer las fechas patrias y puentes oficiales de Argentina.',
      'Si tienes feriados locales o fiestas patronales, agrégalos manualmente indicando fecha y nombre.',
      'Consulta distintos años utilizando el selector numérico superior.',
    ],
    strengths: [
      'Inmunidad de feriados manuales: los asuetos cargados a mano nunca son borrados por la sincronización.',
      'Actualización instantánea mediante API oficial en un solo clic.',
    ],
    limitations: [
      'No se pueden cargar dos feriados con idéntica fecha dentro del mismo año.',
    ],
    tips: [
      'Sincroniza al inicio de cada año calendario para mantener las reglas de asistencia al día.',
    ],
  },

  'configuracion-overview': {
    id: 'configuracion-overview',
    title: 'Configuración General del Sistema',
    subtitle: 'Personalización de empresa, integraciones, liquidación y apariencia',
    purpose:
      'Centro de control global donde se ajustan parámetros fiscales de la empresa, cotizaciones de dólar, canales de mensajería, reglas de liquidación y visibilidad de columnas.',
    workflow: [
      'Navega entre las pestañas: General, Negocio, Liquidaciones, Comunicaciones, Integraciones y Sistema.',
      'En General, configura el tema visual, atajos y visibilidad de columnas como la columna Número en Proyectos.',
      'En Negocio, carga la razón social y CUIT para el membrete de reportes.',
      'En Integraciones, define qué dólares consultar automáticamente.',
      'Presiona "Guardar Cambios" para aplicar las modificaciones.',
    ],
    strengths: [
      'Configuración modular y reactiva aplicada en tiempo real.',
      'Alertas si intentas abandonar la pantalla con cambios sin confirmar.',
    ],
    limitations: [
      'Algunos parámetros de base de datos requieren reinicio de la aplicación.',
    ],
    tips: [
      'Completa los datos de tu empresa en Negocio para que todos tus reportes salgan listos para presentar.',
    ],
  },
}
