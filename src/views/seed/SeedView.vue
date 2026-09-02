<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useToast } from 'primevue/usetoast'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useSistemaStore, type SeedSummary } from '@/stores/useSistemaStore'

const { t } = useI18n()
const toast = useToast()
const router = useRouter()
const sistema = useSistemaStore()

const loading = ref(false)
const summary = ref<SeedSummary | null>(null)

async function ejecutarSembrado(): Promise<void> {
  loading.value = true
  try {
    const res = await sistema.seedDemoData()
    summary.value = res
    toast.add({
      severity: 'success',
      summary: t('Seed.Success'),
      life: 4000,
    })
  } catch (err: unknown) {
    toast.add({
      severity: 'error',
      summary: t('General.Error'),
      detail: String(err),
      life: 5000,
    })
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <section class="flex h-full flex-col gap-6 overflow-auto p-6">
    <PageHeader :title="$t('Seed.Title')" :subtitle="$t('Seed.Subtitle')" />

    <div class="flex max-w-4xl flex-col gap-6">
      <!-- Tarjeta informativa y disparador -->
      <div class="rounded-lg border border-border bg-surface-card p-6 shadow-sm">
        <div class="mb-4 flex items-center gap-2 border-b border-border pb-3">
          <AppIcon name="database" :size="20" class="text-primary" />
          <h3 class="text-sm font-semibold text-foreground">{{ $t('Seed.WarningTitle') }}</h3>
        </div>

        <p class="mb-6 text-sm text-muted-foreground leading-relaxed">
          {{ $t('Seed.WarningText') }}
        </p>

        <div class="flex items-center gap-3">
          <Button
            :disabled="loading"
            class="flex items-center gap-2 px-6"
            @click="ejecutarSembrado"
          >
            <AppIcon v-if="!loading" name="play" :size="16" />
            <AppIcon v-else name="loader" :size="16" class="animate-spin" />
            <span>{{ loading ? $t('Seed.Loading') : $t('Seed.Button') }}</span>
          </Button>
        </div>
      </div>

      <!-- Resultados del sembrado -->
      <div v-if="summary" class="rounded-lg border border-border bg-surface-card p-6 shadow-sm animate-fade-in">
        <div class="mb-4 flex items-center gap-2 border-b border-border pb-3">
          <AppIcon name="check-circle" :size="20" class="text-success" />
          <h3 class="text-sm font-semibold text-foreground">{{ $t('Seed.SummaryTitle') }}</h3>
        </div>

        <dl class="grid grid-cols-2 gap-4 text-sm sm:grid-cols-3 md:grid-cols-5">
          <div class="flex flex-col gap-1 rounded-lg border border-border bg-surface-raised p-3">
            <dt class="text-xs text-muted-foreground">{{ $t('Seed.Categorias') }}</dt>
            <dd class="text-lg font-bold text-foreground tabular-nums">{{ summary.categorias }}</dd>
          </div>

          <div class="flex flex-col gap-1 rounded-lg border border-border bg-surface-raised p-3">
            <dt class="text-xs text-muted-foreground">{{ $t('Seed.TiposMovimiento') }}</dt>
            <dd class="text-lg font-bold text-foreground tabular-nums">{{ summary.tiposMovimiento }}</dd>
          </div>

          <div class="flex flex-col gap-1 rounded-lg border border-border bg-surface-raised p-3">
            <dt class="text-xs text-muted-foreground">{{ $t('Seed.Empleados') }}</dt>
            <dd class="text-lg font-bold text-foreground tabular-nums">{{ summary.empleados }}</dd>
          </div>

          <div class="flex flex-col gap-1 rounded-lg border border-border bg-surface-raised p-3">
            <dt class="text-xs text-muted-foreground">{{ $t('Seed.Clientes') }}</dt>
            <dd class="text-lg font-bold text-foreground tabular-nums">{{ summary.clientes }}</dd>
          </div>

          <div class="flex flex-col gap-1 rounded-lg border border-border bg-surface-raised p-3">
            <dt class="text-xs text-muted-foreground">{{ $t('Seed.Proyectos') }}</dt>
            <dd class="text-lg font-bold text-foreground tabular-nums">{{ summary.proyectos }}</dd>
          </div>

          <div class="flex flex-col gap-1 rounded-lg border border-border bg-surface-raised p-3">
            <dt class="text-xs text-muted-foreground">{{ $t('Seed.Trabajos') }}</dt>
            <dd class="text-lg font-bold text-foreground tabular-nums">{{ summary.trabajos }}</dd>
          </div>

          <div class="flex flex-col gap-1 rounded-lg border border-border bg-surface-raised p-3">
            <dt class="text-xs text-muted-foreground">{{ $t('Seed.OrdenesTrabajo') }}</dt>
            <dd class="text-lg font-bold text-foreground tabular-nums">{{ summary.ordenesTrabajo }}</dd>
          </div>

          <div class="flex flex-col gap-1 rounded-lg border border-border bg-surface-raised p-3">
            <dt class="text-xs text-muted-foreground">{{ $t('Seed.Movimientos') }}</dt>
            <dd class="text-lg font-bold text-foreground tabular-nums">{{ summary.movimientos }}</dd>
          </div>

          <div class="flex flex-col gap-1 rounded-lg border border-border bg-surface-raised p-3">
            <dt class="text-xs text-muted-foreground">{{ $t('Seed.Facturas') }}</dt>
            <dd class="text-lg font-bold text-foreground tabular-nums">{{ summary.facturas }}</dd>
          </div>

          <div class="flex flex-col gap-1 rounded-lg border border-border bg-surface-raised p-3">
            <dt class="text-xs text-muted-foreground">{{ $t('Seed.Liquidaciones') }}</dt>
            <dd class="text-lg font-bold text-foreground tabular-nums">{{ summary.liquidaciones }}</dd>
          </div>
        </dl>

        <div class="mt-6 flex flex-wrap gap-3 border-t border-border pt-4">
          <Button variant="outline" class="flex items-center gap-2" @click="router.push({ name: 'dashboard' })">
            <AppIcon name="layout-dashboard" :size="16" />
            <span>{{ $t('Seed.GoToDashboard') }}</span>
          </Button>
          <Button variant="outline" class="flex items-center gap-2" @click="router.push({ name: 'movimientos' })">
            <AppIcon name="receipt" :size="16" />
            <span>{{ $t('Seed.GoToMovimientos') }}</span>
          </Button>
        </div>
      </div>
    </div>
  </section>
</template>
