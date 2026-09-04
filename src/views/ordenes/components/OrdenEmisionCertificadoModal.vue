<script setup lang="ts">
import Dialog from 'primevue/dialog'
import Textarea from 'primevue/textarea'
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import DateInput from '@/components/domain/DateInput.vue'
import DecimalInput from '@/components/domain/DecimalInput.vue'
import FieldError from '@/components/domain/FieldError.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PercentBar from '@/components/domain/PercentBar.vue'
import { Button } from '@/components/ui/button'
import { useApiError } from '@/composables/useApiError'
import {
  useCertificadosStore,
  type CertificadoBorradorItem,
} from '@/stores/useCertificadosStore'

const props = defineProps<{
  visible: boolean
  ordenId: string
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const router = useRouter()
const { notify, fieldErrors } = useApiError()
const certificados = useCertificadosStore()

const emitiendo = ref(false)
const erroresEmision = ref<Record<string, string>>({})
const fechaEmision = ref(new Date().toISOString().slice(0, 10))
const observacionesEmision = ref<string | null>(null)
const avances = ref<Record<string, string>>({})

const borrador = computed(() => certificados.borrador)

watch(
  () => props.visible,
  async (val) => {
    if (!val) return
    erroresEmision.value = {}
    fechaEmision.value = new Date().toISOString().slice(0, 10)
    observacionesEmision.value = null
    try {
      const draft = await certificados.preparar(props.ordenId)
      avances.value = Object.fromEntries(
        draft.items.map((i) => [i.ordenTrabajoItemId, i.porcentajeActual]),
      )
    } catch (e) {
      notify(e)
    }
  }
)

function acumuladoDe(item: CertificadoBorradorItem): string {
  const pedido = Number(avances.value[item.ordenTrabajoItemId] ?? '0')
  return (Number(item.porcentajeAcumuladoAnterior) + pedido).toFixed(4)
}

function excede(item: CertificadoBorradorItem): boolean {
  return Number(acumuladoDe(item)) > 100
}

function subtotalDe(item: CertificadoBorradorItem): string {
  const pedido = Number(avances.value[item.ordenTrabajoItemId] ?? '0')
  return ((Number(item.base) * pedido) / 100).toFixed(4)
}

const totalCertificadoAEmitir = computed(() => {
  const draft = borrador.value
  if (!draft) return '0.0000'
  return draft.items.reduce((acc, i) => acc + Number(subtotalDe(i)), 0).toFixed(4)
})

const ajusteUocraAEmitir = computed(() => {
  const draft = borrador.value
  if (!draft) return '0.0000'
  const sub = Number(totalCertificadoAEmitir.value)
  const pct = Number(draft.ajusteUocraPorcentaje)
  return ((sub * pct) / 100).toFixed(4)
})

const otrosDescuentosAEmitir = computed(() => {
  const draft = borrador.value
  if (!draft) return '0.0000'
  const sub = Number(totalCertificadoAEmitir.value)
  const totalOrden = draft.items.reduce((acc, i) => acc + Number(i.base), 0)
  const restante = Number(draft.otrosDescuentos)
  if (totalOrden <= 0 || restante <= 0) return '0.0000'
  const prop = (sub / totalOrden) * restante
  return Math.min(prop, restante).toFixed(4)
})

const totalNetoAEmitir = computed(() => {
  const bruto = Number(totalCertificadoAEmitir.value)
  const uocra = Number(ajusteUocraAEmitir.value)
  const otros = Number(otrosDescuentosAEmitir.value)
  return Math.max(0, bruto + uocra - otros).toFixed(4)
})

const hayExcedidos = computed(() => borrador.value?.items.some(excede) ?? false)
const hayAvance = computed(() => Object.values(avances.value).some((v) => Number(v) > 0))

async function emitir(): Promise<void> {
  if (emitiendo.value || !borrador.value) return
  emitiendo.value = true
  erroresEmision.value = {}
  try {
    const emitido = await certificados.create({
      ordenTrabajoId: props.ordenId,
      fecha: fechaEmision.value,
      observaciones: observacionesEmision.value,
      items: borrador.value.items.map((i) => ({
        ordenTrabajoItemId: i.ordenTrabajoItemId,
        porcentajeActual: avances.value[i.ordenTrabajoItemId] ?? '0.0000',
      })),
    })
    emit('update:visible', false)
    await router.push({ name: 'certificado-detalle', params: { certificadoId: emitido.id } })
  } catch (e) {
    const api = notify(e)
    if (api.code === 'VALIDATION') erroresEmision.value = fieldErrors(api)
  } finally {
    emitiendo.value = false
  }
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    maximizable
    :header="$t('Certificados.Emitir')"
    class="w-full max-w-5xl"
    @update:visible="emit('update:visible', $event)"
  >
    <div v-if="borrador" class="space-y-4">
      <p class="text-sm text-muted-foreground">
        {{
          $t('Certificados.EmitirSubtitulo', {
            numero: borrador.numeroSugerido,
            orden: borrador.ordenTitulo,
          })
        }}
      </p>

      <label class="flex max-w-xs flex-col gap-1">
        <span class="text-sm">{{ $t('Certificados.Fecha') }}</span>
        <DateInput v-model="fechaEmision" :invalid="Boolean(erroresEmision.fecha)" />
        <FieldError id="cert-fecha-error" :message="erroresEmision.fecha" />
      </label>

      <div class="space-y-2">
        <div
          v-for="(item, index) in borrador.items"
          :key="item.ordenTrabajoItemId"
          class="grid grid-cols-12 items-end gap-2 rounded-md border p-2"
          :class="excede(item) ? 'border-state-overdue' : 'border-border'"
        >
          <div class="col-span-12 md:col-span-4">
            <p class="text-sm">{{ item.descripcion }}</p>
            <p class="text-xs text-muted-foreground">
              {{ item.cantidad }} {{ item.unidad }} ·
              <MoneyText :value="item.precioUnitario" />
            </p>
          </div>
          <div class="col-span-6 md:col-span-2">
            <span class="text-xs text-muted-foreground">
              {{ $t('Certificados.AcumuladoAnterior') }}
            </span>
            <PercentBar :value="item.porcentajeAcumuladoAnterior" />
          </div>
          <label class="col-span-6 flex flex-col gap-1 md:col-span-2">
            <span class="text-xs text-muted-foreground">{{
              $t('Certificados.AvanceAhora')
            }}</span>
            <DecimalInput
              :model-value="avances[item.ordenTrabajoItemId] ?? '0.0000'"
              :min="0"
              :max="100"
              suffix=" %"
              :invalid="
                excede(item) || Boolean(erroresEmision[`items[${index}].porcentajeActual`])
              "
              @update:model-value="avances[item.ordenTrabajoItemId] = $event"
            />
            <FieldError
              :id="`cert-item-${index}-error`"
              :message="erroresEmision[`items[${index}].porcentajeActual`]"
            />
          </label>
          <div class="col-span-6 md:col-span-2">
            <span class="text-xs text-muted-foreground">{{ $t('Certificados.Acumulado') }}</span>
            <PercentBar :value="acumuladoDe(item)" />
          </div>
          <div class="col-span-6 text-right md:col-span-2">
            <span class="text-xs text-muted-foreground">{{ $t('Certificados.Subtotal') }}</span>
            <p><MoneyText :value="subtotalDe(item)" /></p>
          </div>
        </div>
      </div>

      <p v-if="hayExcedidos" class="text-sm text-state-overdue">
        {{ $t('Certificados.AcumuladoExcedidoAviso') }}
      </p>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Certificados.Observaciones') }}</span>
        <Textarea v-model="observacionesEmision" rows="2" auto-resize />
      </label>

      <dl class="space-y-1.5 rounded-md border border-border bg-surface-card p-3 text-sm">
        <div class="flex justify-between">
          <dt class="text-muted-foreground">{{ $t('Certificados.TotalCertificado') }}</dt>
          <dd><MoneyText :value="totalCertificadoAEmitir" /></dd>
        </div>
        <div v-if="Number(ajusteUocraAEmitir) > 0" class="flex justify-between text-muted-foreground">
          <dt>{{ $t('Certificados.AjusteUocra') }} (+{{ borrador.ajusteUocraPorcentaje }}%)</dt>
          <dd>+ <MoneyText :value="ajusteUocraAEmitir" /></dd>
        </div>
        <div v-if="Number(otrosDescuentosAEmitir) > 0" class="flex justify-between text-muted-foreground">
          <dt>{{ $t('Certificados.OtrosDescuentos') }}</dt>
          <dd>- <MoneyText :value="otrosDescuentosAEmitir" /></dd>
        </div>
        <div class="flex justify-between border-t border-border pt-1 font-semibold">
          <dt>{{ $t('Certificados.TotalNeto') }}</dt>
          <dd><MoneyText :value="totalNetoAEmitir" colored /></dd>
        </div>
      </dl>
    </div>

    <template #footer>
      <Button variant="outline" :disabled="emitiendo" @click="emit('update:visible', false)">
        {{ $t('General.Cancel') }}
      </Button>
      <Button :disabled="emitiendo || hayExcedidos || !hayAvance" @click="emitir()">
        {{ $t('Certificados.Emitir') }}
      </Button>
    </template>
  </Dialog>
</template>
