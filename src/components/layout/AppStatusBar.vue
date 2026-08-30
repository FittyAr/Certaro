<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import MoneyText from '@/components/domain/MoneyText.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { useBreadcrumb } from '@/composables/useBreadcrumb'
import { useConfigStore } from '@/stores/useConfigStore'
import { useDashboardStore } from '@/stores/useDashboardStore'
import { useUiStore } from '@/stores/useUiStore'

/** Status bar of `docs/10-navegacion-y-atajos.md` §1 and §6. */

const router = useRouter()
const ui = useUiStore()
const config = useConfigStore()
const { crumbs } = useBreadcrumb()

/**
 * The history is the browser's, but "can I go back" is not exposed by it, so entering a second
 * route within the application is what enables the button. The legacy system kept its own stack
 * and did not push on `GoBack`, so the history was consumed and forward never worked.
 */
const navigations = ref(0)
router.afterEach(() => {
  navigations.value += 1
})
const canGoBack = computed(() => navigations.value > 1)

/** Over four levels the middle ones collapse, so the bar never wraps. */
const shown = computed(() => {
  const all = crumbs.value
  if (all.length <= 4) return all
  return [all[0]!, { label: '…' }, ...all.slice(-2)]
})

const stateKey = computed(() => `Status.${ui.bootstrapState}`)

/**
 * The default house's selling rate, which is the number the movement form pre-loads (doc 13 §2.5).
 * Absent rather than zero when the service is unreachable: the bar simply does not show it.
 */
const dashboard = useDashboardStore()
const cotizacion = computed(() => {
  const preferida = config.config?.dashboard.cotizacionPorDefecto
  const casas = dashboard.cotizaciones
  return casas.find((c) => c.casa === preferida) ?? casas[0] ?? null
})
</script>

<template>
  <footer
    class="flex items-center gap-3 border-t border-border bg-surface-raised px-3 text-xs text-muted-foreground"
  >
    <button
      type="button"
      class="flex items-center gap-1 rounded px-1 hover:bg-muted disabled:opacity-40"
      :disabled="!canGoBack"
      :aria-label="$t('General.Back')"
      @click="router.back()"
    >
      <AppIcon name="arrow-left" :size="14" />
      <span>{{ $t('General.Back') }}</span>
    </button>

    <span>{{ $t(stateKey) }}</span>

    <nav class="ml-auto flex items-center gap-1 truncate" :aria-label="$t('Menu.Aria.Breadcrumb')">
      <template v-for="(crumb, index) in shown" :key="index">
        <span v-if="index > 0" aria-hidden="true">›</span>
        <RouterLink v-if="crumb.to" :to="crumb.to" class="truncate hover:underline">
          {{ crumb.label }}
        </RouterLink>
        <span v-else class="truncate text-foreground">{{ crumb.label }}</span>
      </template>
    </nav>

    <span v-if="cotizacion" class="flex items-center gap-1" :title="$t('Cotizaciones.Venta')">
      <AppIcon name="dollar-sign" :size="12" />
      <span>{{ cotizacion.nombre }}</span>
      <MoneyText :value="cotizacion.venta" />
      <AppIcon v-if="cotizacion.desactualizada" name="clock" :size="12" />
    </span>

    <span v-if="config.info">{{ config.info.version }}</span>
  </footer>
</template>
