export interface Device {
  id: string
  deviceCode: string
  deviceName: string
  hostname: string
  ipAddress: string
  registerStatus: string
  onlineStatus: string
  currentMode: string
  lastSeenAt: string | null
}

export interface RegisterDeviceRequest {
  deviceCode: string
  machineFingerprint: string
  hostname: string
  ipAddress: string
  macAddress: string
  agentVersion: string
}

export interface RegisterDeviceResponse {
  deviceId: string
  teacherFingerprint: string
  initialMode: string
  heartbeatIntervalSeconds: number
}

export interface ApiResponse<T> {
  code: number
  message: string
  data: T | null
}

export interface DashboardData {
  studentCount: number
  registeredDeviceCount: number
  onlineDeviceCount: number
  offlineDeviceCount: number
}
