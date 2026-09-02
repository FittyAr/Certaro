# Certaro v2 - Módulo Calendario y Planificador

## 1. Visión General

El módulo Calendario proporciona una vista temporal unificada para programar trabajos, reuniones, entregas y mantenimientos, así como la asignación de recursos humanos y materiales.

## 2. Vistas del Calendario

1. **Vista Mes**: Cuadrícula de 7 columnas con conteo de eventos y marcadores de estado.
2. **Vista Semana**: Grilla horaria (07:00 a 20:00) dividida en los 7 días de la semana en curso.
3. **Vista Día**: Agenda detallada por franja horaria con los recursos asignados.
4. **Vista de Recursos (Resource Day View)**:
   - Muestra el día seleccionado con columnas dedicadas a cada recurso activo (empleados, grúas, vehículos, herramientas).
   - Permite visualizar de un vistazo la carga de trabajo y disponibilidad de cada técnico o equipo de trabajo.

## 3. Proyección de Eventos Virtuales

El servicio de calendario realiza una integración en tiempo real entre eventos propios y entidades existentes de la base de datos:
- **Feriados Nacionales**: Se proyectan automáticamente como eventos de día completo con etiqueta virtual.
- **Trabajos Contratados**: Se proyectan en sus fechas de inicio y fin estimadas.
- **Vencimientos de Facturas**: Recordatorio de cobros pendientes.

## 4. Gestión de Recursos

- Sincronización automática de la nómina de empleados (`sincronizar_empleados_a_recursos`).
- Clasificación de recursos en grupos ("Personal", "Vehículos", "Equipos").
- Tipos de recurso soportados: `Empleado`, `Vehiculo`, `Herramienta`, `Proyecto`.
