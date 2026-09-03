<script setup lang="ts">
import { computed, ref } from 'vue'
import { HELP_REGISTRY, type HelpTopic } from '@/lib/helpRegistry'

const props = withDefaults(
  defineProps<{
    topicId: string
    label?: string
    iconSize?: number
    title?: string
  }>(),
  {
    label: '',
    iconSize: 13,
    title: 'Ayuda y documentación de esta función',
  },
)

const isOpen = ref(false)

const topic = computed<HelpTopic | null>(() => {
  return HELP_REGISTRY[props.topicId] ?? null
})

function openModal() {
  if (topic.value) {
    isOpen.value = true
  }
}

function closeModal() {
  isOpen.value = false
}
</script>

<template>
  <div class="inline-flex items-center">
    <button
      type="button"
      class="inline-flex items-center gap-1 rounded-full p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors focus:outline-hidden focus:ring-1 focus:ring-primary"
      :title="props.title"
      @click.stop="openModal"
    >
      <span
        class="inline-flex items-center justify-center font-bold font-mono rounded-full border border-border bg-surface-elevated text-foreground"
        :style="{ width: `${props.iconSize + 5}px`, height: `${props.iconSize + 5}px`, fontSize: `${props.iconSize - 2}px` }"
      >
        ?
      </span>
      <span v-if="props.label" class="text-xs font-medium">{{ props.label }}</span>
    </button>

    <!-- Contextual Help Modal -->
    <Teleport to="body">
      <div
        v-if="isOpen && topic"
        class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-xs p-4 select-none"
        @click.self="closeModal"
      >
        <div
          class="w-full max-w-xl max-h-[85vh] rounded-2xl bg-surface-card border border-border shadow-2xl flex flex-col overflow-hidden text-foreground animate-in fade-in zoom-in-95 duration-150"
        >
          <!-- Header -->
          <div class="p-5 border-b border-border bg-muted/20 flex items-start justify-between gap-4">
            <div class="flex items-start gap-3">
              <span
                class="w-8 h-8 rounded-xl bg-primary/10 text-primary border border-primary/20 flex items-center justify-center font-mono font-bold text-sm shrink-0"
              >
                ?
              </span>
              <div>
                <h3 class="text-base font-bold text-foreground leading-tight">
                  {{ topic.title }}
                </h3>
                <p class="text-xs text-muted-foreground mt-0.5">
                  {{ topic.subtitle }}
                </p>
              </div>
            </div>
            <button
              type="button"
              class="rounded-lg p-1.5 text-muted-foreground hover:bg-muted hover:text-foreground text-sm"
              title="Cerrar"
              @click="closeModal"
            >
              ✕
            </button>
          </div>

          <!-- Body -->
          <div class="flex-1 overflow-y-auto p-5 flex flex-col gap-4 text-xs">
            <!-- Purpose -->
            <div class="p-3.5 rounded-xl bg-primary/5 border border-primary/20 text-foreground leading-relaxed">
              <strong class="font-semibold text-primary block mb-1">¿Para qué sirve?</strong>
              {{ topic.purpose }}
            </div>

            <!-- Workflow -->
            <div v-if="topic.workflow.length > 0">
              <h4 class="font-semibold text-foreground mb-2 flex items-center gap-1.5 text-xs">
                <span>🔄</span>
                <span>Flujo de Trabajo Recomendado</span>
              </h4>
              <ol class="flex flex-col gap-1.5 pl-1">
                <li
                  v-for="(step, idx) in topic.workflow"
                  :key="idx"
                  class="flex items-start gap-2 text-muted-foreground leading-normal"
                >
                  <span
                    class="w-4 h-4 rounded-full bg-surface-elevated border border-border text-foreground font-mono text-[10px] font-bold flex items-center justify-center shrink-0 mt-0.5"
                  >
                    {{ idx + 1 }}
                  </span>
                  <span class="text-foreground">{{ step }}</span>
                </li>
              </ol>
            </div>

            <!-- Strengths / Capabilities -->
            <div v-if="topic.strengths.length > 0">
              <h4 class="font-semibold text-foreground mb-2 flex items-center gap-1.5 text-xs">
                <span>✨</span>
                <span>Capacidades y Fortalezas</span>
              </h4>
              <ul class="grid grid-cols-1 gap-1.5 pl-1">
                <li
                  v-for="(item, idx) in topic.strengths"
                  :key="idx"
                  class="flex items-start gap-2 text-foreground"
                >
                  <span class="text-primary font-bold shrink-0">✓</span>
                  <span>{{ item }}</span>
                </li>
              </ul>
            </div>

            <!-- Limitations / Business Rules -->
            <div v-if="topic.limitations.length > 0" class="p-3.5 rounded-xl bg-warning/5 border border-warning/30 text-foreground">
              <h4 class="font-semibold text-warning mb-1.5 flex items-center gap-1.5 text-xs">
                <span>⚠</span>
                <span>Reglas de Negocio y Restricciones</span>
              </h4>
              <ul class="flex flex-col gap-1.5 pl-1 text-[11px] text-muted-foreground">
                <li v-for="(lim, idx) in topic.limitations" :key="idx" class="flex items-start gap-2">
                  <span class="text-warning font-bold shrink-0">•</span>
                  <span class="text-foreground">{{ lim }}</span>
                </li>
              </ul>
            </div>

            <!-- Tips -->
            <div v-if="topic.tips.length > 0" class="p-3 rounded-xl bg-surface-elevated border border-border">
              <h4 class="font-semibold text-foreground mb-1 flex items-center gap-1.5 text-xs">
                <span>💡</span>
                <span>Consejos de Productividad</span>
              </h4>
              <ul class="flex flex-col gap-1 pl-1 text-muted-foreground">
                <li v-for="(tip, idx) in topic.tips" :key="idx" class="flex items-start gap-2">
                  <span class="text-primary font-bold shrink-0">→</span>
                  <span>{{ tip }}</span>
                </li>
              </ul>
            </div>
          </div>

          <!-- Footer -->
          <div class="p-3.5 border-t border-border bg-muted/10 flex justify-end">
            <button
              type="button"
              class="px-4 py-1.5 rounded-lg text-xs font-semibold bg-primary text-primary-foreground hover:bg-primary/90 transition-colors shadow-xs"
              @click="closeModal"
            >
              Entendido
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
