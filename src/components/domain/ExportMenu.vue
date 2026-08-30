<script setup lang="ts">
import Menu from 'primevue/menu'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import { FORMATOS_MOVIMIENTOS, type ExportResult, type FormatoExport } from '@/api/reportes'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useExport } from '@/composables/useExport'

/**
 * The export button, one per screen that exports. See `docs/12` §1.2.
 *
 * A single button with the formats behind it, rather than one button per format: the legacy toolbar
 * had four and the user still could not tell which covered the filter.
 */

const props = withDefaults(
  defineProps<{
    /** Report identifier the backend knows. */
    reporte: string
    /** Formats offered, in menu order. */
    formatos?: readonly FormatoExport[]
    /** Names the subject, for the suggested filename. */
    detalle?: string
    /** Rows the export will cover, to warn that it is the filter and not the page. */
    cantidad?: number
    disabled?: boolean
    run: (formato: FormatoExport, destino: string) => Promise<ExportResult>
  }>(),
  { formatos: () => FORMATOS_MOVIMIENTOS, detalle: undefined, cantidad: undefined },
)

const { t } = useI18n()
const { exportar, exportando } = useExport()

const menu = ref<InstanceType<typeof Menu> | null>(null)

const items = computed(() =>
  props.formatos.map((formato) => ({
    label: t(`Export.Tipo.${formato}`),
    command: () =>
      void exportar({
        reporte: props.reporte,
        formato,
        detalle: props.detalle,
        run: (destino) => props.run(formato, destino),
      }),
  })),
)

const aviso = computed(() =>
  props.cantidad === undefined
    ? undefined
    : t('Export.ScopeNotice', { cantidad: props.cantidad }),
)
</script>

<template>
  <div>
    <Button
      variant="secondary"
      :disabled="disabled || exportando"
      :title="aviso"
      aria-haspopup="true"
      aria-controls="export-menu"
      @click="menu?.toggle($event)"
    >
      <AppIcon :name="exportando ? 'loader' : 'download'" :size="16" />
      {{ exportando ? $t('Export.Generando') : $t('Export.Exportar') }}
    </Button>
    <Menu id="export-menu" ref="menu" :model="items" :popup="true" />
  </div>
</template>
