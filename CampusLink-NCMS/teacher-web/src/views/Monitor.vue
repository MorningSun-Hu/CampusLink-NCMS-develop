<template>
  <div class="monitor">
    <div class="header">
      <h2 class="page-title">监控总览</h2>
      <el-button-group>
        <el-button :type="viewMode === 'list' ? 'primary' : 'default'" @click="viewMode = 'list'">列表视图</el-button>
        <el-button :type="viewMode === 'grid' ? 'primary' : 'default'" @click="viewMode = 'grid'">座位图</el-button>
      </el-button-group>
    </div>

    <el-card v-if="viewMode === 'list'">
      <div class="online-list">
        <div class="list-header">
          <span class="title">在线设备 ({{ devices.length }})</span>
          <el-button @click="loadDevices" :loading="loading">刷新</el-button>
        </div>
        <el-empty v-if="devices.length === 0" description="暂无在线设备" />
        <el-table v-else :data="devices" border stripe>
          <el-table-column prop="deviceCode" label="设备编号" />
          <el-table-column prop="deviceName" label="设备名称" />
          <el-table-column prop="currentMode" label="当前模式">
            <template #default="{ row }">
              <el-tag>{{ modeLabel(row.currentMode) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="lastSeenAt" label="最后心跳时间">
            <template #default="{ row }">
              <span :class="{ 'heartbeat-late': isLate(row.lastSeenAt) }">
                {{ formatTime(row.lastSeenAt) }}
              </span>
            </template>
          </el-table-column>
          <el-table-column label="状态">
            <template #default="{ row }">
              <el-tag :type="isLate(row.lastSeenAt) ? 'warning' : 'success'">
                {{ isLate(row.lastSeenAt) ? '延迟' : '在线' }}
              </el-tag>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </el-card>

    <el-card v-else>
      <div class="seat-map-header">
        <span class="seat-stats">座位数: {{ devices.length }}</span>
        <el-button @click="loadDevices" :loading="loading">刷新</el-button>
      </div>
      <el-empty v-if="devices.length === 0" description="暂无在线设备" />
      <div v-else class="seat-grid">
        <div
          v-for="device in devices"
          :key="device.id"
          class="seat-item"
          :class="{
            'seat-online': !isLate(device.lastSeenAt),
            'seat-late': isLate(device.lastSeenAt),
            'seat-locked': device.currentMode === 'locked',
            'seat-exam': device.currentMode === 'exam',
          }"
        >
          <div class="seat-index">{{ device.deviceName || device.deviceCode }}</div>
          <div class="seat-mode">{{ modeLabel(device.currentMode) }}</div>
          <div class="seat-time">{{ formatTime(device.lastSeenAt) }}</div>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { listDevices, type Device } from '@/api/devices'
import { formatDateTime, isStale } from '@/utils/time'

const devices = ref<Device[]>([])
const loading = ref(false)
const viewMode = ref<'list' | 'grid'>('list')

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

const isLate = (time: string | null) => isStale(time, 30000)

const formatTime = (time: string | null) => formatDateTime(time)

const modeLabel = (mode: string) => {
  const map: Record<string, string> = {
    open: '开放',
    teaching: '教学',
    exam: '考试',
    locked: '锁屏',
  }
  return map[mode] || mode
}

onMounted(() => {
  loadDevices()
})
</script>

<style scoped>
.monitor { padding: 20px; }
.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.page-title { font-size: 24px; font-weight: 500; color: #333; margin: 0; }
.list-header, .seat-map-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px; }
.title, .seat-stats { font-size: 16px; font-weight: 500; }
.heartbeat-late { color: #f56c6c; }

.seat-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
  gap: 12px;
}
.seat-item {
  padding: 12px;
  border-radius: 8px;
  text-align: center;
  cursor: pointer;
  transition: all 0.3s;
  border: 2px solid #e0e0e0;
}
.seat-item:hover { transform: scale(1.05); box-shadow: 0 2px 8px rgba(0,0,0,0.15); }
.seat-online { background: #f0f9eb; border-color: #67c23a; }
.seat-late { background: #fdf6ec; border-color: #e6a23c; }
.seat-locked { background: #fef0f0; border-color: #f56c6c; }
.seat-exam { background: #ecf5ff; border-color: #409eff; }
.seat-index { font-size: 12px; color: #999; margin-bottom: 4px; }
.seat-mode { font-size: 14px; font-weight: 500; color: #333; margin-bottom: 4px; }
.seat-time { font-size: 11px; color: #bbb; }
</style>
