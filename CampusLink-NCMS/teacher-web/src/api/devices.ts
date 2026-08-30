import api from './http'
import type { ApiResponse, RegisterDeviceRequest, RegisterDeviceResponse, Device } from './types'

export type { Device } from './types'

export async function registerDevice(data: RegisterDeviceRequest) {
  const response = await api.post<ApiResponse<RegisterDeviceResponse>>('/devices/register', data)
  return response.data
}

export async function listDevices(onlineStatus?: string) {
  const response = await api.post<ApiResponse<Device[]>>('/devices', { onlineStatus })
  return response.data
}

export async function switchDeviceMode(deviceId: string, targetMode: string, operatorName: string) {
  const response = await api.post(`/devices/${deviceId}/mode`, { device_id: deviceId, target_mode: targetMode, operator_name: operatorName })
  return response.data
}

export async function lockDevice(deviceId: string, reason?: string) {
  const response = await api.post(`/devices/${deviceId}/lock`, { device_id: deviceId, reason })
  return response.data
}

export async function unlockDevice(deviceId: string) {
  const response = await api.post(`/devices/${deviceId}/unlock`, { device_id: deviceId })
  return response.data
}

export async function healthCheck() {
  const response = await api.get<string>('/health')
  return response.data
}
