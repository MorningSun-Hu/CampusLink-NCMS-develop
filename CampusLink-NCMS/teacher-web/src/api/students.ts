import api from './http'
import type { ApiResponse } from './types'

export interface Student {
  id: string
  studentNo: string
  name: string
  seatNo: string | null
  status: string
  classId: string | null
  className: string | null
  passwordSet: boolean
  createdAt: string
  updatedAt: string
}

export interface StudentListResponse {
  students: Student[]
  total: number
  page: number
  pageSize: number
}

export interface CreateStudentRequest {
  studentNo?: string
  name: string
  password?: string
  seatNo?: string
  classId?: string
}

export interface UpdateStudentRequest {
  studentNo?: string
  name?: string
  password?: string
  seatNo?: string
  status?: string
  classId?: string
}

export async function listStudents(params: { page?: number; pageSize?: number; keyword?: string; classId?: string }) {
  const response = await api.get<ApiResponse<StudentListResponse>>('/students', { params })
  return response.data
}

export async function getStudent(id: string) {
  const response = await api.get<ApiResponse<Student>>(`/students/${id}`)
  return response.data
}

export async function createStudent(data: CreateStudentRequest) {
  const response = await api.post<ApiResponse<Student>>('/students', data)
  return response.data
}

export async function updateStudent(id: string, data: UpdateStudentRequest) {
  const response = await api.put<ApiResponse<Student>>(`/students/${id}`, data)
  return response.data
}

export async function deleteStudent(id: string) {
  const response = await api.delete<ApiResponse<string>>(`/students/${id}`)
  return response.data
}

export async function resetStudentPassword(id: string, password?: string) {
  const response = await api.post<ApiResponse<string>>(`/students/${id}/reset-password`, { password })
  return response.data
}

export async function importStudents(file: File) {
  const formData = new FormData()
  formData.append('file', file)
  const response = await api.post<ApiResponse<{ success: boolean; imported: number }>>('/students/import', formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
  })
  return response.data
}

export function getExportUrl() {
  return '/api/students/export'
}
