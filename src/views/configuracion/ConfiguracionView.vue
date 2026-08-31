<script setup lang="ts">
import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import Tab from 'primevue/tab'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'
import { onMounted, ref } from 'vue'

import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { useSistemaStore } from '@/stores/useSistemaStore'

import GeneralSection from './sections/GeneralSection.vue'
import BusinessSection from './sections/BusinessSection.vue'
import SettlementSection from './sections/SettlementSection.vue'
import CommunicationSection from './sections/CommunicationSection.vue'
import IntegrationsSection from './sections/IntegrationsSection.vue'
import SystemSection from './sections/SystemSection.vue'

/**
 * Settings screen. See `docs/09-modulos-funcionales.md` §3.15.
 *
 * Six tabs, each owning its section of AppConfig. Changes are applied with an explicit button,
 * not on every keystroke: leaving with unapplied changes asks first.
 */

const sistema = useSistemaStore()
const activeTab = ref('general')

onMounted(() => void sistema.loadConfig())
</script>

<template>
  <section class="flex h-full flex-col gap-6 overflow-auto p-6">
    <PageHeader :title="$t('Configuracion.Title')" />

    <Tabs v-model:value="activeTab" lazy class="flex-1">
      <TabList class="border-b border-border">
        <Tab value="general" class="flex items-center gap-2">
          <AppIcon name="sliders" :size="16" />
          <span>{{ $t('Configuracion.General') }}</span>
        </Tab>
        <Tab value="business" class="flex items-center gap-2">
          <AppIcon name="building-2" :size="16" />
          <span>{{ $t('Configuracion.Business') }}</span>
        </Tab>
        <Tab value="settlement" class="flex items-center gap-2">
          <AppIcon name="users" :size="16" />
          <span>{{ $t('Configuracion.Settlement') }}</span>
        </Tab>
        <Tab value="communication" class="flex items-center gap-2">
          <AppIcon name="message-square" :size="16" />
          <span>{{ $t('Configuracion.Communication') }}</span>
        </Tab>
        <Tab value="integrations" class="flex items-center gap-2">
          <AppIcon name="globe" :size="16" />
          <span>{{ $t('Configuracion.Integrations') }}</span>
        </Tab>
        <Tab value="system" class="flex items-center gap-2">
          <AppIcon name="database" :size="16" />
          <span>{{ $t('Configuracion.Sistema') }}</span>
        </Tab>
      </TabList>
      <TabPanels class="bg-transparent px-0 py-6">
        <TabPanel value="general">
          <GeneralSection />
        </TabPanel>
        <TabPanel value="business">
          <BusinessSection />
        </TabPanel>
        <TabPanel value="settlement">
          <SettlementSection />
        </TabPanel>
        <TabPanel value="communication">
          <CommunicationSection />
        </TabPanel>
        <TabPanel value="integrations">
          <IntegrationsSection />
        </TabPanel>
        <TabPanel value="system">
          <SystemSection />
        </TabPanel>
      </TabPanels>
    </Tabs>
  </section>
</template>
