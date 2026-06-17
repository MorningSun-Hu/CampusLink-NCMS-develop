import axios from 'axios'

const api = axios.create({
  baseURL: '/api',
  timeout: 10000,
})

export interface HardwareInfo {
  id: string
  device_id: string
  cpu_model: string | null
  cpu_cores: number | null
  total_memory_bytes: number | null
  disk_info: string | null
  mac_addresses: string | null
  gpu_info: string | null
  os_version: string | null
  hostname: string | null
  created_at: string
}

export interface HardwareChange {
  id: string
  device_id: string
  change_type: string
  field_name: string
  old_value: string | null
  new_value: string | null
  detected_at: string
}

export async function getHardwareSnapshot(deviceId: string) {
  const resp = await api.get('/hardware/snapshot', { params: { device_id: deviceId } })
  return resp.data
}

export async function getHardwareChanges(params?: { device_id?: string }) {
  const resp = await api.get('/hardware/changes', { params })
  return resp.data
}
