<script setup lang="ts">
import { useMediaQuery } from '@vueuse/core'
import { computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import AppHeader from '@/components/layout/AppHeader.vue'
import AppSidebar from '@/components/layout/AppSidebar.vue'
import AppStatusBar from '@/components/layout/AppStatusBar.vue'
import CommandPalette from '@/components/layout/CommandPalette.vue'
import { handleEscape } from '@/composables/useEscapeStack'
import { useShortcuts } from '@/composables/useShortcuts'
import { useVersionCheck } from '@/composables/useVersionCheck'
import { numericShortcutRoutes } from '@/router/menu'
import { useNavigationStore } from '@/stores/useNavigationStore'
import { useUiStore } from '@/stores/useUiStore'

/**
 * The application frame of `docs/10-navegacion-y-atajos.md` §1.
 *
 * The content area is the only part that scrolls; header, sidebar and status bar stay put.
 */

const ui = useUiStore()
const navigation = useNavigationStore()
const router = useRouter()
const route = useRoute()
const versionCheck = useVersionCheck()

/** The single breakpoint of §7: below it the sidebar floats over the content. */
const isNarrow = useMediaQuery('(max-width: 767px)')
const overlayMode = computed(() => isNarrow.value)

watch(
  () => route.name,
  (name) => {
    if (name) navigation.markVisited(String(name))
    // On a narrow window the sidebar covers the content, so navigating has to close it.
    if (overlayMode.value) ui.sidebarExpanded = false
  },
)

const globalShortcuts = {
  'ctrl+k': { handler: () => ui.openCommandPalette(), allowInInput: true },
  'ctrl+p': { handler: () => ui.openCommandPalette(), allowInInput: true },
  'ctrl+b': () => ui.toggleSidebar(),
  'ctrl+shift+p': () => ui.togglePrivacy(),
  'ctrl+,': () => void router.push({ name: 'configuracion' }),
  'alt+arrowleft': () => router.back(),
  'alt+arrowright': () => router.forward(),
  f1: () => ui.openShortcutHelp(),
  escape: { handler: () => void handleEscape(), allowInInput: true },
  ...Object.fromEntries(
    numericShortcutRoutes().map((name, index) => [
      `ctrl+${index + 1}`,
      () => void router.push({ name }),
    ]),
  ),
}

useShortcuts(globalShortcuts)
</script>

<template>
  <div class="grid h-full grid-rows-[56px_1fr_28px] bg-background text-foreground">
    <AppHeader :overlay="overlayMode" />

    <div
      class="relative grid min-h-0"
      :class="overlayMode ? 'grid-cols-1' : 'grid-cols-[auto_1fr]'"
    >
      <AppSidebar :overlay="overlayMode" />

      <main class="min-w-0 overflow-y-auto">
        <div
          v-if="versionCheck.available.value"
          class="flex items-center justify-between bg-primary/10 px-4 py-2 text-sm"
        >
          <span>{{ $t('Update.Available', { version: versionCheck.available.value.version }) }}</span>
          <a
            :href="versionCheck.available.value.url"
            target="_blank"
            class="font-medium text-primary underline"
          >
            {{ $t('Update.Download') }}
          </a>
        </div>
        <RouterView />
      </main>
    </div>

    <AppStatusBar />
    <CommandPalette />
  </div>
</template>
