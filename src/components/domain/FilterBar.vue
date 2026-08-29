<script setup lang="ts">
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useEscapeLayer } from '@/composables/useEscapeStack'
import { watch } from 'vue'

/**
 * Filter container with a single clear action. The bar registers itself in the Escape cascade
 * while it holds a value, which is step 5 of `docs/10-navegacion-y-atajos.md` §4.3.
 */
const props = defineProps<{ active: boolean }>()
const emit = defineEmits<{ clear: [] }>()

const layer = useEscapeLayer('filters', () => {
  if (!props.active) return false
  emit('clear')
  return true
})

watch(
  () => props.active,
  (active) => (active ? layer.push() : layer.pop()),
  { immediate: true },
)
</script>

<template>
  <div class="flex flex-wrap items-end gap-3 rounded-md border border-border bg-surface-raised p-3">
    <slot />
    <Button v-if="props.active" variant="ghost" size="sm" class="ml-auto" @click="emit('clear')">
      <AppIcon name="x" :size="14" />
      {{ $t('General.ClearFilters') }}
    </Button>
  </div>
</template>
