import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import MoneyText from '@/components/domain/MoneyText.vue'
import { useConfigStore } from '@/stores/useConfigStore'
import { useUiStore } from '@/stores/useUiStore'

/** Privacy mode is the feature this component carries for the whole application (doc 09 §5.1). */

function withConfig(): void {
  useConfigStore().config = {
    locale: {
      simboloMoneda: '$',
      separadorMiles: '.',
      separadorDecimal: ',',
      decimalesMoneda: 2,
      decimalesPorcentaje: 2,
    },
  } as never
}

describe('MoneyText', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    withConfig()
  })

  it('formatea el importe con la configuracion regional', () => {
    const wrapper = mount(MoneyText, { props: { value: '1234.5000' } })
    expect(wrapper.text()).toBe('$ 1.234,50')
  })

  it('en modo privacidad oculta el importe y no filtra el signo', () => {
    useUiStore().privacyMode = true
    const wrapper = mount(MoneyText, { props: { value: '-1234.5000', colored: true } })
    expect(wrapper.text()).toBe('•••••')
    expect(wrapper.text()).not.toContain('-')
    // Colour would give away the sign the mask is hiding.
    expect(wrapper.classes()).not.toContain('text-money-negative')
  })

  it('un valor ausente muestra el marcador, tambien en modo privacidad', () => {
    useUiStore().privacyMode = true
    const wrapper = mount(MoneyText, { props: { value: null } })
    expect(wrapper.text()).toBe('—')
  })

  it('colorea por signo cuando se lo pide', () => {
    const negativo = mount(MoneyText, { props: { value: '-1.0000', colored: true } })
    const positivo = mount(MoneyText, { props: { value: '1.0000', colored: true } })
    const cero = mount(MoneyText, { props: { value: '0.0000', colored: true } })
    expect(negativo.classes()).toContain('text-money-negative')
    expect(positivo.classes()).toContain('text-money-positive')
    expect(cero.classes()).toContain('text-money-neutral')
  })
})
