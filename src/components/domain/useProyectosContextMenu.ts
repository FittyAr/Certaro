import type { TreeNode } from 'primevue/treenode'
import { computed, type ComputedRef, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import type { ProyectoListItem } from '@/stores/useProyectosStore'
import type { TrabajoListItem } from '@/stores/useTrabajosStore'

export interface ProyectosContextMenuHandlers {
  onProyectoEdit: (row: ProyectoListItem) => void
  onProyectoDelete: (row: ProyectoListItem) => void
  onProyectoTransition: (row: ProyectoListItem, estado: string) => void
  onProyectoCreateTrabajo: (row: ProyectoListItem) => void
  onTrabajoNavigate: (row: TrabajoListItem) => void
}

export function useProyectosContextMenu(
  contextNode: Ref<TreeNode | null>,
  handlers: ProyectosContextMenuHandlers,
): ComputedRef<any[]> {
  const { t } = useI18n()
  const router = useRouter()

  return computed(() => {
    const node = contextNode.value
    if (!node) return []
    const data = node.data as {
      isProyecto?: boolean
      isTrabajo?: boolean
      proyecto?: ProyectoListItem
      trabajo?: TrabajoListItem
    }

    if (data.isProyecto && data.proyecto) {
      const p = data.proyecto
      return [
        {
          label: t('General.View'),
          icon: 'pi pi-eye',
          command: () => void router.push({ name: 'proyecto-detalle', params: { proyectoId: p.id } }),
        },
        {
          label: t('General.Edit'),
          icon: 'pi pi-pencil',
          command: () => handlers.onProyectoEdit(p),
        },
        {
          label: t('General.Delete'),
          icon: 'pi pi-trash',
          disabled: !p.puedeEliminarse,
          command: () => handlers.onProyectoDelete(p),
        },
        { separator: true },
        {
          label: t('Actions.Proyecto.Finalizada'),
          icon: 'pi pi-check',
          disabled: p.estado === 'Finalizada' || p.estado === 'Cancelada',
          command: () => handlers.onProyectoTransition(p, 'Finalizada'),
        },
        { separator: true },
        {
          label: 'Agregar Trabajo',
          icon: 'pi pi-plus',
          disabled: p.estado === 'Finalizada' || p.estado === 'Cancelada',
          command: () => handlers.onProyectoCreateTrabajo(p),
        },
        {
          label: t('Proyectos.VerTrabajos'),
          icon: 'pi pi-hammer',
          command: () => void router.push({ name: 'proyecto-trabajos', params: { proyectoId: p.id } }),
        },
        {
          label: t('Proyectos.VerCaja'),
          icon: 'pi pi-wallet',
          command: () => void router.push({ name: 'proyecto-caja', params: { proyectoId: p.id } }),
        },
        {
          label: t('Proyectos.VerKanban') || 'Ver en Kanban',
          icon: 'pi pi-th-large',
          command: () => void router.push({ path: '/kanban', query: { proyectoId: p.id } }),
        },
      ]
    }

    if (data.isTrabajo && data.trabajo) {
      const tr = data.trabajo
      return [
        {
          label: t('General.View'),
          icon: 'pi pi-eye',
          command: () => handlers.onTrabajoNavigate(tr),
        },
        {
          label: t('General.Edit'),
          icon: 'pi pi-pencil',
          command: () => void router.push({ name: 'trabajo-detalle', params: { trabajoId: tr.id } }),
        },
      ]
    }

    return []
  })
}
