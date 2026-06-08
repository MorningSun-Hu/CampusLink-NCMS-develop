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
      <el-table-column prop="currentMode" label="当前模式">
        <template #default="{ row }">
          <el-tag :type="getModeTagType(row.currentMode)">{{ formatMode(row.currentMode) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="lastSeenAt" label="最后心跳时间">
        <template #default="{ row }">
          {{ row.lastSeenAt || '-' }}
        </template>
      </el-table-column>
      <el-table-column label="操作" width="150">
        <template #default="{ row }">
          <el-button type="primary" size="small" @click="showModeSwitchDialog(row)">切换模式</el-button>
        </template>
      </el-table-column>
    </el-table>

    <!-- 模式切换弹窗 -->
    <el-dialog v-model="dialogVisible" title="切换模式" width="400px">
      <el-form :model="modeForm" label-width="80px">
        <el-form-item label="目标模式">
          <el-select v-model="modeForm.targetMode" style="width: 100%">
            <el-option label="开放模式" value="open" />
            <el-option label="授课模式" value="teaching" />
            <el-option label="条件开放" value="conditional_open" />
            <el-option label="考试模式" value="exam" />
            <el-option label="锁定" value="locked" />
          </el-select>
        </el-form-item>
        <el-form-item label="操作人">
          <el-input v-model="modeForm.operatorName" placeholder="请输入操作人姓名" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="confirmModeSwitch" :loading="switching">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { listDevices, switchDeviceMode, type Device } from '@/api/devices'
import { ElMessage } from 'element-plus'

const devices = ref<Device[]>([])
const loading = ref(false)
const onlineStatus = ref<string | undefined>()
const dialogVisible = ref(false)
const switching = ref(false)
const selectedDevice = ref<Device | null>(null)
const modeForm = ref({
  targetMode: 'open',
  operatorName: 'admin',
})

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

const showModeSwitchDialog = (device: Device) => {
  selectedDevice.value = device
  modeForm.value.targetMode = device.currentMode
  dialogVisible.value = true
}

const confirmModeSwitch = async () => {
  if (!selectedDevice.value) return
  
  switching.value = true
  try {
    await switchDeviceMode(
      selectedDevice.value.id,
      modeForm.value.targetMode,
      modeForm.value.operatorName
    )
    ElMessage.success('模式切换成功')
    dialogVisible.value = false
    loadDevices()
  } catch (error) {
    console.error('Mode switch failed:', error)
    ElMessage.error('模式切换失败')
  } finally {
    switching.value = false
  }
}

const getModeTagType = (mode: string) => {
  const types: Record<string, string> = {
    open: 'success',
    teaching: 'primary',
    exam: 'warning',
    locked: 'danger',
    conditional_open: 'info',
  }
  return types[mode] || 'info'
}

const formatMode = (mode: string) => {
  const names: Record<string, string> = {
    open: '开放模式',
    teaching: '授课模式',
    exam: '考试模式',
    locked: '锁定',
    conditional_open: '条件开放',
  }
  return names[mode] || mode
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
