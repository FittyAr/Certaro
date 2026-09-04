<script setup lang="ts">
import Select from 'primevue/select'
import { onMounted, ref } from 'vue'

import type { FormatoExport } from '@/stores/useReportesStore'
import DateInput from '@/components/domain/DateInput.vue'
import ExportMenu from '@/components/domain/ExportMenu.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { useCertificadosStore } from '@/stores/useCertificadosStore'
import { useLiquidacionesStore } from '@/stores/useLiquidacionesStore'
import type { MovimientoFiltro } from '@/stores/useMovimientosStore'
import { useReportesStore } from '@/stores/useReportesStore'

/**
 * The report centre. See `docs/09-modulos-funcionales.md` §3.12.
 *
 * Each card asks only for what its report needs and then opens the save dialog. Nothing is written
 * anywhere the user did not choose.
 */

const liquidaciones = useLiquidacionesStore()
const certificados = useCertificadosStore()
const reportes = useReportesStore()

/** Its own filter: the report centre is reached without passing through the ledger. */
const filtro = ref<MovimientoFiltro>({ concepto: '' })

const liquidacionId = ref<string | null>(null)
const certificadoId = ref<string | null>(null)

const opcionesLiquidacion = ref<{ label: string; value: string }[]>([])
const opcionesCertificado = ref<{ label: string; value: string }[]>([])

async function cargarSelectores(): Promise<void> {
  const [liq, cert] = await Promise.all([
    liquidaciones.fetchPaged({ page: 1, pageSize: 100, filtro: {}, sortDir: 'Desc' }),
    certificados.fetchPaged({ page: 1, pageSize: 100, filtro: {}, sortDir: 'Desc' }),
  ])
  opcionesLiquidacion.value = liq.items.map((l) => ({
    label: `${l.empleadoNombre} · ${l.fechaInicio} · ${l.fechaFin}`,
    value: l.id,
  }))
  opcionesCertificado.value = cert.items.map((c) => ({
    label: `${c.proyectoNombre} · Nº ${c.numero}`,
    value: c.id,
  }))
}

function nombreDe(opciones: { label: string; value: string }[], id: string | null): string {
  return opciones.find((o) => o.value === id)?.label ?? ''
}

const soloPdf: readonly FormatoExport[] = ['Pdf'] as const

onMounted(() => void cargarSelectores())
</script>

<template>
  <section class="flex h-full flex-col gap-4 overflow-auto p-6">
    <PageHeader :title="$t('Reportes.Title')" :subtitle="$t('Reportes.Subtitle')">
      <template #actions>
        <HelpButton topic-id="reportes-overview" title="Ayuda sobre Reportes y Exportación" />
      </template>
    </PageHeader>

    <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
      <article class="flex flex-col gap-3 rounded-md border border-border bg-surface-raised p-4">
        <header>
          <h2 class="font-medium">{{ $t('Reportes.Movimientos.Title') }}</h2>
          <p class="text-sm text-muted-foreground">
            {{ $t('Reportes.Movimientos.Descripcion') }}
          </p>
        </header>
        <div class="grid grid-cols-2 gap-3">
          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Desde') }}</span>
            <DateInput v-model="filtro.fechaDesde" />
          </label>
          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Hasta') }}</span>
            <DateInput v-model="filtro.fechaHasta" />
          </label>
        </div>
        <ExportMenu
          reporte="movimientos"
          class="self-start"
          :run="(formato, destino) => reportes.exportMovimientos(filtro, formato, destino)"
        />
      </article>

      <article class="flex flex-col gap-3 rounded-md border border-border bg-surface-raised p-4">
        <header>
          <h2 class="font-medium">{{ $t('Reportes.Liquidacion.Title') }}</h2>
          <p class="text-sm text-muted-foreground">
            {{ $t('Reportes.Liquidacion.Descripcion') }}
          </p>
        </header>
        <Select
          v-model="liquidacionId"
          :options="opcionesLiquidacion"
          option-label="label"
          option-value="value"
          filter
          :placeholder="$t('Reportes.Elegir')"
        />
        <ExportMenu
          reporte="liquidacion"
          class="self-start"
          :formatos="soloPdf"
          :detalle="nombreDe(opcionesLiquidacion, liquidacionId)"
          :disabled="!liquidacionId"
          :run="(_formato, destino) => reportes.exportLiquidacion(liquidacionId!, destino)"
        />
      </article>

      <article class="flex flex-col gap-3 rounded-md border border-border bg-surface-raised p-4">
        <header>
          <h2 class="font-medium">{{ $t('Reportes.Certificado.Title') }}</h2>
          <p class="text-sm text-muted-foreground">
            {{ $t('Reportes.Certificado.Descripcion') }}
          </p>
        </header>
        <Select
          v-model="certificadoId"
          :options="opcionesCertificado"
          option-label="label"
          option-value="value"
          filter
          :placeholder="$t('Reportes.Elegir')"
        />
        <ExportMenu
          reporte="certificado"
          class="self-start"
          :formatos="soloPdf"
          :detalle="nombreDe(opcionesCertificado, certificadoId)"
          :disabled="!certificadoId"
          :run="(_formato, destino) => reportes.exportCertificado(certificadoId!, destino)"
        />
      </article>
    </div>
  </section>
</template>
