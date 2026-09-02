<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'

import { Button } from '@/components/ui/button'
import AppIcon from '@/components/ui/AppIcon.vue'
import { useAuthStore } from '@/stores/useAuthStore'
import { useUiStore } from '@/stores/useUiStore'

/** Fixed header of `docs/10-navegacion-y-atajos.md` §1. */

defineProps<{ overlay: boolean }>()

const ui = useUiStore()
const authStore = useAuthStore()
const route = useRoute()
const { t } = useI18n()

const title = computed(() => t(route.meta.titleKey ?? 'Menu.Dashboard'))
const themeIcon = computed(() =>
  ui.theme === 'dark' ? 'moon' : ui.theme === 'light' ? 'sun' : 'monitor',
)
</script>

<template>
  <header class="flex items-center gap-2 border-b border-border bg-surface-card px-3">
    <Button
      variant="ghost"
      size="icon"
      :aria-label="$t('Menu.Aria.Toggle')"
      :title="`${$t('Menu.Aria.Toggle')} (Ctrl+B)`"
      @click="ui.toggleSidebar()"
    >
      <AppIcon name="menu" />
    </Button>

    <h1 class="truncate text-base font-semibold">{{ title }}</h1>

    <button
      type="button"
      class="ml-auto flex w-64 items-center gap-2 rounded-md border border-border bg-background px-3 py-1.5 text-sm text-muted-foreground hover:bg-muted"
      @click="ui.openCommandPalette()"
    >
      <AppIcon name="search" :size="16" />
      <span class="flex-1 text-left">{{ $t('CommandPalette.SearchPlaceholder') }}</span>
      <span class="text-xs">Ctrl+K</span>
    </button>

    <Button
      variant="ghost"
      size="icon"
      :aria-label="$t('General.Theme')"
      :title="$t('General.Theme')"
      @click="ui.cycleTheme()"
    >
      <AppIcon :name="themeIcon" />
    </Button>

    <Button
      variant="ghost"
      size="icon"
      :aria-label="$t('General.PrivacyMode')"
      :title="`${$t('General.PrivacyMode')} (Ctrl+Shift+P)`"
      @click="ui.togglePrivacy()"
    >
      <AppIcon :name="ui.privacyMode ? 'eye-off' : 'eye'" />
    </Button>

    <div v-if="authStore.user" class="flex items-center gap-2 pl-2 border-l border-border">
      <div
        class="w-7 h-7 rounded-full bg-primary/10 text-primary flex items-center justify-center font-bold text-xs"
        :title="authStore.user.email"
      >
        {{ authStore.user.nombreCompleto.slice(0, 2).toUpperCase() }}
      </div>
      <span class="text-xs font-medium text-foreground max-w-[120px] truncate hidden md:inline">
        {{ authStore.user.nombreCompleto }}
      </span>
      <Button
        v-if="authStore.requiresLogin"
        variant="ghost"
        size="icon"
        class="text-muted-foreground hover:text-destructive"
        title="Cerrar sesión"
        @click="authStore.logout()"
      >
        <AppIcon name="LogOut" :size="16" />
      </Button>
    </div>
  </header>
</template>
