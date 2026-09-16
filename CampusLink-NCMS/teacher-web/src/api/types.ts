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
  classId: string | null
  className: string | null
  seatNo: string | null
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
  pendingRepairCount: number
  pendingWhitelistCount: number
}

export interface AttendanceStatistics {
  total: number
  present: number
  late: number
  absent: number
  leave: number
  records: AttendanceRecord[]
}

export interface AttendanceRecord {
  id: string
  student_id: string | null
  student_no: string | null
  student_name: string | null
  device_id: string
  check_in_time: string
  check_out_time: string | null
  status: string
  remarks: string | null
  seat_no: string | null
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
