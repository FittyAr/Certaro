<script setup lang="ts">
import Column from 'primevue/column'
import Select from 'primevue/select'
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import DataGrid from '@/components/domain/DataGrid.vue'
import DateInput from '@/components/domain/DateInput.vue'
import DateText from '@/components/domain/DateText.vue'
import Divider from 'primevue/divider'
import FilterBar from '@/components/domain/FilterBar.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useApiError } from '@/composables/useApiError'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useServerTable } from '@/composables/useServerTable'
import type { LookupItem } from '@/stores/useCatalogStore'
import {
  useCertificadosStore,
  type CertificadoFiltro,
  type CertificadoListItem,
} from '@/stores/useCertificadosStore'
import { useClientesStore } from '@/stores/useClientesStore'
import { useProyectosStore } from '@/stores/useProyectosStore'
import { useTrabajosStore } from '@/stores/useTrabajosStore'

/**
 * The certification history. See `docs/09-modulos-funcionales.md` §3.7.
 *
 * New certificates are not created here: they are issued from the work order they belong to, which
 * is the only place that knows what is left to certify on each line.
 */

const router = useRouter()
const { notify } = useApiError()
const { confirmDelete } = useConfirmDelete()
const store = useCertificadosStore()
const clientes = useClientesStore()
const proyectos = useProyectosStore()
const trabajos = useTrabajosStore()

const table = useServerTable<CertificadoFiltro, CertificadoListItem>({
  key: 'certificados',
  initialFilter: {},
  fetch: (query) => store.fetchPaged(query),
  defaultSort: { field: 'fecha', dir: 'Desc' },
})

const opcionesCliente = ref<LookupItem[]>([])
const opcionesProyecto = ref<LookupItem[]>([])
const opcionesTrabajo = ref<LookupItem[]>([])

/** Narrowing by customer narrows the site list, and by site the job list: the usual way in. */
watch(
  () => table.filter.value.clienteId,
  async (clienteId) => {
    try {
      opcionesProyecto.value = await proyectos.lookup(clienteId, undefined, 200)
    } catch (e) {
      notify(e)
    }
  },
)

watch(
  () => table.filter.value.proyectoId,
  async (proyectoId) => {
    try {
      opcionesTrabajo.value = await trabajos.lookup(proyectoId, undefined, 200)
    } catch (e) {
      notify(e)
    }
  },
)

const filtrosActivos = computed(() =>
  Boolean(
    table.filter.value.clienteId ||
    table.filter.value.proyectoId ||
    table.filter.value.trabajoId ||
    table.filter.value.fechaDesde ||
    table.filter.value.fechaHasta,
  ),
)

function abrirDetalle(row: CertificadoListItem): void {
  void router.push({ name: 'certificado-detalle', params: { certificadoId: row.id } })
}

/** Voiding is a delete of the last certificate of its order, and it gives the progress back. */
function onAnular(row: CertificadoListItem): void {
  confirmDelete({
    entityKey: 'Entity.Certificado',
    label: String(row.numero),
    action: () => store.remove(row.id, row.rowVersion),
    onDone: () => table.reload(),
  })
}

function esFacturado(id: string): boolean {
  try {
    return Boolean(localStorage.getItem(`certaro:cert-facturado:${id}`))
  } catch {
    return false
  }
}

onMounted(async () => {
  table.start()
  try {
    ;[opcionesCliente.value, opcionesProyecto.value] = await Promise.all([
      clientes.lookup(undefined, 200),
      proyectos.lookup(undefined, undefined, 200),
    ])
  } catch (e) {
    notify(e)
  }
})
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="$t('Menu.Certificados')" :subtitle="$t('Certificados.Subtitle')">
      <template #actions>
        <HelpButton topic-id="certificados-overview" title="Ayuda sobre Certificados de Obra" />
      </template>
    </PageHeader>

    <FilterBar :active="filtrosActivos" @clear="table.resetFilter()">
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Certificados.Cliente') }}</span>
        <Select
          v-model="table.filter.value.clienteId"
          :options="opcionesCliente"
          option-label="label"
          option-value="id"
          filter
          show-clear
          :placeholder="$t('General.All')"
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Certificados.Proyecto') }}</span>
        <Select
          v-model="table.filter.value.proyectoId"
          :options="opcionesProyecto"
          option-label="label"
          option-value="id"
          filter
          show-clear
          :placeholder="$t('General.All')"
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Certificados.Trabajo') }}</span>
        <Select
          v-model="table.filter.value.trabajoId"
          :options="opcionesTrabajo"
          option-label="label"
          option-value="id"
          filter
          show-clear
          :placeholder="$t('General.All')"
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Certificados.Desde') }}</span>
        <DateInput v-model="table.filter.value.fechaDesde" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Certificados.Hasta') }}</span>
        <DateInput v-model="table.filter.value.fechaHasta" />
      </label>
    </FilterBar>

    <Divider />

    <DataGrid :table="table" empty-key="Certificados.Empty" class="flex-1" @row-edit="abrirDetalle">
      <Column field="numero" :header="$t('Certificados.Numero')" sortable />
      <Column field="fecha" :header="$t('Certificados.Fecha')" sortable>
        <template #body="{ data }"><DateText :value="data.fecha" /></template>
      </Column>
      <Column field="proyectoNombre" :header="$t('Certificados.Proyecto')">
        <template #body="{ data }">{{ data.proyectoNumero }} · {{ data.proyectoNombre }}</template>
      </Column>
      <Column field="trabajoDescripcion" :header="$t('Certificados.Trabajo')" />
      <Column field="ordenTitulo" :header="$t('Certificados.Orden')" />
      <Column field="totalCertificado" :header="$t('Certificados.TotalCertificado')">
        <template #body="{ data }"><MoneyText :value="data.totalCertificado" /></template>
      </Column>
      <Column field="totalNeto" :header="$t('Certificados.TotalNeto')" sortable>
        <template #body="{ data }"><MoneyText :value="data.totalNeto" /></template>
      </Column>
      <Column header="Facturación" class="w-28 text-center">
        <template #body="{ data }">
          <span
            v-if="esFacturado(data.id)"
            class="inline-flex items-center gap-1 rounded bg-success/15 px-2 py-0.5 text-xs font-semibold text-success"
          >
            <AppIcon name="check" :size="12" />
            Facturado
          </span>
          <span
            v-else
            class="inline-flex items-center gap-1 rounded bg-muted px-2 py-0.5 text-[11px] font-medium text-muted-foreground"
          >
            Pendiente
          </span>
        </template>
      </Column>

      <template #actions="{ data }">
        <div class="flex gap-1">
          <Button
            variant="ghost"
            size="sm"
            :title="$t('Certificados.VerDetalle')"
            @click="abrirDetalle(data)"
          >
            <AppIcon name="eye" :size="14" />
          </Button>
          <!-- Only the last one: voiding an earlier certificate would leave the later ones
               resting on a history that no longer explains them. -->
          <Button
            v-if="data.esUltimo"
            variant="ghost"
            size="sm"
            :title="$t('Certificados.Anular')"
            @click="onAnular(data)"
          >
            <AppIcon name="trash-2" :size="14" />
          </Button>
        </div>
      </template>
    </DataGrid>
  </section>
</template>
