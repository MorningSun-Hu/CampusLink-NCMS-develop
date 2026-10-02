import api from './http'
import type { ApiResponse } from './types'

export interface AdminUser {
  id: string
  username: string
  displayName: string
  role: string
  status: string
  lastLoginAt: string | null
  createdAt: string
}

export interface CreateUserParams {
  username: string
  password: string
  displayName?: string
}

export interface UpdateUserParams {
  displayName?: string
  status?: string
}

export async function listUsers() {
  const response = await api.get<ApiResponse<AdminUser[]>>('/users')
  return response.data
}

export async function createUser(data: CreateUserParams) {
  const response = await api.post<ApiResponse<AdminUser>>('/users', data)
  return response.data
}

export async function updateUser(id: string, data: UpdateUserParams) {
  const response = await api.put<ApiResponse<AdminUser>>(`/users/${id}`, data)
  return response.data
}

export async function deleteUser(id: string) {
  const response = await api.delete<ApiResponse<string>>(`/users/${id}`)
  return response.data
}

export async function resetUserPassword(id: string, newPassword: string) {
  const response = await api.post<ApiResponse<string>>(`/users/${id}/reset-password`, { newPassword })
  return response.data
}

export async function changeOwnPassword(oldPassword: string, newPassword: string) {
  const response = await api.post<ApiResponse<string>>('/auth/change-password', { oldPassword, newPassword })
  return response.data
}
