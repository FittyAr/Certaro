import { computed } from 'vue'
import { useAuthStore } from '@/stores/useAuthStore'

export function usePermission() {
  const authStore = useAuthStore()

  const isSqliteMode = computed(() => authStore.isSqliteMode)
  const user = computed(() => authStore.user)
  const roles = computed(() => authStore.roles)

  function can(permiso: string): boolean {
    return authStore.hasPermission(permiso)
  }

  function hasRole(rolNombre: string): boolean {
    return authStore.hasRole(rolNombre)
  }

  return {
    isSqliteMode,
    user,
    roles,
    can,
    hasRole,
  }
}
