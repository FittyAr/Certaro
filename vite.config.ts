import { execSync } from 'node:child_process'
import { existsSync, readFileSync, writeFileSync } from 'node:fs'
import { fileURLToPath, URL } from 'node:url'

import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

const LICENSE_ENV = 'VITE_PRIMEUI_LICENSE_KEY'

/**
 * Resolves the PrimeUI license key with zero manual setup, in this order:
 *
 * 1. Real environment variable — what CI uses (`vars.PRIMEUI_KEY || secrets.PRIMEUI_KEY`).
 * 2. `.env.local` — auto-loaded by Vite and gitignored; used as the local cache.
 * 3. GitHub repository *variable* `PRIMEUI_KEY` via the `gh` CLI. Variables, unlike
 *    secrets, are readable back (`gh secret get` does not exist), so any machine
 *    with `gh auth login` picks the key up automatically — one
 *    `gh variable set PRIMEUI_KEY --body "<key>"` covers every workstation.
 *
 * The fetched value is cached into `.env.local`; delete that file to re-fetch
 * (for example after renewing the license).
 */
function ensurePrimeUiLicense(): void {
  if (process.env[LICENSE_ENV]?.trim()) return

  const envLocalPath = fileURLToPath(new URL('.env.local', import.meta.url))
  if (existsSync(envLocalPath) && readFileSync(envLocalPath, 'utf8').includes(`${LICENSE_ENV}=`)) {
    return
  }

  try {
    const key = execSync('gh variable get PRIMEUI_KEY', {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
      timeout: 10_000,
    })?.trim()
    if (key) {
      writeFileSync(
        envLocalPath,
        `# Managed by vite.config.ts — delete this file to re-fetch the key.\n${LICENSE_ENV}=${key}\n`,
      )
    }
  } catch {
    // gh missing, unauthenticated, or the variable is absent: PrimeVue will show its
    // development-time license notice, which is the documented behaviour for a missing key.
  }
}

ensurePrimeUiLicense()

// Tauri drives the dev server, so the port is fixed and failures must be loud: silently moving to
// another port would leave the desktop window pointing at nothing.
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  clearScreen: false,
  server: {
    host: '0.0.0.0',
    port: 1420,
    strictPort: true,
    allowedHosts: true,
    watch: {
      ignored: ['**/src-tauri/**', '**/target/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: 'esnext',
    sourcemap: true,
  },
})
