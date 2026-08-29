<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import AppIcon from '@/components/ui/AppIcon.vue'
import { useBreadcrumb } from '@/composables/useBreadcrumb'
import { useConfigStore } from '@/stores/useConfigStore'
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

    <span v-if="config.info">{{ config.info.version }}</span>
  </footer>
</template>
