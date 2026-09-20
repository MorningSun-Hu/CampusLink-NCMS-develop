<template>
  <div class="usage">
    <div class="header">
      <h2 class="page-title">设备使用</h2>
      <div class="header-actions">
        <el-button type="primary" @click="doExport" :loading="exporting">导出</el-button>
        <el-button @click="loadData">刷新</el-button>
      </div>
    </div>

    <div class="filters">
      <el-select v-model="filters.deviceId" placeholder="选择设备" clearable filterable style="width: 220px" @change="loadData">
        <el-option v-for="d in devices" :key="d.id" :label="`${d.deviceName}（${d.ipAddress}）`" :value="d.id" />
      </el-select>
      <el-select v-model="filters.classId" placeholder="选择班级" clearable filterable style="width: 180px" @change="loadData">
        <el-option v-for="c in classes" :key="c.id" :label="c.name" :value="c.id" />
      </el-select>
      <el-input v-model="filters.studentKeyword" placeholder="使用人/学号/姓名" clearable style="width: 180px" @keyup.enter="loadData" />
      <el-date-picker
        v-model="dateRange"
        type="daterange"
        start-placeholder="开始日期"
        end-placeholder="结束日期"
        value-format="YYYY-MM-DD"
        @change="loadData"
      />
      <el-button type="primary" @click="loadData">查询</el-button>
    </div>

    <el-table :data="records" v-loading="loading" border stripe empty-text="暂无匹配的使用记录">
      <el-table-column prop="deviceName" label="设备" min-width="140">
        <template #default="{ row }">{{ row.deviceName || row.deviceId }}</template>
      </el-table-column>
      <el-table-column prop="userName" label="使用人" width="120">
        <template #default="{ row }">{{ row.userName || row.studentName || '-' }}</template>
      </el-table-column>
      <el-table-column prop="className" label="班级" width="140">
        <template #default="{ row }">{{ row.className || '-' }}</template>
      </el-table-column>
      <el-table-column prop="seatNo" label="座位号" width="90">
        <template #default="{ row }">{{ row.seatNo || '-' }}</template>
      </el-table-column>
      <el-table-column prop="startTime" label="开始时间" width="180">
        <template #default="{ row }">{{ formatDateTime(row.startTime) }}</template>
      </el-table-column>
      <el-table-column prop="endTime" label="结束时间" width="180">
        <template #default="{ row }">{{ formatDateTime(row.endTime) }}</template>
      </el-table-column>
      <el-table-column label="使用时长" width="120">
        <template #default="{ row }">{{ formatDuration(row.durationSeconds) }}</template>
      </el-table-column>
      <el-table-column label="检查结果" min-width="180">
        <template #default="{ row }">
          <el-tag :type="row.inspectionOk ? 'success' : 'warning'" size="small">
            {{ row.inspectionOk ? '正常' : '异常' }}
          </el-tag>
          <span class="summary">{{ row.inspectionSummary || '-' }}</span>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="120" fixed="right">
        <template #default="{ row }">
          <el-button
            v-if="!row.endTime && row.mode !== 'exam'"
            type="warning"
            size="small"
            :loading="endingId === row.id"
            @click="endSession(row)"
          >结束会话</el-button>
          <span v-else>-</span>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { getUsageList, endUsageSession, exportUsage } from '@/api/usage'
import { listDevices, type Device } from '@/api/devices'
import { listClasses, type ClassInfo } from '@/api/classes'
import type { UsageQuery, UsageRecord } from '@/api/types'
import { formatDateTime } from '@/utils/time'

const records = ref<UsageRecord[]>([])
const devices = ref<Device[]>([])
const classes = ref<ClassInfo[]>([])
const loading = ref(false)
const exporting = ref(false)
const endingId = ref('')
const dateRange = ref<[string, string] | null>(null)
const filters = reactive<UsageQuery>({
  deviceId: undefined,
  classId: undefined,
  studentKeyword: '',
})

function queryParams(): UsageQuery {
  return {
    deviceId: filters.deviceId || undefined,
    classId: filters.classId || undefined,
    studentKeyword: filters.studentKeyword || undefined,
    startDate: dateRange.value?.[0],
    endDate: dateRange.value?.[1],
  }
}

async function loadData() {
  loading.value = true
  try {
    const res = await getUsageList(queryParams())
    records.value = res.code === 0 && res.data ? res.data : []
  } catch (e: any) {
    ElMessage.error('查询使用记录失败: ' + (e?.message || '未知错误'))
  } finally {
    loading.value = false
  }
}

async function endSession(row: UsageRecord) {
  endingId.value = row.id
  try {
    const res = await endUsageSession(row.id)
    if (res.code === 0) {
      ElMessage.success('会话已结束')
      loadData()
    } else {
      ElMessage.error(res.message || '结束会话失败')
    }
  } catch (e: any) {
    ElMessage.error('结束会话失败: ' + (e?.message || '未知错误'))
  } finally {
    endingId.value = ''
  }
}

async function doExport() {
  exporting.value = true
  try {
    const blob = await exportUsage(queryParams())
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'usage.csv'
    a.click()
    URL.revokeObjectURL(url)
  } catch (e: any) {
    ElMessage.error('导出失败: ' + (e?.message || '未知错误'))
  } finally {
    exporting.value = false
  }
}

function formatDuration(seconds: number | null) {
  if (seconds == null) return '进行中'
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  if (h > 0) return `${h}小时${m}分`
  if (m > 0) return `${m}分${s}秒`
  return `${s}秒`
}

onMounted(async () => {
  try {
    const [d, c] = await Promise.all([listDevices(), listClasses()])
    if (d.code === 0 && d.data) devices.value = d.data
    if (c.code === 0 && c.data) classes.value = c.data
  } catch (e) {
    console.error(e)
  }
  loadData()
})
</script>

<style scoped>
.usage {
  background: #fff;
  border-radius: 4px;
  padding: 20px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.page-title {
  font-size: 20px;
  margin: 0;
}

.header-actions,
.filters {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  align-items: center;
}

.filters {
  margin-bottom: 16px;
}

.summary {
  margin-left: 8px;
  color: #666;
}
</style>
