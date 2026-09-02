<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ListState from '@/components/domain/ListState.vue'
import StatePill from '@/components/domain/StatePill.vue'
import Divider from 'primevue/divider'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useProyectosStore, type ProyectoDetalle } from '@/stores/useProyectosStore'
const route = useRoute()
const router = useRouter()
const { notify } = useApiError()
const store = useProyectosStore()
const proyectoId = computed(() => String(route.params.proyectoId ?? ''))
const proyecto = ref<ProyectoDetalle | null>(null)
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)
async function cargar(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    proyecto.value = await store.fetchOne(proyectoId.value)
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
    <PageHeader :title="proyecto?.nombre ?? $t('Menu.Proyectos')" :subtitle="proyecto?.clienteNombre">
      <template #actions>
        <Button variant="outline" @click="router.back()">
          <AppIcon name="arrow-left" :size="16" />{{ $t('General.Back') }}
        </Button>
        <Button
          v-if="proyecto"
          variant="outline"
          @click="router.push({ name: 'proyecto-trabajos', params: { proyectoId: proyecto.id } })"
        >
          {{ $t('Proyectos.VerTrabajos') }}
        </Button>
        <Button
          v-if="proyecto"
          variant="outline"
          @click="router.push({ name: 'proyecto-caja', params: { proyectoId: proyecto.id } })"
        >
          {{ $t('Proyectos.VerCaja') }}
        </Button>
      </template>
    </PageHeader>

    <Divider />
    <ListState
      :loading="loading"
      :first-load="firstLoad"
      :error="error"
      :is-empty="!proyecto"
      :is-filtered="false"
      empty-key="Proyectos.Empty"
      class="flex-1"
      @retry="cargar()"
    >
      <div
        v-if="proyecto"
        class="grid gap-4 rounded-md border border-border p-4 text-sm md:grid-cols-2"
      >
        <div>
          <span class="text-xs text-muted-foreground">Numero</span>
          <p>{{ proyecto.numero }}</p>
        </div>
        <div>
          <span class="text-xs text-muted-foreground">Estado</span>
          <p><StatePill entity="Proyecto" :value="proyecto.estado.actual" /></p>
        </div>
        <div>
          <span class="text-xs text-muted-foreground">Cliente</span>
          <p>{{ proyecto.clienteNombre }}</p>
        </div>
        <div>
          <span class="text-xs text-muted-foreground">Localidad</span>
          <p>{{ proyecto.localidad ?? '—' }}</p>
        </div>
        <div class="md:col-span-2">
          <span class="text-xs text-muted-foreground">Direccion</span>
          <p>{{ proyecto.direccion ?? '—' }}</p>
        </div>
      </div>
    </ListState>
  </section>
</template>
