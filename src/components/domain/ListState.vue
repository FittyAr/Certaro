<script setup lang="ts">
import type { ApiError } from '@/api/client'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError } from '@/composables/useApiError'

/**
 * The four states of a list. See `docs/16-frontend.md` §6.1. No list is allowed to go without one.
 *
 * Two kinds of empty, with different messages: nothing recorded at all invites creating the first
 * record, while nothing matching the filters offers to clear them.
 */
const props = withDefaults(
  defineProps<{
    loading: boolean
    firstLoad: boolean
    error: ApiError | null
    isEmpty: boolean
    isFiltered: boolean
    /** Message for the "nothing recorded yet" case, specific to the module. */
    emptyKey: string
  }>(),
  {},
)

const emit = defineEmits<{ retry: []; clearFilters: [] }>()

const { translate } = useApiError()
</script>

<template>
  <!-- First load shows the shape of the table; a spinner replacing a populated table on every
       keystroke of a filter is a flicker nobody wants. -->
  <div v-if="props.loading && props.firstLoad" class="space-y-2 p-4" role="status">
    <div v-for="row in 6" :key="row" class="h-8 animate-pulse rounded bg-muted" />
    <span class="sr-only">{{ $t('General.Loading') }}</span>
  </div>

  <div v-else-if="props.error" class="flex flex-col items-center gap-3 p-10 text-center">
    <AppIcon name="triangle-alert" :size="28" />
    <p class="text-sm">{{ translate(props.error) }}</p>
    <Button variant="outline" size="sm" @click="emit('retry')">{{ $t('General.Retry') }}</Button>
  </div>

  <div v-else-if="props.isEmpty" class="flex flex-col items-center gap-3 p-10 text-center">
    <AppIcon name="inbox" :size="28" />
    <p class="text-sm text-muted-foreground">
      {{ props.isFiltered ? $t('General.NoResults') : $t(props.emptyKey) }}
    </p>
    <Button v-if="props.isFiltered" variant="outline" size="sm" @click="emit('clearFilters')">
      {{ $t('General.ClearFilters') }}
    </Button>
    <slot v-else name="empty-action" />
  </div>

  <slot v-else />
</template>
