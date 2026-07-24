import api from './http'
import type { ApiResponse } from './types'

export interface NetworkAccount {
  id: string
  accountName: string
  password: string
  deviceId: string | null
  status: string
  loginUrl: string | null
  logoutUrl: string | null
  autoLogin: number
  autoLogout: number
  description: string | null
  createdAt: string
  updatedAt: string
}

export interface CreateNetworkAccountParams {
  account_name: string
  password: string
  device_id?: string
  login_url?: string
  logout_url?: string
  auto_login?: boolean
  auto_logout?: boolean
  description?: string
}

export interface UpdateNetworkAccountParams {
  account_name?: string
  password?: string
  device_id?: string
  status?: string
  login_url?: string
  logout_url?: string
  auto_login?: boolean
  auto_logout?: boolean
  description?: string
}

export async function listNetworkAccounts() {
  const response = await api.get<ApiResponse<NetworkAccount[]>>('/network-accounts')
  return response.data
}

export async function createNetworkAccount(data: CreateNetworkAccountParams) {
  const response = await api.post<ApiResponse<NetworkAccount>>('/network-accounts', data)
  return response.data
}

export async function updateNetworkAccount(id: string, data: UpdateNetworkAccountParams) {
  const response = await api.put<ApiResponse<NetworkAccount>>(`/network-accounts/${id}`, data)
  return response.data
}

export async function deleteNetworkAccount(id: string) {
  const response = await api.delete<ApiResponse<null>>(`/network-accounts/${id}`)
  return response.data
}
