import api from './http'
import type {
  ApiResponse,
  AttendanceRecord,
  AttendanceStatistics,
  AttendanceQuery,
  AttendanceBoard,
} from './types'

export interface RetroactiveParams {
  student_id: string
  device_id: string
  check_in_time: string
  remarks?: string
}

export async function getAttendanceList(params?: AttendanceQuery) {
  const response = await api.get<ApiResponse<AttendanceRecord[]>>('/attendance', { params })
  return response.data
}

export async function getAttendanceStatistics(date?: string) {
  const response = await api.get<ApiResponse<AttendanceStatistics>>('/attendance/statistics', { params: { date } })
  return response.data
}

export async function getAttendanceBoard(params: { classId?: string; date?: string }) {
  const response = await api.get<ApiResponse<AttendanceBoard>>('/attendance/board', { params })
  return response.data
}

export async function retroactive(data: RetroactiveParams) {
  const response = await api.post<ApiResponse<null>>('/attendance/retroactive', data)
  return response.data
}

export interface AttendanceContext {
  deviceId: string
  mode: string
  requiresCheckin: boolean
  classId: string | null
}

export async function getAttendanceContext(deviceId: string) {
  const response = await api.get<ApiResponse<AttendanceContext>>('/attendance/context', { params: { device_id: deviceId } })
  return response.data
}

export async function exportAttendance(params?: AttendanceQuery) {
  const response = await api.get('/attendance/export', { params, responseType: 'blob' })
  return response.data as Blob
}
