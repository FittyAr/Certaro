import { computed, ref } from 'vue'
import { useToast } from 'primevue/usetoast'
import { useI18n } from 'vue-i18n'

import type { Cambios } from '@/api/sistema'
import { useSistemaStore } from '@/stores/useSistemaStore'

/**
 * Config form helper. See `docs/09` §3.15.
 *
 * Each settings section clones the relevant slice of AppConfig into a local reactive object,
 * tracks whether it differs from the snapshot, and applies only the changed keys.
 */

export function useConfigForm<T extends Record<string, unknown>>(section: () => T | null) {
  const sistema = useSistemaStore()
  const toast = useToast()
  const { t } = useI18n()

  const draft = ref<T | null>(null)
  const saving = ref(false)

  /** Snapshots the current config section into the draft. */
  function load(): void {
    const current = section()
    if (current) {
      draft.value = structuredClone(current) as T
    }
  }

  /** Keys whose value differs from the snapshot. */
  const cambios = computed<Cambios>(() => {
    if (!draft.value) return {}
    const current = section()
    if (!current) return {}
    const diff: Cambios = {}
    for (const [key, value] of Object.entries(draft.value)) {
      const original = current[key]
      if (JSON.stringify(value) !== JSON.stringify(original)) {
        diff[key] = typeof value === 'string' ? value : JSON.stringify(value)
      }
    }
    return diff
  })

  const isDirty = computed(() => Object.keys(cambios.value).length > 0)

  async function apply(): Promise<void> {
    if (!isDirty.value) return
    saving.value = true
    try {
      await sistema.applyConfig(cambios.value)
      toast.add({
        severity: 'success',
        summary: t('Configuracion.Applied'),
        life: 3000,
      })
      // Reload the draft from the new snapshot.
      load()
    } finally {
      saving.value = false
    }
  }

  async function resetKey(key: string): Promise<void> {
    saving.value = true
    try {
      await sistema.resetConfig([key])
      load()
    } finally {
      saving.value = false
    }
  }

  return { draft, saving, isDirty, apply, resetKey, load }
}
