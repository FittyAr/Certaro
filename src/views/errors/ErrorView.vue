<script setup lang="ts">
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'

/**
 * What the error barrier of `docs/16-frontend.md` §6.4 renders. A render failure in one screen
 * shows this with a way out, rather than a blank window.
 */
defineProps<{ detail?: string }>()
const emit = defineEmits<{ retry: [] }>()
</script>

<template>
  <div class="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
    <AppIcon name="triangle-alert" :size="32" />
    <p class="text-sm">{{ $t('Error.Unexpected') }}</p>
    <!-- The detail is a trace identifier, never a path or a SQL fragment. -->
    <p v-if="detail" class="text-xs text-muted-foreground tabular-nums">{{ detail }}</p>
    <Button variant="outline" size="sm" @click="emit('retry')">{{ $t('General.Retry') }}</Button>
  </div>
</template>
