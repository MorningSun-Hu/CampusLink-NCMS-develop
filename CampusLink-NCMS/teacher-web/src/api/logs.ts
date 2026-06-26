import axios from 'axios'

const api = axios.create({
  baseURL: '/api',
  timeout: 10000,
})

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
