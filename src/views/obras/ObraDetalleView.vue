<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ListState from '@/components/domain/ListState.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useObrasStore, type ObraDetalle } from '@/stores/useObrasStore'
const route = useRoute()
const router = useRouter()
const { notify } = useApiError()
const store = useObrasStore()
const obraId = computed(() => String(route.params.obraId ?? ''))
const obra = ref<ObraDetalle | null>(null)
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)
async function cargar(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    obra.value = await store.fetchOne(obraId.value)
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
    <PageHeader :title="obra?.nombre ?? $t('Menu.Obras')" :subtitle="obra?.clienteNombre">
      <template #actions>
        <Button variant="outline" @click="router.back()">
          <AppIcon name="arrow-left" :size="16" />{{ $t('General.Back') }}
        </Button>
        <Button
          v-if="obra"
          variant="outline"
          @click="router.push({ name: 'obra-trabajos', params: { obraId: obra.id } })"
        >
          {{ $t('Obras.VerTrabajos') }}
        </Button>
        <Button
          v-if="obra"
          variant="outline"
          @click="router.push({ name: 'obra-caja', params: { obraId: obra.id } })"
        >
          {{ $t('Obras.VerCaja') }}
        </Button>
      </template>
    </PageHeader>
    <ListState
      :loading="loading"
      :first-load="firstLoad"
      :error="error"
      :is-empty="!obra"
      :is-filtered="false"
      empty-key="Obras.Empty"
      class="flex-1"
      @retry="cargar()"
    >
      <div
        v-if="obra"
        class="grid gap-4 rounded-md border border-border p-4 text-sm md:grid-cols-2"
      >
        <div>
          <span class="text-xs text-muted-foreground">Numero</span>
          <p>{{ obra.numero }}</p>
        </div>
        <div>
          <span class="text-xs text-muted-foreground">Estado</span>
          <p>{{ obra.estado }}</p>
        </div>
        <div>
          <span class="text-xs text-muted-foreground">Cliente</span>
          <p>{{ obra.clienteNombre }}</p>
        </div>
        <div>
          <span class="text-xs text-muted-foreground">Localidad</span>
          <p>{{ obra.localidad ?? '—' }}</p>
        </div>
        <div class="md:col-span-2">
          <span class="text-xs text-muted-foreground">Direccion</span>
          <p>{{ obra.direccion ?? '—' }}</p>
        </div>
      </div>
    </ListState>
  </section>
</template>
