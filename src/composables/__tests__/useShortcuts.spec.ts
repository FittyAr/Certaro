import { effectScope } from 'vue'
import { describe, expect, it, vi } from 'vitest'

import { comboOf, isTypingIn, useShortcuts } from '@/composables/useShortcuts'

function press(target: EventTarget, init: KeyboardEventInit & { key: string }) {
  const event = new KeyboardEvent('keydown', { ...init, bubbles: true, cancelable: true })
  target.dispatchEvent(event)
  return event
}

function withShortcuts(map: Parameters<typeof useShortcuts>[0], target: EventTarget) {
  const scope = effectScope()
  scope.run(() => useShortcuts(map, target))
  return () => scope.stop()
}

describe('useShortcuts', () => {
  it('normaliza la combinacion en el orden ctrl, alt, shift', () => {
    const event = new KeyboardEvent('keydown', {
      key: 'Enter',
      ctrlKey: true,
      shiftKey: true,
    })
    expect(comboOf(event)).toBe('ctrl+shift+enter')
  })

  it('dispara el manejador y evita la accion por defecto del navegador', () => {
    const target = new EventTarget()
    const handler = vi.fn()
    withShortcuts({ f5: handler }, target)

    const event = press(target, { key: 'F5' })

    expect(handler).toHaveBeenCalledOnce()
    expect(event.defaultPrevented).toBe(true)
  })

  it('un atajo no se dispara mientras se escribe', () => {
    const input = document.createElement('input')
    document.body.append(input)
    const handler = vi.fn()
    withShortcuts({ 'ctrl+n': handler }, input)

    press(input, { key: 'n', ctrlKey: true })

    expect(handler).not.toHaveBeenCalled()
    input.remove()
  })

  it('ctrl+s si se dispara dentro de un campo, que es la excepcion esperada', () => {
    const input = document.createElement('input')
    document.body.append(input)
    const handler = vi.fn()
    withShortcuts({ 'ctrl+s': { handler, allowInInput: true } }, input)

    press(input, { key: 's', ctrlKey: true })

    expect(handler).toHaveBeenCalledOnce()
    input.remove()
  })

  it('la condicion when deja pasar la tecla cuando no aplica', () => {
    const target = new EventTarget()
    const handler = vi.fn()
    withShortcuts({ 'ctrl+s': { handler, when: () => false } }, target)

    press(target, { key: 's', ctrlKey: true })

    expect(handler).not.toHaveBeenCalled()
  })

  it('los atajos se desregistran al destruirse el ambito', () => {
    const target = new EventTarget()
    const handler = vi.fn()
    const stop = withShortcuts({ f5: handler }, target)

    stop()
    press(target, { key: 'F5' })

    expect(handler).not.toHaveBeenCalled()
  })

  it('reconoce donde se esta escribiendo', () => {
    const input = document.createElement('textarea')
    expect(isTypingIn(input)).toBe(true)
    expect(isTypingIn(document.createElement('div'))).toBe(false)
  })
})
