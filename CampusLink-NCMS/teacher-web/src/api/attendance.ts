import axios from 'axios'
import type { ApiResponse, CheckInRequest, CheckInResponse, RetroactiveRequest, AttendanceRecord, AttendanceStatistics } from './types'

const api = axios.create({
  baseURL: '/api',
  timeout: 10000,
})

export async function checkIn(data: CheckInRequest) {
  const response = await api.post<ApiResponse<CheckInResponse>>('/attendance/check-in', data)
  return response.data
}

export async function retroactive(data: RetroactiveRequest) {
  const response = await api.post<ApiResponse<null>>('/attendance/retroactive', data)
  return response.data
}

export async function getAttendanceList(params?: { record_type?: string; time_type?: string; date?: string }) {
  const response = await api.get<ApiResponse<AttendanceRecord[]>>('/attendance', { params })
  return response.data
}

export async function getAttendanceStatistics(date?: string) {
  const response = await api.get<ApiResponse<AttendanceStatistics[]>>('/attendance/statistics', { params: { date } })
  return response.data
}
