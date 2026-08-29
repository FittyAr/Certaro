import { effectScope } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
  handleEscape,
  openLayers,
  resetEscapeStack,
  useEscapeLayer,
} from '@/composables/useEscapeStack'

/** The Escape cascade of `docs/10-navegacion-y-atajos.md` §4.3. */

function layer(kind: Parameters<typeof useEscapeLayer>[0], close: () => boolean) {
  // A layer is registered inside a scope because it deregisters itself when the scope is disposed.
  const scope = effectScope()
  const api = scope.run(() => useEscapeLayer(kind, close))!
  api.push()
  return { ...api, stop: () => scope.stop() }
}

describe('useEscapeStack', () => {
  beforeEach(() => resetEscapeStack())

  it('cierra siempre la capa de arriba', async () => {
    const closeDrawer = vi.fn(() => true)
    const closePalette = vi.fn(() => true)

    layer('drawer', closeDrawer)
    layer('palette', closePalette)

    await handleEscape()

    expect(closePalette).toHaveBeenCalledOnce()
    expect(closeDrawer).not.toHaveBeenCalled()
    expect(openLayers()).toEqual(['drawer'])
  })

  it('respeta el orden con paleta, drawer y desplegable abiertos', async () => {
    const order: string[] = []
    layer('drawer', () => (order.push('drawer'), true))
    layer('menu', () => (order.push('menu'), true))
    layer('palette', () => (order.push('palette'), true))

    await handleEscape()
    await handleEscape()
    await handleEscape()

    expect(order).toEqual(['palette', 'menu', 'drawer'])
  })

  it('una capa que no consume la tecla se queda apilada', async () => {
    layer('filters', () => false)
    expect(await handleEscape()).toBe(false)
    expect(openLayers()).toEqual(['filters'])
  })

  it('sin nada abierto la tecla no hace nada', async () => {
    // The legacy system navigated back as the last step, which took the user out of a screen
    // they had not asked to leave.
    expect(await handleEscape()).toBe(false)
  })

  it('una capa se desregistra al destruirse su ambito', async () => {
    const closed = vi.fn(() => true)
    const drawer = layer('drawer', closed)
    drawer.stop()

    expect(openLayers()).toEqual([])
    expect(await handleEscape()).toBe(false)
    expect(closed).not.toHaveBeenCalled()
  })
})
