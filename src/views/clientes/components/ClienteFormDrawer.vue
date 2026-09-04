<script setup lang="ts">
import InputText from 'primevue/inputtext'
import ToggleSwitch from 'primevue/toggleswitch'
import CrudDrawer from '@/components/domain/CrudDrawer.vue'
import FieldError from '@/components/domain/FieldError.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useCrudDrawer } from '@/composables/useCrudDrawer'
import type { ClienteContactoInput, ClienteInput } from '@/stores/useClientesStore'

type Model = ClienteInput & { rowVersion?: string }

const props = defineProps<{
  drawer: ReturnType<typeof useCrudDrawer<Model>>
}>()

function agregarContacto(): void {
  const contactos = props.drawer.model.value.contactos
  contactos.push({
    etiqueta: '',
    email: '',
    nombre: null,
    telefono: null,
    // The first contact is the main one; after that the user decides.
    esPrincipal: contactos.length === 0,
  })
}

function quitarContacto(indice: number): void {
  props.drawer.model.value.contactos.splice(indice, 1)
}

/** Exactly one contact can be the main one, so choosing a new one clears the previous. */
function marcarPrincipal(contacto: ClienteContactoInput): void {
  for (const otro of props.drawer.model.value.contactos) {
    otro.esPrincipal = otro === contacto
  }
}
</script>

<template>
  <CrudDrawer :drawer="drawer" title-key="Entity.Cliente">
    <label class="flex flex-col gap-1">
      <span class="text-sm">{{ $t('Clientes.Nombre') }}</span>
      <InputText
        v-model="drawer.model.value.nombre"
        :invalid="Boolean(drawer.fieldErrors.value.nombre)"
        aria-describedby="cli-nombre-error"
      />
      <FieldError id="cli-nombre-error" :message="drawer.fieldErrors.value.nombre" />
    </label>

    <div class="grid grid-cols-2 gap-3">
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Clientes.Cuit') }}</span>
        <InputText
          v-model="drawer.model.value.cuit"
          :invalid="Boolean(drawer.fieldErrors.value.cuit)"
          aria-describedby="cli-cuit-error"
        />
        <FieldError id="cli-cuit-error" :message="drawer.fieldErrors.value.cuit" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Clientes.CondicionIva') }}</span>
        <InputText v-model="drawer.model.value.condicionIva" />
      </label>
    </div>

    <label class="flex flex-col gap-1">
      <span class="text-sm">{{ $t('Clientes.Direccion') }}</span>
      <InputText v-model="drawer.model.value.direccion" />
    </label>

    <div class="grid grid-cols-2 gap-3">
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Clientes.Telefono') }}</span>
        <InputText v-model="drawer.model.value.telefono" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Clientes.Email') }}</span>
        <InputText
          v-model="drawer.model.value.email"
          :invalid="Boolean(drawer.fieldErrors.value.email)"
          aria-describedby="cli-email-error"
        />
        <FieldError id="cli-email-error" :message="drawer.fieldErrors.value.email" />
      </label>
    </div>

    <div class="flex flex-col gap-2 border-t border-border pt-3">
      <div class="flex items-center justify-between">
        <span class="text-sm font-medium">{{ $t('Clientes.Contactos') }}</span>
        <Button variant="secondary" size="sm" @click="agregarContacto()">
          <AppIcon name="plus" :size="14" />
          {{ $t('Clientes.AgregarContacto') }}
        </Button>
      </div>

      <p v-if="!drawer.model.value.contactos?.length" class="text-xs text-muted-foreground">
        {{ $t('Clientes.SinContactos') }}
      </p>

      <div
        v-for="(contacto, indice) in (drawer.model.value.contactos ?? [])"
        :key="contacto.id ?? indice"
        class="flex flex-col gap-2 rounded-md border border-border p-3"
      >
        <div class="grid grid-cols-2 gap-2">
          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">{{ $t('Clientes.Etiqueta') }}</span>
            <InputText
              v-model="contacto.etiqueta"
              :invalid="Boolean(drawer.fieldErrors.value[`contactos[${indice}].etiqueta`])"
            />
            <FieldError
              :id="`cli-contacto-${indice}-etiqueta-error`"
              :message="drawer.fieldErrors.value[`contactos[${indice}].etiqueta`]"
            />
          </label>
          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">{{ $t('Clientes.Email') }}</span>
            <InputText
              v-model="contacto.email"
              :invalid="Boolean(drawer.fieldErrors.value[`contactos[${indice}].email`])"
            />
            <FieldError
              :id="`cli-contacto-${indice}-email-error`"
              :message="drawer.fieldErrors.value[`contactos[${indice}].email`]"
            />
          </label>
        </div>
        <div class="grid grid-cols-2 gap-2">
          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">{{ $t('Clientes.Nombre') }}</span>
            <InputText v-model="contacto.nombre" />
          </label>
          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">{{ $t('Clientes.Telefono') }}</span>
            <InputText v-model="contacto.telefono" />
          </label>
        </div>
        <div class="flex items-center justify-between">
          <label class="flex items-center gap-2">
            <ToggleSwitch
              :model-value="contacto.esPrincipal"
              @update:model-value="marcarPrincipal(contacto)"
            />
            <span class="text-xs">{{ $t('Clientes.EsPrincipal') }}</span>
          </label>
          <Button variant="ghost" size="sm" @click="quitarContacto(indice)">
            <AppIcon name="trash-2" :size="14" />
          </Button>
        </div>
      </div>
    </div>
  </CrudDrawer>
</template>
