<template>
  <div class="devices">
    <div class="header">
      <h2 class="page-title">设备管理</h2>
      <el-button type="primary" @click="loadDevices">刷新</el-button>
    </div>
    <div class="filters">
      <el-select v-model="onlineStatus" placeholder="在线状态" clearable @change="loadDevices">
        <el-option label="在线" value="online" />
        <el-option label="离线" value="offline" />
      </el-select>
    </div>
    <el-table :data="devices" v-loading="loading" border stripe>
      <el-table-column prop="deviceCode" label="设备编号" />
      <el-table-column prop="deviceName" label="设备名称" />
      <el-table-column prop="ipAddress" label="IP 地址" />
      <el-table-column prop="registerStatus" label="注册状态">
        <template #default="{ row }">
          <el-tag :type="row.registerStatus === 'verified' ? 'success' : 'warning'">
            {{ row.registerStatus }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="onlineStatus" label="在线状态">
        <template #default="{ row }">
          <el-tag :type="row.onlineStatus === 'online' ? 'success' : 'danger'">
            {{ row.onlineStatus }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="currentMode" label="当前模式" />
      <el-table-column prop="lastSeenAt" label="最后心跳时间">
        <template #default="{ row }">
          {{ row.lastSeenAt || '-' }}
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { listDevices, type Device } from '@/api/devices'

const devices = ref<Device[]>([])
const loading = ref(false)
const onlineStatus = ref<string | undefined>()

const loadDevices = async () => {
  loading.value = true
  try {
    const result = await listDevices(onlineStatus.value)
    if (result.data) {
      devices.value = result.data
    }
  } catch (error) {
    console.error('Load devices failed:', error)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadDevices()
})
</script>

<style scoped>
.devices {
  padding: 20px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.page-title {
  font-size: 24px;
  font-weight: 500;
  color: #333;
}

.filters {
  margin-bottom: 20px;
}
</style>
