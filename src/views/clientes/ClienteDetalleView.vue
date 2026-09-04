<script setup lang="ts">
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import Divider from 'primevue/divider'
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import ListState from '@/components/domain/ListState.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import StatePill from '@/components/domain/StatePill.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useClientesStore, type ClienteDetalle } from '@/stores/useClientesStore'
import { useComercialStore } from '@/stores/useComercialStore'
import { useProyectosStore, type ProyectoListItem } from '@/stores/useProyectosStore'

const route = useRoute()
const router = useRouter()
const { notify } = useApiError()
const store = useClientesStore()
const comercialStore = useComercialStore()
const proyectosStore = useProyectosStore()

const clienteId = computed(() => String(route.params.clienteId ?? ''))
const cliente = ref<ClienteDetalle | null>(null)
const deudaCliente = ref<string>('0.0000')
const proyectosDelCliente = ref<ProyectoListItem[]>([])
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)

async function cargar(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    const [c, cc, proys] = await Promise.all([
      store.fetchOne(clienteId.value),
      comercialStore.fetchCuentaCorriente({ clienteId: clienteId.value }),
      proyectosStore.fetchPaged({
        page: 1,
        pageSize: 50,
        filtro: { clienteId: clienteId.value },
        sortDir: 'Desc',
      }),
    ])
    cliente.value = c
    deudaCliente.value = cc.saldo
    proyectosDelCliente.value = proys.items
  } catch (e) {
    error.value = notify(e)
  } finally {
    loading.value = false
    firstLoad.value = false
  }
}

