<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import Drawer from 'primevue/drawer'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import type { EntidadAdjunto } from '@/api/adjuntos'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useAdjuntosStore } from '@/stores/useAdjuntosStore'
import type { Uuid } from '@/api/types'

/**
 * Attachment drawer. See `docs/13-servicios-externos-y-archivos.md` §1.
 *
 * Opens from any screen that supports attachments. The list loads when the drawer opens and
 * clears when it closes, so a stale count never survives from a previous open.
 */

const props = defineProps<{
  entidadTipo: EntidadAdjunto
  entidadId: Uuid | null
  /** How many attachments the parent knows about, for the badge on the trigger button. */
  count?: number
}>()

const visible = defineModel<boolean>('visible', { default: false })

const { t } = useI18n()
const { confirmDelete } = useConfirmDelete()
const store = useAdjuntosStore()

async function onOpen(): Promise<void> {
  if (props.entidadId) {
    await store.load(props.entidadTipo, props.entidadId)
  }
}

function onClose(): void {
  store.clear()
}

async function onAdd(): Promise<void> {
  if (!props.entidadId) return
  const ruta = await open({
    multiple: false,
    title: t('Adjuntos.Add'),
  })
  if (!ruta) return
  await store.add({
    entidadTipo: props.entidadTipo,
    entidadId: props.entidadId,
    rutaOrigen: ruta,
  })
}

function onDelete(id: Uuid, nombre: string): void {
  confirmDelete({
    entityKey: 'Entity.Adjunto',
    label: nombre,
    action: () => store.remove(id),
  })
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

const hasItems = computed(() => store.items.length > 0)
</script>

<template>
  <Drawer
    v-model:visible="visible"
    position="right"
    :close-on-escape="true"
    class="w-full md:!w-[420px]"
    @show="onOpen"
    @hide="onClose"
  >
    <template #header>
      <h3 class="text-base font-semibold">
        {{ $t('Adjuntos.Title') }}
        <span v-if="count" class="ml-1 text-sm text-muted-foreground">({{ count }})</span>
      </h3>
    </template>

    <div class="flex flex-col gap-3">
      <div v-if="store.loading" class="py-8 text-center text-muted-foreground">
        {{ $t('General.Loading') }}
      </div>

      <div v-else-if="!hasItems" class="py-8 text-center text-muted-foreground">
        <p>{{ $t('Adjuntos.Empty') }}</p>
        <p class="mt-1 text-xs">{{ $t('Adjuntos.EmptyHint') }}</p>
      </div>

      <div v-else class="flex flex-col gap-2">
        <div
          v-for="adjunto in store.items"
          :key="adjunto.id"
          class="flex items-center gap-3 rounded-md border border-border bg-surface-raised p-3"
        >
          <AppIcon name="file" :size="18" class="shrink-0 text-muted-foreground" />
          <div class="min-w-0 flex-1">
            <p class="truncate text-sm font-medium">{{ adjunto.nombreArchivo }}</p>
            <p class="text-xs text-muted-foreground">
              {{ formatSize(adjunto.tamano) }} · {{ adjunto.mime }}
            </p>
          </div>
          <div class="flex shrink-0 gap-1">
            <Button
              variant="ghost"
              size="sm"
              :title="$t('Adjuntos.Open')"
              @click="store.open(adjunto.id)"
            >
              <AppIcon name="external-link" :size="14" />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              :title="$t('Adjuntos.Reveal')"
              @click="store.reveal(adjunto.id)"
            >
              <AppIcon name="folder-open" :size="14" />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              :title="$t('General.Delete')"
              @click="onDelete(adjunto.id, adjunto.nombreArchivo)"
            >
              <AppIcon name="trash-2" :size="14" />
            </Button>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <Button :disabled="!entidadId" @click="onAdd">
        <AppIcon name="plus" :size="16" />
        {{ $t('Adjuntos.Add') }}
      </Button>
    </template>
  </Drawer>
</template>
