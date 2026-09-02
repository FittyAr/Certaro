import { defineStore } from 'pinia'
import { ref } from 'vue'

/**
 * Navigation state that outlives a single screen: the entity names the breadcrumb needs and the
 * recently visited destinations the command palette offers.
 */
export const useNavigationStore = defineStore('navigation', () => {
  /** Route name to the name of the entity it currently points at. */
  const names = ref<Record<string, string>>({})
  /** Route names, most recent first, capped at five. */
  const recent = ref<string[]>([])

  /**
   * Published by a detail screen once it loaded its record, so `Proyectos › Edificio Rivadavia 1230`
   * can show the site name instead of a placeholder.
   */
  function publishName(routeName: string, label: string): void {
    names.value = { ...names.value, [routeName]: label }
  }

  function resolvedName(routeName: string): string | undefined {
    return names.value[routeName]
  }

  function clearNames(): void {
    names.value = {}
  }

  function markVisited(routeName: string): void {
    recent.value = [routeName, ...recent.value.filter((r) => r !== routeName)].slice(0, 5)
  }

  return { names, recent, publishName, resolvedName, clearNames, markVisited }
})
