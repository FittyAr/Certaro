<script setup lang="ts">
import Dialog from 'primevue/dialog'
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import AppIcon from '@/components/ui/AppIcon.vue'
import { useEscapeLayer } from '@/composables/useEscapeStack'
import { menuItems, numericShortcutRoutes } from '@/router/menu'
import { useConfigStore } from '@/stores/useConfigStore'
import { useNavigationStore } from '@/stores/useNavigationStore'
import { useUiStore } from '@/stores/useUiStore'

/**
 * Command palette of `docs/10-navegacion-y-atajos.md` §5.
 *
 * Every destination comes from `MENU`, so a new screen is in the palette the moment it is in the
 * menu. The legacy palette listed 10 of 15 screens and matched against a keyword field that
 * literally held the route name, so searching for «cobro» found nothing.
 */

interface Command {
  id: string
  groupKey: string
  label: string
  /** Lowercased, unaccented text the query is matched against. */
  haystack: string
  icon: string
  shortcut?: string
  run: () => void
}

const ui = useUiStore()
const config = useConfigStore()
const navigation = useNavigationStore()
const router = useRouter()
const { t } = useI18n()

const query = ref('')
const highlighted = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)

const layer = useEscapeLayer('palette', () => {
  if (!ui.commandPaletteOpen) return false
  ui.commandPaletteOpen = false
  return true
})

/** Accents are removed so «liquidacion» finds «Liquidaciones». */
function fold(value: string): string {
  return value
    .normalize('NFD')
    .replace(/\p{Diacritic}/gu, '')
    .toLowerCase()
}

const seedEnabled = computed(() => config.config?.application.seedEnabled ?? false)

const shortcutOf = computed(() => {
  const map = new Map<string, string>()
  numericShortcutRoutes().forEach((name, index) => map.set(name, `Ctrl+${index + 1}`))
  return map
})

const commands = computed<Command[]>(() =>
  menuItems(seedEnabled.value).map((item) => {
    const label = t(item.labelKey)
    // Synonyms are i18n keys, not a string glued to the label, so «sueldos» finds Liquidaciones
    // in Spanish and «payroll» does in English.
    const synonyms = (item.synonymKeys ?? []).map((key) => t(key))
    return {
      id: item.route,
      groupKey: 'CommandPalette.Group.Navigation',
      label,
      haystack: fold([label, ...synonyms].join(' ')),
      icon: item.icon,
      shortcut: shortcutOf.value.get(item.route),
      run: () => void router.push({ name: item.route }),
    }
  }),
)

const results = computed(() => {
  const needle = fold(query.value.trim())
  if (!needle) {
    // With no text: the recently used destinations first, then the whole menu.
    const recent = navigation.recent
    return [...commands.value].sort((a, b) => indexOrLast(recent, a.id) - indexOrLast(recent, b.id))
  }
  return commands.value.filter((c) => c.haystack.includes(needle))
})

function indexOrLast(list: string[], id: string): number {
  const index = list.indexOf(id)
  return index === -1 ? Number.MAX_SAFE_INTEGER : index
}

/** Results in display order, grouped by category. */
const groups = computed(() => {
  const map = new Map<string, Command[]>()
  for (const command of results.value) {
    const bucket = map.get(command.groupKey) ?? []
    bucket.push(command)
    map.set(command.groupKey, bucket)
  }
  return [...map.entries()].map(([groupKey, items]) => ({ groupKey, items }))
})

watch(
  () => ui.commandPaletteOpen,
  async (open) => {
    if (!open) {
      layer.pop()
      return
    }
    layer.push()
    query.value = ''
    highlighted.value = 0
    await nextTick()
    inputRef.value?.focus()
  },
)

watch(query, () => {
  highlighted.value = 0
})

function move(delta: number): void {
  const count = results.value.length
  if (count === 0) return
  highlighted.value = (highlighted.value + delta + count) % count
}

function run(command: Command | undefined): void {
  if (!command) return
  ui.commandPaletteOpen = false
  command.run()
}

function positionOf(command: Command): number {
  return results.value.indexOf(command)
}
</script>

<template>
  <Dialog
    v-model:visible="ui.commandPaletteOpen"
    modal
    dismissable-mask
    :show-header="false"
    :close-on-escape="false"
    class="w-full max-w-xl"
    :pt="{ content: { class: 'p-0' } }"
  >
    <div class="flex items-center gap-2 border-b border-border px-3 py-2">
      <AppIcon name="search" :size="16" />
      <input
        ref="inputRef"
        v-model="query"
        class="w-full bg-transparent py-1 text-sm outline-none"
        :placeholder="$t('CommandPalette.SearchPlaceholder')"
        :aria-label="$t('CommandPalette.SearchPlaceholder')"
        @keydown.down.prevent="move(1)"
        @keydown.up.prevent="move(-1)"
        @keydown.enter.prevent="run(results[highlighted])"
      />
    </div>

    <div class="max-h-80 overflow-y-auto py-1">
      <p v-if="results.length === 0" class="px-4 py-6 text-center text-sm text-muted-foreground">
        {{ $t('CommandPalette.NoResults') }}
      </p>

      <div v-for="group in groups" :key="group.groupKey">
        <p class="px-3 py-1 text-xs uppercase tracking-wide text-muted-foreground">
          {{ $t(group.groupKey) }}
        </p>
        <button
          v-for="command in group.items"
          :key="command.id"
          type="button"
          class="flex w-full items-center gap-3 px-3 py-2 text-left text-sm hover:bg-muted"
          :class="positionOf(command) === highlighted ? 'bg-muted' : ''"
          @click="run(command)"
          @mousemove="highlighted = positionOf(command)"
        >
          <AppIcon :name="command.icon" :size="16" />
          <span class="flex-1 truncate">{{ command.label }}</span>
          <span v-if="command.shortcut" class="text-xs text-muted-foreground">
            {{ command.shortcut }}
          </span>
        </button>
      </div>
    </div>

    <p class="border-t border-border px-3 py-2 text-xs text-muted-foreground">
      {{ $t('CommandPalette.Hint') }}
    </p>
  </Dialog>
</template>
