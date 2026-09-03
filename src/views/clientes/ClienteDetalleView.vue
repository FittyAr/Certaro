<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ListState from '@/components/domain/ListState.vue'
import Divider from 'primevue/divider'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useClientesStore, type ClienteDetalle } from '@/stores/useClientesStore'
const route = useRoute()
const router = useRouter()
const { notify } = useApiError()
const store = useClientesStore()
const clienteId = computed(() => String(route.params.clienteId ?? ''))
const cliente = ref<ClienteDetalle | null>(null)
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)
async function cargar(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    cliente.value = await store.fetchOne(clienteId.value)
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
      </div>
    </ListState>
  </section>
</template>
