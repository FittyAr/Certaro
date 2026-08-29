<script setup lang="ts">
/**
 * The validation message of one field. Rendered with an id so the input can point at it with
 * `aria-describedby`, which is what makes a screen reader announce the problem.
 */
defineProps<{ id: string; message?: string }>()
</script>

<template>
  <p v-if="message" :id="id" class="text-xs text-destructive" role="alert">{{ message }}</p>
</template>
