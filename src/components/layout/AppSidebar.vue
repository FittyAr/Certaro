<script setup lang="ts">
import { computed } from 'vue'

import Divider from 'primevue/divider'

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

function isGroupExpanded(group: (typeof MENU)[number]): boolean {
  return ui.isGroupExpanded(group.labelKey, group.defaultExpanded ?? true)
}

function toggleGroup(group: (typeof MENU)[number]): void {
  ui.toggleGroup(group.labelKey, group.defaultExpanded ?? true)
}
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
    <div v-for="(group, index) in groups" :key="group.labelKey" class="mb-2">
      <!-- Collapsed sidebar: simple divider -->
      <Divider v-if="!expanded" class="mx-3 my-2" />

      <!-- Expanded: collapsible header -->
      <button
        v-else-if="group.collapsible"
        class="flex w-full items-center gap-2 px-4 py-2 text-xs font-medium uppercase tracking-wide text-muted-foreground hover:text-foreground"
        :aria-expanded="isGroupExpanded(group)"
        :aria-label="$t(group.labelKey)"
        @click="toggleGroup(group)"
      >
        <span class="flex-1 text-left">{{ $t(group.labelKey) }}</span>
        <AppIcon
          name="chevron-down"
          :size="14"
          class="transition-transform duration-200"
          :class="isGroupExpanded(group) ? '' : '-rotate-90'"
        />
      </button>
      <p
        v-else
        class="px-4 py-2 text-xs font-medium uppercase tracking-wide text-muted-foreground"
      >
        {{ $t(group.labelKey) }}
      </p>

      <div
        v-show="!expanded || isGroupExpanded(group)"
        class="overflow-hidden transition-all"
      >
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
      <Divider v-if="expanded && index < groups.length - 1" class="my-2" />
    </div>
  </nav>
</template>
