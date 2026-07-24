import api from './http'
import type { ApiResponse } from './types'

export interface RepairOrder {
  id: string
  deviceId: string
  deviceName: string
  reporter: string
  issueType: string
  description: string
  status: string
  assignedTo: string
  createdAt: string
  resolvedAt: string | null
}

export interface RepairListResponse {
  orders: RepairOrder[]
  total: number
  page: number
  pageSize: number
}

export async function listRepairs(params: { page?: number; pageSize?: number; status?: string }) {
  const response = await api.get<ApiResponse<RepairListResponse>>('/repair-orders', { params })
  return response.data
}

export async function createRepair(data: { deviceId: string; deviceName: string; reporter: string; issueType: string; description: string }) {
  return api.post<ApiResponse<RepairOrder>>('/repair-orders', data)
}

export async function updateRepair(id: string, data: { status?: string; assignedTo?: string }) {
  return api.put<ApiResponse<RepairOrder>>(`/repair-orders/${id}`, data)
}

export async function deleteRepair(id: string) {
  return api.delete<ApiResponse<string>>(`/repair-orders/${id}`)
}
