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
      <el-table-column prop="className" label="班级">
        <template #default="{ row }">{{ row.className || '-' }}</template>
      </el-table-column>
      <el-table-column prop="seatNo" label="座位号" width="90">
        <template #default="{ row }">{{ row.seatNo || '-' }}</template>
      </el-table-column>
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
          {{ formatDateTime(row.lastSeenAt) }}
        </template>
      </el-table-column>
      <el-table-column label="操作" width="250">
        <template #default="{ row }">
          <el-button type="primary" size="small" @click="showModeSwitchDialog(row)">切换模式</el-button>
          <el-button type="warning" size="small" @click="handleLock(row)">锁屏</el-button>
          <el-button type="success" size="small" @click="handleUnlock(row)">解锁</el-button>
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
            <el-option label="考试模式" value="exam" />
            <el-option label="锁定" value="locked" />
          </el-select>
        </el-form-item>
        <el-form-item v-if="modeForm.targetMode === 'teaching'" label="班级">
          <el-select v-model="modeForm.classId" placeholder="请选择班级" filterable style="width: 100%">
            <el-option v-for="c in classes" :key="c.id" :label="c.name" :value="c.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="操作人">
          <el-input v-model="modeForm.operatorName" placeholder="请输入操作人姓名" />
        </el-form-item>
        <el-alert
          v-if="modeForm.targetMode === 'teaching'"
          type="warning"
          :closable="false"
          show-icon
          title="授课模式需选择班级"
          description="系统将按设备座位号匹配班级学生：匹配成功进入授课，否则进入锁定。"
        />
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
import { listDevices, switchDeviceMode, lockDevice, unlockDevice, type Device } from '@/api/devices'
import { listClasses, type ClassInfo } from '@/api/classes'
import { formatDateTime } from '@/utils/time'
import { ElMessage } from 'element-plus'

const devices = ref<Device[]>([])
const loading = ref(false)
const onlineStatus = ref<string | undefined>()
const dialogVisible = ref(false)
const switching = ref(false)
const selectedDevice = ref<Device | null>(null)
const classes = ref<ClassInfo[]>([])
const modeForm = ref({
  targetMode: 'open',
  operatorName: 'admin',
  classId: '' as string,
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
  modeForm.value.classId = device.classId || ''
  dialogVisible.value = true
}

const confirmModeSwitch = async () => {
  if (!selectedDevice.value) return
  if (modeForm.value.targetMode === 'teaching' && !modeForm.value.classId) {
    ElMessage.warning('切换到授课模式前请先选择班级')
    return
  }

  switching.value = true
  try {
    const res = await switchDeviceMode(
      selectedDevice.value.id,
      modeForm.value.targetMode,
      modeForm.value.operatorName,
      modeForm.value.targetMode === 'teaching' ? modeForm.value.classId : undefined
    )
    if (res.code !== 0) {
      ElMessage.error(res.message || '模式切换失败')
      return
    }
    const data = res.data || {}
    if (data.admissionBlocked) {
      ElMessage.warning('座位不匹配，该设备已进入锁定模式')
    } else if (modeForm.value.targetMode === 'teaching') {
      ElMessage.success('已进入授课模式')
    } else {
      ElMessage.success('模式切换成功')
    }
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
  }
  return types[mode] || 'info'
}

const formatMode = (mode: string) => {
  const names: Record<string, string> = {
    open: '开放模式',
    teaching: '授课模式',
    exam: '考试模式',
    locked: '锁定',
  }
  return names[mode] || mode
}

const handleLock = async (device: Device) => {
  try {
    await lockDevice(device.id, '教师远程锁定')
    ElMessage.success('锁屏命令已发送')
    await loadDevices()
  } catch (error) {
    console.error('Lock failed:', error)
    ElMessage.error('锁屏命令发送失败')
  }
}

const handleUnlock = async (device: Device) => {
  try {
    await unlockDevice(device.id)
    ElMessage.success('解锁命令已发送')
    await loadDevices()
  } catch (error) {
    console.error('Unlock failed:', error)
    ElMessage.error('解锁命令发送失败')
  }
}

onMounted(() => {
  loadDevices()
  listClasses().then((res) => {
    if (res.code === 0 && res.data) classes.value = res.data
  }).catch(() => {})
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
