import { onScopeDispose, ref } from 'vue'

/**
 * The Escape cascade of `docs/10-navegacion-y-atajos.md` §4.3.
 *
 * One implementation for the whole application, based on a stack: whatever opens last is what
 * Escape closes first. The legacy system had a global cascade plus local Escape bindings in six
 * edit screens, so the behaviour depended on where the focus happened to be.
 *
 * With nothing open, Escape does nothing. It deliberately does **not** navigate back, which in the
 * legacy system took the user out of a screen they had not asked to leave.
 */

export type EscapeLayerKind =
  'palette' | 'modal' | 'drawer' | 'menu' | 'filters' | 'sidebar-overlay'

interface Layer {
  id: number
  kind: EscapeLayerKind
  /** Returns `true` when the layer consumed the key. */
  close: () => boolean | Promise<boolean>
}

const layers = ref<Layer[]>([])
let nextId = 1

/** Registers a layer while the calling scope is alive. */
export function useEscapeLayer(kind: EscapeLayerKind, close: () => boolean | Promise<boolean>) {
  const id = nextId++

  function push() {
    if (!layers.value.some((l) => l.id === id)) {
      layers.value.push({ id, kind, close })
    }
  }

  function pop() {
    layers.value = layers.value.filter((l) => l.id !== id)
  }

  onScopeDispose(pop)

  return { push, pop }
}

/** Closes the topmost layer. Returns whether the key was consumed. */
export async function handleEscape(): Promise<boolean> {
  const top = layers.value.at(-1)
  if (!top) return false

  const consumed = await top.close()
  if (consumed) {
    layers.value = layers.value.filter((l) => l.id !== top.id)
  }
  return consumed
}

/** The kinds currently open, outermost first. Exposed for tests. */
export function openLayers(): EscapeLayerKind[] {
  return layers.value.map((l) => l.kind)
}

/** Clears the stack. Only for tests, which share module state between cases. */
export function resetEscapeStack(): void {
  layers.value = []
}
