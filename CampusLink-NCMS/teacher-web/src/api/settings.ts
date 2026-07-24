import api from './http'
import type { ApiResponse } from './types'

export interface LockPasswordStatus {
  configured: boolean
}

export interface ScheduleConfig {
  enabled: boolean
  time: string | null
  targetMode: string | null
}

export async function getLockPasswordStatus() {
  const response = await api.get<ApiResponse<LockPasswordStatus>>('/settings/lock-password')
  return response.data
}

export async function updateLockPassword(password: string) {
  const response = await api.put<ApiResponse<string>>('/settings/lock-password', { password })
  return response.data
}

export async function getScheduleConfig() {
  const response = await api.get<ApiResponse<ScheduleConfig>>('/settings/schedule')
  return response.data
}

export async function updateScheduleConfig(config: ScheduleConfig) {
  const response = await api.put<ApiResponse<string>>('/settings/schedule', {
    enabled: config.enabled,
    time: config.time,
    target_mode: config.targetMode,
  })
  return response.data
}
