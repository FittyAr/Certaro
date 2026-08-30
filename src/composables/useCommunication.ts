import { computed } from 'vue'

import type { EmailClient } from '@/api/types'
import { useConfigStore } from '@/stores/useConfigStore'

/**
 * Email and WhatsApp deep links. See `docs/13-servicios-externos-y-archivos.md` §7.
 *
 * The application does not send anything: it opens the system client. The phone is normalised
 * before building the WhatsApp URL, because a formatted number produces a broken link.
 */

/**
 * Strips formatting characters and adds the country code if missing.
 *
 * `(011) 4567-8901` with code `54` becomes `5401145678901`. The result is digits only, which is
 * what WhatsApp expects.
 */
export function normalizarTelefono(telefono: string, codigoPais: string): string {
  const digits = telefono.replace(/\D/g, '')
  if (!digits) return ''
  if (digits.startsWith(codigoPais)) return digits
  return codigoPais + digits
}

export function useCommunication() {
  const config = useConfigStore()

  const codigoPais = computed(() => config.config?.communication.codigoPais ?? '54')

  function emailUrl(destinatarios: string[], asunto: string, cuerpo: string): string {
    const cliente: EmailClient = config.config?.communication.emailCliente ?? 'systemDefault'
    const to = destinatarios.join(',')
    const encodedSubject = encodeURIComponent(asunto)
    const encodedBody = encodeURIComponent(cuerpo)

    if (cliente === 'systemDefault') {
      return `mailto:${to}?subject=${encodedSubject}&body=${encodedBody}`
    }

    const urls: Record<EmailClient, string> = {
      systemDefault: '',
      gmail: config.config?.communication.gmailUrl ?? 'https://mail.google.com/mail/u/0/?view=cm&fs=1&to={email}',
      outlook: config.config?.communication.outlookUrl ?? 'https://outlook.live.com/mail/0/deeplink/compose?to={email}',
      yahoo: config.config?.communication.yahooUrl ?? 'https://mail.yahoo.com/d/compose-message?to={email}',
    }

    const base = urls[cliente].replace('{email}', encodeURIComponent(to))
    return `${base}&subject=${encodedSubject}&body=${encodedBody}`
  }

  function whatsappUrl(telefono: string, mensaje: string): string | null {
    const normalizado = normalizarTelefono(telefono, codigoPais.value)
    if (!normalizado) return null
    return `https://api.whatsapp.com/send?phone=${normalizado}&text=${encodeURIComponent(mensaje)}`
  }

  return { emailUrl, whatsappUrl, normalizarTelefono, codigoPais }
}
