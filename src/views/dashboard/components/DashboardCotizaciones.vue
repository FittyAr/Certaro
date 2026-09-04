<script setup lang="ts">
import { computed } from 'vue'
import DateText from '@/components/domain/DateText.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { useDashboardStore } from '@/stores/useDashboardStore'

const store = useDashboardStore()
const cotizaciones = computed(() => store.cotizaciones)
</script>

<template>
  <div v-if="cotizaciones?.length" class="flex flex-col gap-1.5">
    <div class="flex items-center justify-between">
      <span class="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
        Mercado Cambiario
      </span>
      <HelpButton topic-id="dashboard-cotizaciones" title="Ayuda sobre cotizaciones de dólar" />
    </div>
    <div class="flex flex-wrap gap-3">
      <article
        v-for="cotizacion in cotizaciones"
        :key="cotizacion.casa"
        class="rounded-md border border-border bg-surface-card px-3 py-2 text-sm"
      >
        <p class="text-xs text-muted-foreground">{{ cotizacion.nombre }}</p>
        <p class="flex items-center gap-3">
          <span>
            {{ $t('Cotizaciones.Compra') }} <MoneyText :value="cotizacion.compra" />
          </span>
          <span>
            {{ $t('Cotizaciones.Venta') }} <MoneyText :value="cotizacion.venta" />
          </span>
        </p>
        <p v-if="cotizacion.desactualizada" class="text-xs text-muted-foreground">
          {{ $t('Cotizaciones.Desactualizada') }}
          <DateText :value="cotizacion.fechaActualizacion" />
        </p>
      </article>
    </div>
  </div>
</template>
