import api from './http'
import type { ApiResponse } from './types'

export interface LoginRequest {
  username: string
  password: string
}

export interface LoginResponseData {
  token: string
  username: string
  displayName: string
  role: string
  expiresAt: number
}

export async function login(data: LoginRequest) {
  const response = await api.post<ApiResponse<LoginResponseData>>('/auth/login', data)
  return response.data
}
