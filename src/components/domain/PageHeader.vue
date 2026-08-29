<script setup lang="ts">
/** Title, subtitle and an action slot. Every screen opens with one. */
withDefaults(defineProps<{ title: string; subtitle?: string }>(), { subtitle: undefined })
</script>

<template>
  <div class="flex items-start justify-between gap-4">
    <div>
      <h2 class="text-2xl font-semibold tracking-tight">{{ title }}</h2>
      <p v-if="subtitle" class="text-sm text-muted-foreground">{{ subtitle }}</p>
    </div>
    <div class="flex items-center gap-2">
      <slot name="actions" />
    </div>
  </div>
</template>
