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
