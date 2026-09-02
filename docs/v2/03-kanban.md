# Certaro v2 - Módulo Tablero Kanban

## 1. Visión General

El módulo Kanban permite la gestión visual de flujos de trabajo tanto mediante tableros personalizados libres como a través de tableros Preset sincronizados automáticamente con las entidades centrales del negocio.

## 2. Tableros Preset y Sincronización Bidireccional

- **Tablero Preset "Trabajos en Curso"**:
  - Mapea las columnas a los estados de dominio:
    - *Pendiente* -> Estado: `Presupuestado`
    - *En Progreso* -> Estado: `EnProgreso`
    - *Pausado* -> Estado: `Pausado`
    - *Finalizado* -> Estado: `Finalizado`
    - *Cancelado* -> Estado: `Cancelado`
  - Mover una tarjeta actualiza automáticamente el estado del `Trabajo` correspondiente.
  - La sincronización (`sincronizar_preset`) detecta trabajos nuevos o modificados fuera del tablero y refleja sus tarjetas.

- **Tablero Preset "Órdenes de Trabajo"**:
  - Columnas: *Pendiente*, *En Ejecución*, *Completada*, *Cancelada*.
  - Sincronización bidireccional con las órdenes de trabajo técnicas.

## 3. Características de Tableros y Tarjetas

- **Columnas y Límites WIP**: Límite de tarjetas simultáneas por columna para evitar cuellos de botella.
- **Tarjetas**: Título, descripción markdown, prioridad (Baja, Normal, Alta, Urgente), fecha límite.
- **Checklists**: Lista de subtareas con estado completado y progreso porcentual visual.
- **Etiquetas**: Tags de colores clasificatorias compartidas por tablero.
- **Filtros**: Búsqueda textual y filtrado rápido por nivel de prioridad.
