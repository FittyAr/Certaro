<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<template>
  <main class="flex h-full flex-col items-center justify-center gap-2 p-6 text-center">
    <p class="text-2xl font-semibold tracking-tight">404</p>
    <p class="text-sm text-muted-foreground">{{ t('General.Empty') }}</p>
    <RouterLink
      to="/"
      class="mt-2 rounded-md border border-border px-3 py-2 text-sm font-medium hover:bg-surface-raised"
    >
      {{ t('General.Close') }}
    </RouterLink>
  </main>
</template>
