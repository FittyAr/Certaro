<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ListState from '@/components/domain/ListState.vue'
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
          @click="router.push({ name: 'cliente-cuenta', params: { clienteId: cliente.id } })"
        >
          <AppIcon name="wallet" :size="16" />{{ $t('Clientes.CuentaCorriente') }}
        </Button>
      </template>
    </PageHeader>
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
      <div
        v-if="cliente"
        class="grid gap-4 rounded-md border border-border p-4 text-sm md:grid-cols-2"
      >
        <div>
          <span class="text-xs text-muted-foreground">{{ $t('Clientes.Nombre') }}</span>
          <p>{{ cliente.nombre }}</p>
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
    </ListState>
  </section>
</template>
