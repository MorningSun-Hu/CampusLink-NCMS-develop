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

export interface AttendanceRecord {
  id: string
  device_id: string
  student_id: string | null
  record_type: string
  time_type: string
  timestamp: string
  created_at: string
}

export interface AttendanceStatistics {
  date: string
  total: number
  present: number
  absent: number
  late: number
  leave_early: number
}

export interface CheckInRequest {
  device_id: string
  student_id?: string
  timestamp?: number
}

export interface CheckInResponse {
  record_id: string
  status: string
}

export interface RetroactiveRequest {
  device_id: string
  record_type: string
  timestamp: number
}

export interface InspectionRecord {
  id: string
  device_id: string
  inspection_type: string
  item_name: string
  status: string
  description: string | null
  is_abnormal: boolean
  created_at: string
}

export interface AlertRecord {
  id: string
  inspection_id: string
  device_id: string
  alert_type: string
  description: string
  resolved: boolean
  resolved_at: string | null
  created_at: string
}

export interface PhotoRecord {
  id: string
  inspection_id: string | null
  file_name: string
  original_name: string
  content_type: string
  file_size: number
  created_at: string
}
