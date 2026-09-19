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
  studentId: string | null
  studentNo: string | null
  studentName: string | null
  deviceId: string
  deviceName: string | null
  classId: string | null
  className: string | null
  checkInTime: string
  checkOutTime: string | null
  status: string
  remarks: string | null
  seatNo: string | null
  usageRecordId?: string | null
}

export interface AttendanceQuery {
  classId?: string
  startDate?: string
  endDate?: string
  studentKeyword?: string
  status?: string
}

export interface BoardStudent {
  studentId: string
  studentNo: string
  studentName: string
  seatNo: string | null
  deviceId: string | null
  deviceName: string | null
  checkInTime: string | null
  status: string
}

export interface AttendanceBoard {
  classId: string | null
  className: string | null
  date: string
  totalStudents: number
  presentCount: number
  absentCount: number
  attendanceRate: number
  present: BoardStudent[]
  absent: BoardStudent[]
}

export interface UsageRecord {
  id: string
  deviceId: string
  deviceName: string | null
  classId: string | null
  className: string | null
  seatNo: string | null
  studentId: string | null
  studentNo: string | null
  studentName: string | null
  userName: string
  mode: string
  startTime: string
  endTime: string | null
  durationSeconds: number | null
  inspectionOk: boolean
  inspectionSummary: string | null
}

export interface UsageQuery {
  deviceId?: string
  classId?: string
  studentKeyword?: string
  startDate?: string
  endDate?: string
}

export interface InspectionRecord {
  id: string
  device_id: string
  device_name: string | null
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
