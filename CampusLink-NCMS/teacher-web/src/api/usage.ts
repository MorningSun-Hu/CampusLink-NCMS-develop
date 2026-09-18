import api from './http'
import type { ApiResponse, UsageQuery, UsageRecord } from './types'

export async function getUsageList(params?: UsageQuery) {
  const response = await api.get<ApiResponse<UsageRecord[]>>('/usage', { params })
  return response.data
}

export async function endUsageSession(id: string) {
  const response = await api.post<ApiResponse<UsageRecord>>(`/usage/${id}/end`)
  return response.data
}

export async function exportUsage(params?: UsageQuery) {
  const response = await api.get('/usage/export', { params, responseType: 'blob' })
  return response.data as Blob
}
