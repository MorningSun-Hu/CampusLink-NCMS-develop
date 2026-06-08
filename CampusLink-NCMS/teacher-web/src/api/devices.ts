import axios from 'axios'
import type { ApiResponse, RegisterDeviceRequest, RegisterDeviceResponse, Device } from './types'

const api = axios.create({
  baseURL: '/api',
  timeout: 5000,
})

export async function registerDevice(data: RegisterDeviceRequest) {
  const response = await api.post<ApiResponse<RegisterDeviceResponse>>('/devices/register', data)
  return response.data
}

export async function listDevices(onlineStatus?: string) {
  const response = await api.post<ApiResponse<Device[]>>('/devices', { onlineStatus })
  return response.data
}

export async function switchDeviceMode(deviceId: string, targetMode: string, operatorName: string) {
  const response = await api.post('/devices/mode', { device_id: deviceId, target_mode: targetMode, operator_name: operatorName })
  return response.data
}

export async function healthCheck() {
  const response = await api.get<string>('/health')
  return response.data
}
