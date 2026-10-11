import { ref, onUnmounted } from 'vue'
import { ElNotification } from 'element-plus'

interface WsMessage {
  type: string
  device_id?: string
  alert_type?: string
  description?: string
  message?: string
}

const ws = ref<WebSocket | null>(null)
const connected = ref(false)
const reconnectTimer = ref<ReturnType<typeof setTimeout> | null>(null)

let reconnectAttempts = 0
const maxReconnectAttempts = 5

function connect() {
  if (ws.value?.readyState === WebSocket.OPEN) return

  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  const url = `${protocol}//${window.location.host}/ws`
  ws.value = new WebSocket(url)

  ws.value.onopen = () => {
    connected.value = true
    reconnectAttempts = 0
  }

  ws.value.onmessage = (event) => {
    try {
      const msg: WsMessage = JSON.parse(event.data)
      handleMessage(msg)
    } catch { /* ignore non-JSON messages */ }
  }

  ws.value.onclose = () => {
    connected.value = false
    if (reconnectAttempts < maxReconnectAttempts) {
      reconnectAttempts++
      reconnectTimer.value = setTimeout(connect, 5000 * reconnectAttempts)
    }
  }

  ws.value.onerror = () => {
    ws.value?.close()
  }
}

function handleMessage(msg: WsMessage) {
  switch (msg.type) {
    case 'alert':
      ElNotification({
        title: alertTitle(msg.alert_type),
        message: msg.description || msg.message || '新告警通知',
        type: 'warning',
        duration: 5000,
      })
      break
    case 'mode_switch':
      ElNotification({
        title: '模式切换',
        message: `设备 ${msg.device_id} 已切换模式`,
        type: 'info',
        duration: 3000,
      })
      break
    case 'device_offline':
      ElNotification({
        title: '设备离线',
        message: `设备 ${msg.device_id} 已离线`,
        type: 'error',
        duration: 3000,
      })
      break
  }
}

function disconnect() {
  if (reconnectTimer.value) {
    clearTimeout(reconnectTimer.value)
    reconnectTimer.value = null
  }
  reconnectAttempts = maxReconnectAttempts
  ws.value?.close()
  ws.value = null
  connected.value = false
}

function alertTitle(alertType?: string) {
  if (alertType === 'hardware') return '硬件告警'
  if (alertType === 'network') return '网络告警'
  return '检查告警'
}

export function useWebSocket() {
  return {
    connected,
    connect,
    disconnect,
  }
}

onUnmounted(() => {
  disconnect()
})
