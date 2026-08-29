import { onScopeDispose } from 'vue'

/**
 * Keyboard shortcuts of `docs/10-navegacion-y-atajos.md` §4.
 *
 * Every module declares its own handlers here and the shell has no switch over the current
 * screen. If a module does not declare a handler the shortcut simply does not apply, which is
 * explicit rather than an oversight — in the legacy system `F5` did nothing on 6 of 15 screens
 * and nobody noticed.
 *
 * `Escape` is not handled here: it is a cascade and lives in `useEscapeStack`, because the order
 * in which things close matters.
 */

export interface ShortcutHandler {
  handler: (event: KeyboardEvent) => void
  /** Evaluated at press time; a false result lets the key through. */
  when?: () => boolean
  /** By default a shortcut is inert while typing. `Ctrl+S` is the expected exception. */
  allowInInput?: boolean
}

export type ShortcutMap = Record<string, ShortcutHandler | ((event: KeyboardEvent) => void)>

const TEXT_INPUTS = new Set(['INPUT', 'TEXTAREA', 'SELECT'])

/** `ctrl+shift+enter` for a given event, in the same normalised form the map uses. */
export function comboOf(event: KeyboardEvent): string {
  const parts: string[] = []
  if (event.ctrlKey || event.metaKey) parts.push('ctrl')
  if (event.altKey) parts.push('alt')
  if (event.shiftKey) parts.push('shift')

  const key = event.key.toLowerCase()
  parts.push(key === ' ' ? 'space' : key)
  return parts.join('+')
}

export function isTypingIn(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false
  return TEXT_INPUTS.has(target.tagName) || target.isContentEditable === true
}

function normalise(entry: ShortcutHandler | ((event: KeyboardEvent) => void)): ShortcutHandler {
  return typeof entry === 'function' ? { handler: entry } : entry
}

/** Registers the map for as long as the calling scope lives. */
export function useShortcuts(map: ShortcutMap, target: EventTarget = window): void {
  const entries = new Map(Object.entries(map).map(([k, v]) => [k.toLowerCase(), normalise(v)]))

  function onKeydown(event: Event) {
    if (!(event instanceof KeyboardEvent)) return

    const entry = entries.get(comboOf(event))
    if (!entry) return
    if (isTypingIn(event.target) && !entry.allowInInput) return
    if (entry.when && !entry.when()) return

    event.preventDefault()
    entry.handler(event)
  }

  target.addEventListener('keydown', onKeydown)
  onScopeDispose(() => target.removeEventListener('keydown', onKeydown))
}
