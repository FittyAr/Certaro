<script setup lang="ts">
import { computed } from 'vue'

import AppIcon from '@/components/ui/AppIcon.vue'
import { useBreadcrumb } from '@/composables/useBreadcrumb'
import { activeMenuRoute, MENU, numericShortcutRoutes } from '@/router/menu'
import { useConfigStore } from '@/stores/useConfigStore'
import { useUiStore } from '@/stores/useUiStore'

/** Navigation of `docs/10-navegacion-y-atajos.md` §3, derived entirely from `MENU`. */

const props = defineProps<{ overlay: boolean }>()

const ui = useUiStore()
const config = useConfigStore()
const { routeChain } = useBreadcrumb()

const seedEnabled = computed(() => config.config?.application.seedEnabled ?? false)

const groups = computed(() =>
  MENU.map((group) => ({
    ...group,
    items: group.items.filter((item) => !item.devOnly || seedEnabled.value),
  })).filter((group) => group.items.length > 0),
)

const active = computed(() => activeMenuRoute(routeChain.value))

/** `Ctrl+1` … `Ctrl+9`, shown as a hint next to the first nine entries. */
const shortcutOf = computed(() => {
  const map = new Map<string, string>()
  numericShortcutRoutes().forEach((name, index) => map.set(name, `Ctrl+${index + 1}`))
  return map
})

const expanded = computed(() => ui.sidebarExpanded)
const hidden = computed(() => props.overlay && !ui.sidebarExpanded)
</script>

<template>
  <nav
    v-show="!hidden"
    class="overflow-y-auto border-r border-border bg-surface-raised py-2"
    :class="[
      expanded ? 'w-[260px]' : 'w-[56px]',
      props.overlay ? 'absolute inset-y-0 left-0 z-30 shadow-lg' : '',
    ]"
    :aria-label="$t('Menu.Aria.Main')"
  >
    <div v-for="group in groups" :key="group.labelKey" class="mb-2">
      <p
        v-if="expanded"
        class="px-4 py-2 text-xs font-medium uppercase tracking-wide text-muted-foreground"
      >
        {{ $t(group.labelKey) }}
      </p>
      <hr v-else class="mx-3 my-2 border-border" />

      <RouterLink
        v-for="item in group.items"
        :key="item.route"
        :to="{ name: item.route }"
        class="mx-2 flex items-center gap-3 rounded-md px-3 py-2 text-sm text-foreground hover:bg-muted"
        :class="active === item.route ? 'bg-muted font-medium' : ''"
        :aria-current="active === item.route ? 'page' : undefined"
        :aria-label="$t(item.labelKey)"
        :title="expanded ? undefined : `${$t(item.labelKey)} ${shortcutOf.get(item.route) ?? ''}`"
      >
        <AppIcon :name="item.icon" />
        <template v-if="expanded">
          <span class="flex-1 truncate">{{ $t(item.labelKey) }}</span>
          <span v-if="shortcutOf.get(item.route)" class="text-xs text-muted-foreground">
            {{ shortcutOf.get(item.route) }}
          </span>
        </template>
      </RouterLink>
    </div>
  </nav>
</template>
