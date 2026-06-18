import axios from 'axios'
import type { ApiResponse, AttendanceRecord, AttendanceStatistics } from './types'

const api = axios.create({
  baseURL: '/api',
  timeout: 10000,
})

export interface RetroactiveParams {
  student_id: string
  device_id: string
  check_in_time: string
  remarks?: string
}

export async function getAttendanceList() {
  const response = await api.get<ApiResponse<AttendanceRecord[]>>('/attendance')
  return response.data
}

export async function getAttendanceStatistics(date?: string) {
  const response = await api.get<ApiResponse<AttendanceStatistics>>('/attendance/statistics', { params: { date } })
  return response.data
}

export async function retroactive(data: RetroactiveParams) {
  const response = await api.post<ApiResponse<null>>('/attendance/retroactive', data)
  return response.data
}
