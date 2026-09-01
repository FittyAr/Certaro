<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ListState from '@/components/domain/ListState.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useTrabajosStore, type TrabajoDetalle } from '@/stores/useTrabajosStore'
const route = useRoute()
const router = useRouter()
const { notify } = useApiError()
const store = useTrabajosStore()
const trabajoId = computed(() => String(route.params.trabajoId ?? ''))
const trabajo = ref<TrabajoDetalle | null>(null)
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)
async function cargar(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    trabajo.value = await store.fetchOne(trabajoId.value)
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
      :title="trabajo?.descripcion ?? $t('Menu.Trabajos')"
      :subtitle="trabajo?.obraNombre"
    >
      <template #actions>
        <Button variant="outline" @click="router.back()">
          <AppIcon name="arrow-left" :size="16" />{{ $t('General.Back') }}
        </Button>
        <Button
          v-if="trabajo"
          @click="router.push({ name: 'trabajo-ordenes', params: { trabajoId: trabajo.id } })"
        >
          Ordenes
        </Button>
      </template>
    </PageHeader>
    <ListState
      :loading="loading"
      :first-load="firstLoad"
      :error="error"
      :is-empty="!trabajo"
      :is-filtered="false"
      empty-key="Trabajos.Empty"
      class="flex-1"
      @retry="cargar()"
    >
      <div
        v-if="trabajo"
        class="grid gap-4 rounded-md border border-border p-4 text-sm md:grid-cols-2"
      >
        <div>
          <span class="text-xs text-muted-foreground">Obra</span>
          <p>{{ trabajo.obraNombre }}</p>
        </div>
        <div>
          <span class="text-xs text-muted-foreground">Estado</span>
          <p>{{ trabajo.estado }}</p>
        </div>
        <div>
          <span class="text-xs text-muted-foreground">Presupuesto</span>
          <p><MoneyText :value="trabajo.presupuesto" /></p>
        </div>
        <div>
          <span class="text-xs text-muted-foreground">Cliente</span>
          <p>{{ trabajo.clienteNombre }}</p>
        </div>
      </div>
    </ListState>
  </section>
</template>
