<template>
  <div class="monitor">
    <h2 class="page-title">监控总览</h2>
    <el-card>
      <div class="online-list">
        <div class="list-header">
          <span class="title">在线设备 ({{ devices.length }})</span>
          <el-button @click="loadDevices" :loading="loading">刷新</el-button>
        </div>
        <el-empty v-if="devices.length === 0" description="暂无在线设备" />
        <el-table v-else :data="devices" border stripe>
          <el-table-column prop="deviceCode" label="设备编号" />
          <el-table-column prop="deviceName" label="设备名称" />
          <el-table-column prop="currentMode" label="当前模式" />
          <el-table-column prop="lastSeenAt" label="最后心跳时间">
            <template #default="{ row }">
              <span :class="{ 'heartbeat-late': isLate(row.lastSeenAt) }">
                {{ formatTime(row.lastSeenAt) }}
              </span>
            </template>
          </el-table-column>
          <el-table-column label="状态">
            <template #default>
              <el-tag type="success">在线</el-tag>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { listDevices, type Device } from '@/api/devices'

const devices = ref<Device[]>([])
const loading = ref(false)

const loadDevices = async () => {
  loading.value = true
  try {
    const result = await listDevices('online')
    if (result.data) {
      devices.value = result.data
    }
  } catch (error) {
    console.error('Load online devices failed:', error)
  } finally {
    loading.value = false
  }
}

const isLate = (time: string | null) => {
  if (!time) return true
  const lastSeen = new Date(time).getTime()
  const now = new Date().getTime()
  return (now - lastSeen) > 30000 // 超过 30 秒视为延迟
}

const formatTime = (time: string | null) => {
  if (!time) return '-'
  const date = new Date(time)
  return date.toLocaleTimeString('zh-CN')
}

onMounted(() => {
  loadDevices()
  // 每 15 秒刷新一次
  const interval = setInterval(loadDevices, 15000)
  return () => clearInterval(interval)
})
</script>

<style scoped>
.monitor {
  padding: 20px;
}

.page-title {
  font-size: 24px;
  font-weight: 500;
  margin-bottom: 20px;
  color: #333;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.title {
  font-size: 16px;
  font-weight: 500;
}

.heartbeat-late {
  color: #f56c6c;
}
</style>