onMounted(cargar)
</script>
<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader
      :title="cliente?.nombre ?? $t('Menu.Clientes')"
      :subtitle="cliente?.cuit ?? undefined"
    >
      <template #actions>
        <Button variant="outline" @click="router.back()">
          <AppIcon name="arrow-left" :size="16" />{{ $t('General.Back') }}
        </Button>
        <Button
          v-if="cliente"
          variant="outline"
          @click="router.push({ name: 'proyectos', query: { clienteId: cliente.id } })"
        >
          <AppIcon name="plus" :size="16" />
          {{ $t('General.New') }} {{ $t('Entity.Proyecto') }}
        </Button>
        <Button
          v-if="cliente"
          @click="router.push({ name: 'cliente-cuenta', params: { clienteId: cliente.id } })"
        >
          <AppIcon name="wallet" :size="16" />{{ $t('Clientes.CuentaCorriente') }}
        </Button>
      </template>
    </PageHeader>

    <Divider />
    <ListState
      :loading="loading"
      :first-load="firstLoad"
      :error="error"
      :is-empty="!cliente"
      :is-filtered="false"
      empty-key="Clientes.Empty"
      class="flex-1"
      @retry="cargar()"
    >
      <div v-if="cliente" class="space-y-4">
        <!-- Indicadores rápidos -->
        <div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
          <div class="rounded-md border border-border bg-card p-3 shadow-xs">
            <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              {{ $t('Clientes.Deuda') }}
            </span>
            <div class="mt-1 text-xl font-bold">
              <MoneyText :value="deudaCliente" colored />
            </div>
          </div>
          <div class="rounded-md border border-border bg-card p-3 shadow-xs">
            <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              {{ $t('Clientes.Proyectos') }}
            </span>
            <div class="mt-1 text-xl font-bold">{{ cliente.proyectosCount }}</div>
          </div>
          <div class="rounded-md border border-border bg-card p-3 shadow-xs">
            <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              {{ $t('Clientes.Facturas') }}
            </span>
            <div class="mt-1 text-xl font-bold">{{ cliente.facturasCount }}</div>
          </div>
        </div>

        <!-- Ficha de datos generales -->
        <div class="grid gap-4 rounded-md border border-border bg-surface-card p-4 text-sm md:grid-cols-2">
          <div>
            <span class="text-xs text-muted-foreground">{{ $t('Clientes.Nombre') }}</span>
            <p class="font-medium">{{ cliente.nombre }}</p>
          </div>
          <div>
            <span class="text-xs text-muted-foreground">{{ $t('Clientes.Cuit') }}</span>
            <p>{{ cliente.cuit ?? '—' }}</p>
          </div>
          <div>
            <span class="text-xs text-muted-foreground">{{ $t('Clientes.Direccion') }}</span>
            <p>{{ cliente.direccion ?? '—' }}</p>
          </div>
          <div>
            <span class="text-xs text-muted-foreground">{{ $t('Clientes.Telefono') }}</span>
            <p>{{ cliente.telefono ?? '—' }}</p>
          </div>
          <div>
            <span class="text-xs text-muted-foreground">{{ $t('Clientes.Email') }}</span>
            <p>{{ cliente.email ?? '—' }}</p>
          </div>
          <div>
            <span class="text-xs text-muted-foreground">{{ $t('Clientes.CondicionIva') }}</span>
            <p>{{ cliente.condicionIva ?? '—' }}</p>
          </div>
        </div>

        <!-- Contactos del cliente -->
        <div v-if="cliente.contactos?.length" class="space-y-2">
          <h3 class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            {{ $t('Clientes.Contactos') }}
          </h3>
          <div class="grid gap-3 sm:grid-cols-2">
            <div
              v-for="c in cliente.contactos"
              :key="c.id"
              class="rounded-md border border-border bg-surface-card p-3 text-xs shadow-xs"
            >
              <div class="flex items-center justify-between">
                <span class="font-medium text-foreground">{{ c.nombre || c.etiqueta }}</span>
                <span
                  v-if="c.esPrincipal"
                  class="rounded bg-primary/10 px-1.5 py-0.5 text-[10px] font-medium text-primary"
                >
                  {{ $t('Clientes.EsPrincipal') }}
                </span>
              </div>
              <p v-if="c.email" class="mt-1 text-muted-foreground">{{ c.email }}</p>
              <p v-if="c.telefono" class="text-muted-foreground">{{ c.telefono }}</p>
            </div>
          </div>
        </div>

        <!-- Proyectos asociados -->
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <h3 class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              {{ $t('Clientes.ProyectosAsociados') }}
            </h3>
            <Button
              variant="outline"
              size="sm"
              class="text-xs"
              @click="router.push({ name: 'proyectos', query: { clienteId: cliente.id } })"
            >
              <AppIcon name="plus" :size="14" />
              {{ $t('General.New') }}
            </Button>
          </div>

          <div v-if="!proyectosDelCliente.length" class="rounded-md border border-border p-4 text-center text-xs text-muted-foreground">
            {{ $t('Clientes.SinProyectos') }}
          </div>

          <DataTable
            v-else
            :value="proyectosDelCliente"
            responsive-layout="scroll"
            class="text-sm border border-border rounded-md overflow-hidden"
          >
            <Column field="numero" :header="$t('Proyectos.Numero')" :sortable="true" style="width: 100px">
              <template #body="{ data }">
                <span class="font-mono text-xs font-semibold">#{{ data.numero }}</span>
              </template>
            </Column>
            <Column field="nombre" :header="$t('Proyectos.Nombre')">
              <template #body="{ data }">
                <button
                  type="button"
                  class="text-left font-medium text-primary hover:underline cursor-pointer"
                  @click="router.push({ name: 'proyecto-detalle', params: { proyectoId: data.id } })"
                >
                  {{ data.nombre }}
                </button>
              </template>
            </Column>
            <Column field="estado.actual" :header="$t('Proyectos.Estado')" style="width: 130px">
              <template #body="{ data }">
                <StatePill entity="Proyecto" :value="data.estado.actual" />
              </template>
            </Column>
            <Column :header="$t('General.Actions')" style="width: 80px">
              <template #body="{ data }">
                <Button
                  variant="ghost"
                  size="sm"
                  :title="$t('General.View')"
                  @click="router.push({ name: 'proyecto-detalle', params: { proyectoId: data.id } })"
                >
                  <AppIcon name="arrow-right" :size="14" />
                </Button>
              </template>
            </Column>
          </DataTable>
        </div>
      </div>
    </ListState>
  </section>
</template>
