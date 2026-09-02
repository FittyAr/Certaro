<script setup lang="ts">
import Divider from 'primevue/divider'
import Drawer from 'primevue/drawer'
import { computed } from 'vue'

import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import type { useCrudDrawer } from '@/composables/useCrudDrawer'

/**
 * The side panel every create and edit uses. See `docs/16-frontend.md` §5.2.
 *
 * Escape is not bound here: the drawer registered itself in the cascade when it opened, so the
 * key closes whatever is actually on top of it.
 */

// eslint-disable-next-line @typescript-eslint/no-explicit-any -- the drawer is generic over its DTO
const props = defineProps<{ drawer: ReturnType<typeof useCrudDrawer<any>>; titleKey: string }>()

const drawer = computed(() => props.drawer)

const title = computed(() =>
  drawer.value.mode.value === 'create' ? 'General.NewOf' : 'General.EditOf',
)
</script>

<template>
  <Drawer
    :visible="drawer.open.value"
    position="right"
    :close-on-escape="false"
    :dismissable="false"
    :auto-focus="false"
    class="w-full md:!w-[480px]"
    @update:visible="(value: boolean) => !value && drawer.close()"
  >
    <template #header>
      <h3 class="text-base font-semibold">{{ $t(title, { entity: $t(props.titleKey) }) }}</h3>
    </template>

    <div
      v-if="drawer.staleConflict.value"
      class="mb-4 flex items-start gap-2 rounded-md border border-border bg-surface-raised p-3 text-sm"
    >
      <AppIcon name="triangle-alert" :size="16" />
      <div class="flex-1">
        <p>{{ $t('Error.Concurrency') }}</p>
        <Button variant="link" size="sm" class="px-0" @click="drawer.reloadCurrent()">
          {{ $t('General.ReloadRecord') }}
        </Button>
      </div>
    </div>

    <form class="space-y-4" @submit.prevent="drawer.save()">
      <slot />
    </form>

    <Divider class="my-4" />

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button variant="outline" :disabled="drawer.saving.value" @click="drawer.close()">
          {{ $t('General.Cancel') }}
        </Button>
        <Button :disabled="drawer.saving.value || drawer.loading.value" @click="drawer.save()">
          {{ $t('General.Save') }}
        </Button>
      </div>
    </template>
  </Drawer>
</template>
