import api from './http'
import type { ApiResponse, Device } from './types'
import type { Student } from './students'

export interface ClassInfo {
  id: string
  name: string
  createdAt: string
  updatedAt: string
}

export interface SeatAssignment {
  deviceId: string
  seatNo: string
}

export interface AutoSeatResult {
  assigned: SeatAssignment[]
  conflicts: string[]
  unmatched: string[]
}

export async function listClasses() {
  const response = await api.get<ApiResponse<ClassInfo[]>>('/classes')
  return response.data
}

export async function createClass(name: string) {
  const response = await api.post<ApiResponse<ClassInfo>>('/classes', { name })
  return response.data
}

export async function updateClass(id: string, name: string) {
  const response = await api.put<ApiResponse<ClassInfo>>(`/classes/${id}`, { name })
  return response.data
}

export async function deleteClass(id: string) {
  const response = await api.delete<ApiResponse<string>>(`/classes/${id}`)
  return response.data
}

export async function listClassStudents(classId: string) {
  const response = await api.get<ApiResponse<Student[]>>(`/classes/${classId}/students`)
  return response.data
}

export async function assignStudents(classId: string, studentIds: string[]) {
  const response = await api.post(`/classes/${classId}/students`, { studentIds })
  return response.data
}

export async function listClassDevices(classId: string) {
  const response = await api.get<ApiResponse<Device[]>>(`/classes/${classId}/devices`)
  return response.data
}

export async function assignDevices(classId: string, deviceIds: string[]) {
  const response = await api.post(`/classes/${classId}/devices`, { deviceIds })
  return response.data
}

export async function batchSetSeats(classId: string, seats: SeatAssignment[]) {
  const response = await api.post(`/classes/${classId}/seats`, { seats })
  return response.data
}

export async function autoAssignSeats(classId: string) {
  const response = await api.post<ApiResponse<AutoSeatResult>>(`/classes/${classId}/seats/auto`)
  return response.data
}

export async function resetClassPasswords(classId: string, password?: string) {
  const response = await api.post(`/classes/${classId}/passwords/reset`, { password })
  return response.data
}

export async function switchClassMode(classId: string, targetMode: string, operatorName = '管理员') {
  const response = await api.post(`/classes/${classId}/mode`, { targetMode, operatorName })
  return response.data
}
