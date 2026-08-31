import { callCommand } from './client'
import type { AppConfig } from './types'

export interface AppInfo {
  name: string
  version: string
  environment: string
  dataDir: string
}

export function ping(message: string): Promise<string> {
  return callCommand<string>('ping', { message })
}

export function getAppInfo(): Promise<AppInfo> {
  return callCommand<AppInfo>('app_info')
}

export function getAppConfig(): Promise<AppConfig> {
  return callCommand<AppConfig>('app_config')
}

export function appIsReady(): Promise<boolean> {
  return callCommand<boolean>('app_is_ready')
}
