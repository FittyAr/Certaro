<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import MoneyText from '@/components/domain/MoneyText.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { useDashboardStore } from '@/stores/useDashboardStore'

const router = useRouter()
const store = useDashboardStore()
const alertas = computed(() => store.alertas)

const severidadClase: Record<string, string> = {
  Info: 'border-l-primary',
  Warning: 'border-l-warning',
  Error: 'border-l-destructive',
}

function irA(destino: string): void {
  void router.push(destino)
}
</script>

<template>
  <div v-if="alertas?.length" class="flex flex-col gap-1.5">
    <div class="flex items-center justify-between">
      <span class="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
        Alertas Operativas
      </span>
      <HelpButton topic-id="dashboard-alertas" title="Ayuda sobre alertas automáticas" />
    </div>
    <div class="grid gap-2 md:grid-cols-2 xl:grid-cols-3">
      <button
        v-for="alerta in alertas"
        :key="alerta.tipo"
        type="button"
        class="flex items-center gap-2 rounded-md border border-border border-l-4 bg-surface-card px-3 py-2 text-left text-sm hover:bg-muted"
        :class="severidadClase[alerta.severidad]"
        @click="irA(alerta.destino)"
      >
        <AppIcon name="triangle-alert" :size="16" />
        <span class="flex-1">
          {{ $t(alerta.clave, { cantidad: alerta.cantidad }) }}
          <MoneyText v-if="alerta.monto" :value="alerta.monto" colored />
        </span>
        <AppIcon name="chevron-right" :size="16" />
      </button>
    </div>
  </div>
</template>
