// The Tailwind bridge for PrimeVue ships no types.
declare module 'tailwindcss-primeui' {
  import type { PluginCreator } from 'tailwindcss/types/config'
  const plugin: PluginCreator
  export default plugin
}
