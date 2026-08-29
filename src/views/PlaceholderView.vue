<script setup lang="ts">
import PageHeader from '@/components/domain/PageHeader.vue'

/**
 * Scaffolding for a screen whose module has not been implemented yet.
 *
 * It exists so the shell — navigation, breadcrumbs, shortcuts, the command palette — can be built
 * and tested against the real 15 routes instead of against a stub router. Each module replaces its
 * own placeholder as it lands.
 */
defineProps<{ titleKey: string }>()
</script>

<template>
  <section class="space-y-6 p-6">
    <PageHeader :title="$t(titleKey)" :subtitle="$t('General.WorkInProgress')" />
  </section>
</template>
