<script setup lang="ts">
import type { ApiError } from '@/api/client'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError } from '@/composables/useApiError'

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

defineOptions({ inheritAttrs: false })

const { translate } = useApiError()
</script>

<template>
  <div :class="['w-full', ($attrs.class as string) || '']">
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
  </div>
</template>
