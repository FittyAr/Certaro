<script setup lang="ts">
import type { AsistenciaGrilla, TipoJornada } from '@/stores/useAsistenciaStore'

defineProps<{
  grilla: AsistenciaGrilla
  clases: Record<TipoJornada, string>
  abreviaturas: Record<TipoJornada, string>
}>()

const emit = defineEmits<{
  (e: 'abrir-rango', empleadoId: string): void
  (e: 'ciclar', empleadoId: string, fecha: string): void
}>()
</script>

<template>
  <div class="overflow-auto">
    <table class="w-full border-collapse text-xs">
      <thead>
        <tr>
          <th
            class="sticky left-0 z-10 bg-background p-2 text-left font-medium"
            :style="{ minWidth: '12rem' }"
          >
            {{ $t('Empleados.Nombre') }}
          </th>
          <th
            v-for="dia in grilla.dias"
            :key="dia.fecha"
            class="p-1 text-center font-medium"
            :class="{
              'text-muted-foreground': dia.esFinDeSemana,
              'text-accent': dia.esFeriado,
            }"
            :title="dia.feriadoNombre ?? undefined"
          >
            {{ dia.fecha.slice(8) }}
          </th>
          <th class="p-1 text-center font-medium text-muted-foreground" title="Jornadas Completas">C</th>
          <th class="p-1 text-center font-medium text-muted-foreground" title="Medias Jornadas">½</th>
          <th class="p-1 text-center font-medium text-muted-foreground" title="Faltas">F</th>
          <th class="p-2 text-right font-medium">{{ $t('Asistencia.Jornadas') }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="fila in grilla.filas" :key="fila.empleadoId" class="border-t border-border">
          <th class="sticky left-0 z-10 bg-background p-2 text-left font-normal">
            <button
              type="button"
              class="underline-offset-2 hover:underline"
              :title="$t('Asistencia.CargaMasiva')"
              @click="emit('abrir-rango', fila.empleadoId)"
            >
              {{ fila.empleadoNombre }}
            </button>
          </th>
          <td v-for="(celda, i) in fila.celdas" :key="celda.fecha" class="p-0.5 text-center">
            <button
              type="button"
              class="inline-flex h-7 w-7 items-center justify-center rounded border border-border transition-colors hover:border-primary"
              :class="celda.tipoJornada ? clases[celda.tipoJornada] : 'text-transparent'"
              :aria-label="`${fila.empleadoNombre} ${celda.fecha}`"
              :title="
                celda.tipoJornada
                  ? $t(`TipoJornada.${celda.tipoJornada}`)
                  : $t('Asistencia.SinRegistro')
              "
              :disabled="grilla.dias[i] === undefined"
              @click="emit('ciclar', fila.empleadoId, celda.fecha)"
            >
              {{ celda.tipoJornada ? abreviaturas[celda.tipoJornada] : '·' }}
            </button>
          </td>
          <td class="p-1 text-center tabular-nums text-muted-foreground">
            {{ fila.resumen.completas }}
          </td>
          <td class="p-1 text-center tabular-nums text-muted-foreground">
            {{ fila.resumen.medias }}
          </td>
          <td class="p-1 text-center tabular-nums text-muted-foreground">
            {{ fila.resumen.faltas + fila.resumen.faltasJustificadas }}
          </td>
          <td class="p-2 text-right font-semibold tabular-nums text-foreground">
            {{ fila.resumen.jornadasEquivalentes }}
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
