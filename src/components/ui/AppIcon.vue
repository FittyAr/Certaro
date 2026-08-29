<script setup lang="ts">
import * as lucide from 'lucide-vue-next'
import { computed, type Component } from 'vue'

/**
 * Resolves a Lucide icon from its kebab-case name, which is the form the menu declares.
 *
 * Doing the lookup here rather than importing each icon where it is used keeps the menu a plain
 * data structure: adding a screen means adding a row to `MENU`, not an import somewhere else.
 */
const props = withDefaults(defineProps<{ name: string; size?: number }>(), { size: 18 })

function toPascal(name: string): string {
  return name
    .split('-')
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join('')
}

const icons = lucide as unknown as Record<string, Component>

// An unknown name falls back to a plain circle rather than rendering nothing, so a typo shows up
// as a placeholder instead of a gap in the menu.
const component = computed<Component>(() => icons[toPascal(props.name)] ?? icons.Circle!)
</script>

<template>
  <component :is="component" :size="props.size" aria-hidden="true" />
</template>
