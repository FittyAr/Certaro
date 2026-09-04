import type { TreeNode } from 'primevue/treenode'
import { ref, computed, type Ref } from 'vue'
import { useTrabajosStore, type TrabajoListItem } from '@/stores/useTrabajosStore'
import type { ProyectoListItem } from '@/stores/useProyectosStore'

export function useProyectosTreeNodes(
  rows: Ref<ProyectoListItem[]>,
  notify: (e: unknown) => unknown,
) {
  const trabajosStore = useTrabajosStore()
  const expandedKeys = ref<Record<string, boolean>>({})
  const trabajosMap = ref<Map<string, TrabajoListItem[]>>(new Map())
  const loadingTrabajos = ref<Set<string>>(new Set())

  const treeValue = computed<TreeNode[]>(() => {
    return rows.value.map((proyecto) => {
      const isExpanded = expandedKeys.value[proyecto.id] === true
      const trabajos = trabajosMap.value.get(proyecto.id) ?? []
      const isLoading = loadingTrabajos.value.has(proyecto.id)

      const children: TreeNode[] | undefined = isExpanded
        ? isLoading
          ? [{ key: `${proyecto.id}-loading`, data: { isLoading: true }, leaf: true }]
          : trabajos.length > 0
            ? trabajos.map((trab) => ({
                key: trab.id,
                data: {
                  isTrabajo: true,
                  trabajo: trab,
                  numero: '—',
                  nombre: trab.descripcion,
                  clienteNombre: trab.clienteNombre,
                  localidad: '—',
                  estado: trab.estado,
                  trabajosCount: null,
                  presupuesto: trab.presupuesto,
                  rentabilidad: null,
                  proyecto: null,
                },
                leaf: true,
              }))
            : [{ key: `${proyecto.id}-empty`, data: { isEmpty: true }, leaf: true }]
        : undefined

      return {
        key: proyecto.id,
        data: {
          isProyecto: true,
          proyecto,
          numero: proyecto.numero,
          nombre: proyecto.nombre,
          clienteNombre: proyecto.clienteNombre,
          localidad: proyecto.localidad,
          estado: proyecto.estado,
          trabajosCount: proyecto.trabajosCount,
          rentabilidad: proyecto.rentabilidad,
        },
        children,
        leaf: proyecto.trabajosCount === 0,
      }
    })
  })

  async function onExpand(node: TreeNode): Promise<void> {
    const proyectoId = String(node.key)
    const proyecto = rows.value.find((p) => p.id === proyectoId)
    if (!proyecto || proyecto.trabajosCount === 0) return
    if (trabajosMap.value.has(proyectoId)) return

    loadingTrabajos.value = new Set(loadingTrabajos.value).add(proyectoId)
    try {
      const res = await trabajosStore.fetchPaged({
        page: 1,
        pageSize: 100,
        filtro: { proyectoId } as unknown as Record<string, unknown>,
        sortDir: 'Asc',
      })
      const next = new Map(trabajosMap.value)
      next.set(proyectoId, res.items)
      trabajosMap.value = next
    } catch (e) {
      notify(e)
    } finally {
      const next = new Set(loadingTrabajos.value)
      next.delete(proyectoId)
      loadingTrabajos.value = next
    }
  }

  function handleExpand(node: TreeNode): void {
    expandedKeys.value = { ...expandedKeys.value, [String(node.key)]: true }
    void onExpand(node)
  }

  function handleCollapse(node: TreeNode): void {
    const key = String(node.key)
    const next = { ...expandedKeys.value }
    delete next[key]
    expandedKeys.value = next
  }

  function pruneRemoved(validIds: Set<string>): void {
    const next = new Map<string, TrabajoListItem[]>()
    for (const [k, v] of trabajosMap.value.entries()) {
      if (validIds.has(k)) next.set(k, v)
    }
    trabajosMap.value = next

    const nextKeys: Record<string, boolean> = {}
    for (const k of Object.keys(expandedKeys.value)) {
      if (validIds.has(k)) nextKeys[k] = true
    }
    expandedKeys.value = nextKeys
  }

  return {
    expandedKeys,
    treeValue,
    handleExpand,
    handleCollapse,
    pruneRemoved,
  }
}
