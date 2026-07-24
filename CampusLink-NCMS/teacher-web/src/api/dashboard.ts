import api from './http'
import type { ApiResponse, DashboardData } from './types'

export async function getDashboardOverview() {
  const response = await api.get<ApiResponse<DashboardData>>('/dashboard/overview')
  return response.data
}
