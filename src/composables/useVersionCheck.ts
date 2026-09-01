import { ref } from 'vue'

import { useConfigStore } from '@/stores/useConfigStore'

/**
 * Checks GitHub for a newer version on startup. See `docs/18-devops.md` §3.4.
 *
 * The check has a short timeout and degrades silently: a network error just means no notification.
 * The result is a non-blocking banner with a download link, not a dialog.
 */

export interface VersionInfo {
  version: string
  url: string
}

const GITHUB_REPO = 'FittyAr/Certaro'

export function useVersionCheck() {
  const config = useConfigStore()
  const available = ref<VersionInfo | null>(null)
  const checking = ref(false)

  async function check(): Promise<void> {
    if (checking.value) return
    checking.value = true

    try {
      const current = config.info?.version ?? '0.0.0'
      const controller = new AbortController()
      const timeout = setTimeout(() => controller.abort(), 5000)

      const response = await fetch(
        `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`,
        { signal: controller.signal },
      )
      clearTimeout(timeout)

      if (!response.ok) return

      const data = await response.json()
      const tag: string = data.tag_name ?? ''
      const latest = tag.replace(/^v/, '')

      if (latest && latest !== current && isNewer(latest, current)) {
        available.value = {
          version: latest,
          url: data.html_url ?? `https://github.com/${GITHUB_REPO}/releases/latest`,
        }
      }
    } catch {
      // Network error, timeout, or parse failure: degrade silently.
    } finally {
      checking.value = false
    }
  }

  return { available, checking, check }
}

/** Simple semver comparison: returns true if `a` is newer than `b`. */
function isNewer(a: string, b: string): boolean {
  const pa = a.split('.').map(Number)
  const pb = b.split('.').map(Number)
  for (let i = 0; i < 3; i++) {
    if ((pa[i] ?? 0) > (pb[i] ?? 0)) return true
    if ((pa[i] ?? 0) < (pb[i] ?? 0)) return false
  }
  return false
}
