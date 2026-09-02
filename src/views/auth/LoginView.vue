<script setup lang="ts">
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/useAuthStore'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const email = ref('')
const password = ref('')
const totpCode = ref('')
const requiresTotp = ref(false)
const errorMsg = ref<string | null>(null)
const loading = ref(false)

async function handleSubmit() {
  errorMsg.value = null
  if (!email.value.trim()) {
    errorMsg.value = 'Por favor ingresa tu correo electrónico'
    return
  }
  if (!password.value) {
    errorMsg.value = 'Por favor ingresa tu contraseña'
    return
  }
  if (requiresTotp.value && !totpCode.value.trim()) {
    errorMsg.value = 'Por favor ingresa el código de autenticación 2FA'
    return
  }

  loading.value = true
  try {
    const res = await authStore.login({
      email: email.value.trim(),
      password: password.value,
      totpCode: requiresTotp.value ? totpCode.value.trim() : null,
    })

    if (res.requiere2fa && !requiresTotp.value) {
      requiresTotp.value = true
      loading.value = false
      return
    }

    const redirect = (route.query.redirect as string) || '/'
    await router.push(redirect)
  } catch (err: any) {
    console.error('Error logging in:', err)
    if (err?.code === 'Validation.Auth.2faCodeRequired' || err?.messageKey === 'Validation.Auth.2faCodeRequired') {
      requiresTotp.value = true
      errorMsg.value = 'Esta cuenta requiere autenticación de dos factores (2FA)'
    } else if (err?.code === 'Validation.Auth.InvalidCredentials' || err?.messageKey === 'Validation.Auth.InvalidCredentials') {
      errorMsg.value = 'Credenciales inválidas. Verifica tu correo y contraseña'
    } else if (err?.code === 'Validation.Auth.Invalid2faCode' || err?.messageKey === 'Validation.Auth.Invalid2faCode') {
      errorMsg.value = 'Código 2FA incorrecto o expirado'
    } else if (err?.code === 'Validation.Auth.UserLocked' || err?.messageKey === 'Validation.Auth.UserLocked') {
      errorMsg.value = 'Cuenta bloqueada temporalmente por múltiples intentos fallidos'
    } else {
      errorMsg.value = err?.message || 'Error al iniciar sesión. Intenta nuevamente'
    }
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-background px-4">
    <div class="w-full max-w-md bg-surface-card border border-border rounded-2xl shadow-2xl p-8 backdrop-blur-sm">
      <div class="flex flex-col items-center mb-8 text-center">
        <div class="w-16 h-16 rounded-2xl bg-primary/10 border border-primary/20 flex items-center justify-center mb-4 text-primary">
          <AppIcon name="ShieldCheck" class="w-8 h-8" />
        </div>
        <h1 class="text-2xl font-bold text-foreground tracking-tight">Certaro Enterprise</h1>
        <p class="text-sm text-muted-foreground mt-1">Inicia sesión en tu cuenta</p>
      </div>

      <div
        v-if="errorMsg"
        class="mb-6 p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-sm flex items-center gap-2"
      >
        <AppIcon name="AlertCircle" class="w-4 h-4 shrink-0" />
        <span>{{ errorMsg }}</span>
      </div>

      <form @submit.prevent="handleSubmit" class="space-y-4">
        <div>
          <label class="block text-xs font-medium text-muted-foreground mb-1.5">Correo Electrónico</label>
          <div class="relative">
            <input
              v-model="email"
              type="email"
              autocomplete="email"
              required
              :disabled="loading || requiresTotp"
              placeholder="admin@certaro.local"
              class="w-full rounded-lg bg-background border border-input px-3.5 py-2.5 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring disabled:opacity-50"
            />
          </div>
        </div>

        <div>
          <label class="block text-xs font-medium text-muted-foreground mb-1.5">Contraseña</label>
          <div class="relative">
            <input
              v-model="password"
              type="password"
              autocomplete="current-password"
              required
              :disabled="loading || requiresTotp"
              placeholder="••••••••"
              class="w-full rounded-lg bg-background border border-input px-3.5 py-2.5 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring disabled:opacity-50"
            />
          </div>
        </div>

        <div v-if="requiresTotp" class="pt-2">
          <label class="block text-xs font-medium text-warning mb-1.5">Código de Autenticación 2FA (6 dígitos)</label>
          <input
            v-model="totpCode"
            type="text"
            inputmode="numeric"
            maxlength="6"
            required
            :disabled="loading"
            placeholder="123456"
            class="w-full rounded-lg bg-background border border-warning/50 px-3.5 py-2.5 text-sm text-foreground text-center font-mono tracking-widest text-lg placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-warning"
          />
          <p class="text-xs text-muted-foreground mt-1">Ingresa el código temporal de Google Authenticator o Authy.</p>
        </div>

        <div class="pt-2">
          <Button
            type="submit"
            :disabled="loading"
            class="w-full py-2.5 font-medium rounded-lg transition-colors flex items-center justify-center gap-2"
          >
            <AppIcon v-if="loading" name="Loader2" class="w-4 h-4 animate-spin" />
            <span>{{ requiresTotp ? 'Verificar y Acceder' : 'Iniciar Sesión' }}</span>
          </Button>
        </div>
      </form>

      <div class="mt-8 pt-6 border-t border-border text-center">
        <p class="text-xs text-muted-foreground">
          Modo Multi-Base de Datos • Certaro v2
        </p>
      </div>
    </div>
  </div>
</template>
