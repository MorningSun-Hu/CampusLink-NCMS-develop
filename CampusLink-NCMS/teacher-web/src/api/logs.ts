import api from './http'

export interface LogEntry {
  id: string
  log_type: string | null
  device_id: string | null
  operator: string | null
  action: string | null
  detail: string | null
  created_at: string
}

export interface LogPageResponse {
  total: number
  page: number
  page_size: number
  items: LogEntry[]
}

export async function queryLogs(params?: {
  log_type?: string
  device_id?: string
  date_from?: string
  page?: number
  page_size?: number
}) {
  const resp = await api.get('/logs', { params })
  return resp.data
}

export function exportLogsUrl(params?: {
  log_type?: string
  device_id?: string
  date_from?: string
}) {
  const searchParams = new URLSearchParams()
  if (params?.log_type) searchParams.set('log_type', params.log_type)
  if (params?.device_id) searchParams.set('device_id', params.device_id)
  if (params?.date_from) searchParams.set('date_from', params.date_from)
  return `/api/logs/export?${searchParams.toString()}`
}

export interface AgentLogFileInfo {
  device_id: string
  file_name: string
  size: number
}

export async function collectStudentLogs() {
  const resp = await api.post('/logs/collect-student')
  return resp.data
}

export async function listAgentLogs() {
  const resp = await api.get('/logs/agent-files')
  return resp.data
}

export async function downloadAgentLog(deviceId: string, file: string) {
  const resp = await api.get('/logs/agent-files/download', {
    params: { device_id: deviceId, file },
    responseType: 'blob',
  })
  const url = URL.createObjectURL(resp.data)
  const a = document.createElement('a')
  a.href = url
  a.download = file
  a.click()
  URL.revokeObjectURL(url)
}
