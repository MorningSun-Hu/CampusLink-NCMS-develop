<template>
  <div class="hardware-page">
    <div class="header">
      <h2 class="page-title">硬件快照</h2>
      <el-button @click="loadData">刷新</el-button>
    </div>

    <div class="filters">
      <el-input v-model="deviceId" placeholder="输入设备ID查询" clearable @keyup.enter="loadSnapshot" style="width: 280px">
        <template #append>
          <el-button @click="loadSnapshot">查询</el-button>
        </template>
      </el-input>
    </div>

    <el-card v-if="snapshot" class="snapshot-card">
      <template #header>设备 {{ snapshot.device_id }} 硬件信息</template>
      <el-descriptions :column="2" border>
        <el-descriptions-item label="CPU">{{ snapshot.cpu_model || '-' }}</el-descriptions-item>
        <el-descriptions-item label="核心数">{{ snapshot.cpu_cores || '-' }}</el-descriptions-item>
        <el-descriptions-item label="内存">{{ formatBytes(snapshot.total_memory_bytes) }}</el-descriptions-item>
        <el-descriptions-item label="操作系统">{{ snapshot.os_version || '-' }}</el-descriptions-item>
        <el-descriptions-item label="主机名">{{ snapshot.hostname || '-' }}</el-descriptions-item>
        <el-descriptions-item label="MAC">{{ snapshot.mac_addresses || '-' }}</el-descriptions-item>
        <el-descriptions-item label="采集时间" :span="2">{{ snapshot.created_at }}</el-descriptions-item>
      </el-descriptions>
    </el-card>
    <el-empty v-else description="输入设备ID查询硬件快照" />

    <el-divider />

    <h3>硬件变更记录</h3>
    <el-table :data="changes" v-loading="changesLoading" border stripe>
      <el-table-column prop="device_id" label="设备ID" width="200" />
      <el-table-column prop="change_type" label="变更类型">
        <template #default="{ row }">
          <el-tag type="warning">{{ row.change_type }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="field_name" label="字段" />
      <el-table-column prop="old_value" label="旧值">
        <template #default="{ row }">{{ row.old_value || '-' }}</template>
      </el-table-column>
      <el-table-column prop="new_value" label="新值">
        <template #default="{ row }">{{ row.new_value || '-' }}</template>
      </el-table-column>
      <el-table-column prop="detected_at" label="检测时间" width="180" />
    </el-table>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { getHardwareSnapshot, getHardwareChanges } from '@/api/hardware'
import type { HardwareInfo, HardwareChange } from '@/api/hardware'

const deviceId = ref('')
const snapshot = ref<HardwareInfo | null>(null)
const changes = ref<HardwareChange[]>([])
const changesLoading = ref(false)

async function loadSnapshot() {
  if (!deviceId.value) {
    ElMessage.warning('请输入设备ID')
    return
  }
  try {
    const res = await getHardwareSnapshot(deviceId.value)
    if (res.code === 0 && res.data) {
      snapshot.value = res.data
    } else {
      snapshot.value = null
    }
  } catch (e: any) {
    ElMessage.error('查询失败: ' + (e?.message || '未知错误'))
  }
}

async function loadChanges() {
  changesLoading.value = true
  try {
    const res = await getHardwareChanges()
    if (res.code === 0 && res.data) {
      changes.value = res.data
    }
  } catch (e: any) {
    ElMessage.error('获取变更记录失败: ' + (e?.message || '未知错误'))
  } finally {
    changesLoading.value = false
  }
}

function loadData() {
  if (deviceId.value) loadSnapshot()
  loadChanges()
}

function formatBytes(bytes: number | null) {
  if (bytes === null || bytes === undefined) return '-'
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB'
}

onMounted(() => {
  loadChanges()
})
</script>

<style scoped>
.hardware-page {
  background: #fff;
  border-radius: 4px;
  padding: 20px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.page-title {
  font-size: 20px;
  margin: 0;
}

.filters {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
}

.snapshot-card {
  margin-bottom: 16px;
}
</style>
