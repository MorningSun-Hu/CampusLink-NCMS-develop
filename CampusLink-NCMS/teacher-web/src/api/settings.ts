import axios from 'axios'
import type { ApiResponse } from './types'

export interface LockPasswordStatus {
  configured: boolean
}

const api = axios.create({ baseURL: '/api', timeout: 5000 })

export async function getLockPasswordStatus() {
  const response = await api.get<ApiResponse<LockPasswordStatus>>('/settings/lock-password')
  return response.data
}

export async function updateLockPassword(password: string) {
  const response = await api.put<ApiResponse<string>>('/settings/lock-password', { password })
  return response.data
}
