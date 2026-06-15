import axios from 'axios'
import type { ApiResponse, InspectionRecord, AlertRecord } from './types'

const api = axios.create({
  baseURL: '/api',
  timeout: 10000,
})

export async function getInspectionList(params?: { inspection_type?: string; device_id?: string }) {
  const response = await api.get<ApiResponse<InspectionRecord[]>>('/inspection', { params })
  return response.data
}

export async function getAlertList(params?: { resolved?: string }) {
  const response = await api.get<ApiResponse<AlertRecord[]>>('/alerts', { params })
  return response.data
}

export async function resolveAlert(alertId: string) {
  const response = await api.post<ApiResponse<null>>(`/alerts/${alertId}/resolve`)
  return response.data
}
