<script setup lang="ts">
import { useRouter } from 'vue-router'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useUiStore } from '@/stores/useUiStore'

const router = useRouter()
const ui = useUiStore()
</script>

<template>
  <section
    class="relative flex flex-col gap-4 rounded-xl border border-primary/30 bg-primary/5 p-5 shadow-xs"
  >
    <div class="flex items-start justify-between">
      <div class="space-y-1">
        <h2 class="text-sm font-semibold text-foreground flex items-center gap-2">
          <AppIcon name="sparkles" :size="16" class="text-primary" />
          {{ $t('Dashboard.Onboarding.Title') }}
        </h2>
        <p class="text-xs text-muted-foreground">
          {{ $t('Dashboard.Onboarding.Subtitle') }}
        </p>
      </div>
      <Button
        variant="ghost"
        size="sm"
        :title="$t('General.Close')"
        @click="ui.dismissOnboarding()"
      >
        <AppIcon name="x" :size="14" />
      </Button>
    </div>

    <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
      <div class="flex flex-col justify-between rounded-lg border border-border bg-card p-3">
        <div class="space-y-1">
          <div class="flex items-center gap-2 font-medium text-xs">
            <span class="flex h-5 w-5 items-center justify-center rounded-full bg-primary/10 text-[10px] font-bold text-primary">1</span>
            {{ $t('Dashboard.Onboarding.Paso1Titulo') }}
          </div>
          <p class="text-[11px] text-muted-foreground">
            {{ $t('Dashboard.Onboarding.Paso1Desc') }}
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          class="mt-3 w-full text-xs"
          @click="router.push({ name: 'configuracion' })"
        >
          {{ $t('Dashboard.Onboarding.Paso1Accion') }}
        </Button>
      </div>

      <div class="flex flex-col justify-between rounded-lg border border-border bg-card p-3">
        <div class="space-y-1">
          <div class="flex items-center gap-2 font-medium text-xs">
            <span class="flex h-5 w-5 items-center justify-center rounded-full bg-primary/10 text-[10px] font-bold text-primary">2</span>
            {{ $t('Dashboard.Onboarding.Paso2Titulo') }}
          </div>
          <p class="text-[11px] text-muted-foreground">
            {{ $t('Dashboard.Onboarding.Paso2Desc') }}
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          class="mt-3 w-full text-xs"
          @click="router.push({ name: 'clientes' })"
        >
          {{ $t('Dashboard.Onboarding.Paso2Accion') }}
        </Button>
      </div>

      <div class="flex flex-col justify-between rounded-lg border border-border bg-card p-3">
        <div class="space-y-1">
          <div class="flex items-center gap-2 font-medium text-xs">
            <span class="flex h-5 w-5 items-center justify-center rounded-full bg-primary/10 text-[10px] font-bold text-primary">3</span>
            {{ $t('Dashboard.Onboarding.Paso3Titulo') }}
          </div>
          <p class="text-[11px] text-muted-foreground">
            {{ $t('Dashboard.Onboarding.Paso3Desc') }}
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          class="mt-3 w-full text-xs"
          @click="router.push({ name: 'proyectos' })"
        >
          {{ $t('Dashboard.Onboarding.Paso3Accion') }}
        </Button>
      </div>

      <div class="flex flex-col justify-between rounded-lg border border-border bg-card p-3">
        <div class="space-y-1">
          <div class="flex items-center gap-2 font-medium text-xs">
            <span class="flex h-5 w-5 items-center justify-center rounded-full bg-primary/10 text-[10px] font-bold text-primary">4</span>
            {{ $t('Dashboard.Onboarding.Paso4Titulo') }}
          </div>
          <p class="text-[11px] text-muted-foreground">
            {{ $t('Dashboard.Onboarding.Paso4Desc') }}
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          class="mt-3 w-full text-xs"
          @click="router.push({ name: 'movimientos' })"
        >
          {{ $t('Dashboard.Onboarding.Paso4Accion') }}
        </Button>
      </div>
    </div>
  </section>
</template>
