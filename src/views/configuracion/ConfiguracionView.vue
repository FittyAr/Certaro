<script setup lang="ts">
import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import Tab from 'primevue/tab'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'
import { onMounted, ref } from 'vue'

import PageHeader from '@/components/domain/PageHeader.vue'
import { useSistemaStore } from '@/stores/useSistemaStore'

import GeneralSection from './sections/GeneralSection.vue'
import BusinessSection from './sections/BusinessSection.vue'
import CommunicationSection from './sections/CommunicationSection.vue'
import IntegrationsSection from './sections/IntegrationsSection.vue'
import SystemSection from './sections/SystemSection.vue'

/**
 * Settings screen. See `docs/09-modulos-funcionales.md` §3.15.
 *
 * Five tabs, each owning its section of AppConfig. Changes are applied with an explicit button,
 * not on every keystroke: leaving with unapplied changes asks first.
 */

const sistema = useSistemaStore()
const activeTab = ref('general')

onMounted(() => void sistema.loadConfig())
</script>

<template>
  <section class="flex h-full flex-col gap-4 overflow-auto p-6">
    <PageHeader :title="$t('Configuracion.Title')" />

    <Tabs v-model:value="activeTab" lazy>
      <TabList>
        <Tab value="general">{{ $t('Configuracion.General') }}</Tab>
        <Tab value="business">{{ $t('Configuracion.Business') }}</Tab>
        <Tab value="communication">{{ $t('Configuracion.Communication') }}</Tab>
        <Tab value="integrations">{{ $t('Configuracion.Integrations') }}</Tab>
        <Tab value="system">{{ $t('Configuracion.Sistema') }}</Tab>
      </TabList>
      <TabPanels>
        <TabPanel value="general">
          <GeneralSection />
        </TabPanel>
        <TabPanel value="business">
          <BusinessSection />
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
