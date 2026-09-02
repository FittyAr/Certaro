<script setup lang="ts">
import ContextMenu from 'primevue/contextmenu'
import { ref } from 'vue'

export interface ContextMenuItem {
  label: string
  icon?: string
  command?: () => void
  disabled?: boolean
  separator?: boolean
  items?: ContextMenuItem[]
}

const props = defineProps<{
  items: ContextMenuItem[]
}>()

const menu = ref<InstanceType<typeof ContextMenu> | null>(null)

function show(event: MouseEvent): void {
  menu.value?.show(event)
}

defineExpose({ show })
</script>

<template>
  <ContextMenu ref="menu" :model="props.items as any" />
</template>
