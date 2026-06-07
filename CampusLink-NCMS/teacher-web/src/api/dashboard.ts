import axios from 'axios'
import type { ApiResponse, DashboardData } from './types'

const api = axios.create({
  baseURL: '/api',
  timeout: 5000,
})

export async function getDashboardOverview() {
  const response = await api.get<ApiResponse<DashboardData>>('/dashboard/overview')
  return response.data
}
