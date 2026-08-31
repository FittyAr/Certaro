<script setup lang="ts">
import AppIcon from '@/components/ui/AppIcon.vue'
</script>

<template>
  <div class="flex h-screen w-screen flex-col items-center justify-center bg-background text-foreground">
    <div class="flex flex-col items-center gap-6 text-center">
      <div class="flex h-16 w-16 items-center justify-center rounded-2xl bg-primary/10 text-primary">
        <AppIcon name="sparkles" :size="36" />
      </div>

      <div class="space-y-1">
        <h1 class="text-2xl font-bold tracking-tight">ElectroObra</h1>
        <p class="text-xs text-muted-foreground">{{ $t('Welcome.Subtitle') }}</p>
      </div>

      <div class="mt-4 flex items-center gap-3 rounded-full border border-border bg-surface-raised px-4 py-2 text-xs text-muted-foreground shadow-sm">
        <span class="inline-block h-2 w-2 animate-ping rounded-full bg-primary" />
        <span>{{ $t('App.InitializingHint') }}...</span>
      </div>
    </div>
  </div>
</template>
