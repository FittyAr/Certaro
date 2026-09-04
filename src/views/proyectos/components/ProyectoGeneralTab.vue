<script setup lang="ts">
import MoneyText from '@/components/domain/MoneyText.vue'
import StatePill from '@/components/domain/StatePill.vue'
import type { ProyectoDetalle } from '@/stores/useProyectosStore'

defineProps<{
  proyecto: ProyectoDetalle
  trabajosCount: number
  totalIngresos: string
  balanceNeto: string
}>()
</script>

<template>
  <div class="space-y-4">
    <div class="grid gap-4 rounded-lg border border-border bg-surface-card p-5 text-sm md:grid-cols-2">
      <div>
        <span class="text-xs text-muted-foreground">{{ $t('Proyectos.Numero') }}</span>
        <p class="font-semibold text-foreground">#{{ proyecto.numero }}</p>
      </div>
      <div>
        <span class="text-xs text-muted-foreground">{{ $t('Proyectos.Estado') }}</span>
        <p class="mt-0.5"><StatePill entity="Proyecto" :value="proyecto.estado.actual" /></p>
      </div>
      <div>
        <span class="text-xs text-muted-foreground">{{ $t('Clientes.Nombre') }}</span>
        <p class="font-medium text-foreground">{{ proyecto.clienteNombre }}</p>
      </div>
      <div>
        <span class="text-xs text-muted-foreground">{{ $t('Proyectos.Localidad') }}</span>
        <p>{{ proyecto.localidad ?? '—' }}</p>
      </div>
      <div class="md:col-span-2">
        <span class="text-xs text-muted-foreground">{{ $t('Clientes.Direccion') }}</span>
        <p>{{ proyecto.direccion ?? '—' }}</p>
      </div>
    </div>

    <!-- Resumen rápido de métricas -->
    <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
      <div class="rounded-lg border border-border bg-card p-4 shadow-xs">
        <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          {{ $t('Menu.Trabajos') }}
        </span>
        <div class="mt-1 text-2xl font-bold">{{ trabajosCount }}</div>
      </div>
      <div class="rounded-lg border border-border bg-card p-4 shadow-xs">
        <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          {{ $t('Movimientos.Ingresos') }}
        </span>
        <div class="mt-1 text-2xl font-bold">
          <MoneyText :value="totalIngresos" colored />
        </div>
      </div>
      <div class="rounded-lg border border-border bg-card p-4 shadow-xs">
        <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          {{ $t('Movimientos.Balance') }}
        </span>
        <div class="mt-1 text-2xl font-bold">
          <MoneyText :value="balanceNeto" colored show-sign />
        </div>
      </div>
    </div>
  </div>
</template>
