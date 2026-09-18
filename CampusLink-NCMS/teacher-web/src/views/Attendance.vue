<template>
  <div class="attendance">
    <div class="header">
      <h2 class="page-title">签到管理</h2>
      <div class="header-actions">
        <el-button type="primary" @click="doExport" :loading="exporting">导出</el-button>
        <el-button @click="loadData">刷新</el-button>
      </div>
    </div>

    <div class="filters">
      <el-select v-model="filters.classId" placeholder="选择班级" clearable filterable style="width: 200px" @change="onClassChange">
        <el-option v-for="c in classes" :key="c.id" :label="c.name" :value="c.id" />
      </el-select>
      <el-date-picker
        v-model="dateRange"
        type="daterange"
        start-placeholder="开始日期"
        end-placeholder="结束日期"
        value-format="YYYY-MM-DD"
        @change="loadData"
      />
      <el-input v-model="filters.studentKeyword" placeholder="学号或姓名" clearable style="width: 180px" @keyup.enter="loadData" />
      <el-select v-model="filters.status" placeholder="签到状态" clearable style="width: 140px" @change="loadData">
        <el-option label="出勤" value="present" />
        <el-option label="缺勤" value="absent" />
        <el-option label="迟到" value="late" />
        <el-option label="请假" value="leave" />
      </el-select>
      <el-button type="primary" @click="loadData">查询</el-button>
    </div>

    <div v-if="board" class="board">
      <el-row :gutter="16">
        <el-col :span="6">
          <el-card class="stat-card">
            <div class="stat-label">应出勤</div>
            <div class="stat-value">{{ board.totalStudents }}</div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card">
            <div class="stat-label">已签到</div>
            <div class="stat-value online">{{ board.presentCount }}</div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card">
            <div class="stat-label">缺勤</div>
            <div class="stat-value offline">{{ board.absentCount }}</div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card">
            <div class="stat-label">出勤率</div>
            <div class="stat-value">{{ formatRate(board.attendanceRate) }}</div>
          </el-card>
        </el-col>
      </el-row>
    </div>
    <el-alert
      v-else
      type="info"
      :closable="false"
      show-icon
      title="请选择班级后查看出勤看板"
      style="margin-bottom: 16px"
    />

    <el-table :data="records" v-loading="loading" border stripe empty-text="暂无匹配的考勤记录">
      <el-table-column prop="studentName" label="姓名" width="110">
        <template #default="{ row }">{{ row.studentName || '-' }}</template>
      </el-table-column>
      <el-table-column prop="studentNo" label="学号" width="130">
        <template #default="{ row }">{{ row.studentNo || '-' }}</template>
      </el-table-column>
      <el-table-column prop="className" label="班级" width="140">
        <template #default="{ row }">{{ row.className || '-' }}</template>
      </el-table-column>
      <el-table-column prop="seatNo" label="座位号" width="90">
        <template #default="{ row }">{{ row.seatNo || '-' }}</template>
      </el-table-column>
      <el-table-column prop="deviceName" label="设备" min-width="140">
        <template #default="{ row }">{{ row.deviceName || row.deviceId || '-' }}</template>
      </el-table-column>
      <el-table-column prop="checkInTime" label="签到时间" width="180">
        <template #default="{ row }">{{ formatDateTime(row.checkInTime) }}</template>
      </el-table-column>
      <el-table-column prop="status" label="状态" width="100">
        <template #default="{ row }">
          <el-tag :type="statusTagType(row.status)">{{ statusLabel(row.status) }}</el-tag>
        </template>
      </el-table-column>
    </el-table>

    <div v-if="board && board.absent.length" class="absent-block">
      <h3>缺勤名单</h3>
      <el-table :data="board.absent" border size="small">
        <el-table-column prop="studentName" label="姓名" width="110" />
        <el-table-column prop="studentNo" label="学号" width="130" />
        <el-table-column prop="seatNo" label="座位号" width="90">
          <template #default="{ row }">{{ row.seatNo || '-' }}</template>
        </el-table-column>
        <el-table-column label="状态" width="100">
          <template #default>
            <el-tag type="danger">缺勤</el-tag>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { getAttendanceList, getAttendanceBoard, exportAttendance } from '@/api/attendance'
import { listClasses, type ClassInfo } from '@/api/classes'
import type { AttendanceQuery, AttendanceRecord, AttendanceBoard } from '@/api/types'
import { formatDateTime } from '@/utils/time'

const records = ref<AttendanceRecord[]>([])
const board = ref<AttendanceBoard | null>(null)
const classes = ref<ClassInfo[]>([])
const loading = ref(false)
const exporting = ref(false)
const dateRange = ref<[string, string] | null>(null)
const filters = reactive<AttendanceQuery>({
  classId: undefined,
  studentKeyword: '',
  status: undefined,
})

function queryParams(): AttendanceQuery {
  return {
    classId: filters.classId || undefined,
    studentKeyword: filters.studentKeyword || undefined,
    status: filters.status || undefined,
    startDate: dateRange.value?.[0],
    endDate: dateRange.value?.[1],
  }
}

function today() {
  const d = new Date()
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

async function loadRecords() {
  loading.value = true
  try {
    const res = await getAttendanceList(queryParams())
    records.value = res.code === 0 && res.data ? res.data : []
  } catch (e: any) {
    ElMessage.error('获取考勤记录失败: ' + (e?.message || '未知错误'))
  } finally {
    loading.value = false
  }
}

async function loadBoard() {
  if (!filters.classId) {
    board.value = null
    return
  }
  try {
    const res = await getAttendanceBoard({
      classId: filters.classId,
      date: dateRange.value?.[1] || dateRange.value?.[0] || today(),
    })
    board.value = res.code === 0 ? res.data : null
  } catch (e) {
    board.value = null
  }
}

function loadData() {
  loadRecords()
  loadBoard()
}

function onClassChange() {
  loadData()
}

async function doExport() {
  exporting.value = true
  try {
    const blob = await exportAttendance(queryParams())
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'attendance.csv'
    a.click()
    URL.revokeObjectURL(url)
  } catch (e: any) {
    ElMessage.error('导出失败: ' + (e?.message || '未知错误'))
  } finally {
    exporting.value = false
  }
}

function formatRate(rate: number) {
  return `${Math.round((rate || 0) * 1000) / 10}%`
}

function statusTagType(status: string) {
  const map: Record<string, string> = {
    present: 'success',
    late: 'warning',
    absent: 'danger',
    leave: 'info',
  }
  return map[status] || 'info'
}

function statusLabel(status: string) {
  const map: Record<string, string> = {
    present: '出勤',
    late: '迟到',
    absent: '缺勤',
    leave: '请假',
  }
  return map[status] || status
}

onMounted(async () => {
  try {
    const res = await listClasses()
    if (res.code === 0 && res.data) classes.value = res.data
  } catch (e) {
    console.error(e)
  }
  loadData()
})
</script>

<style scoped>
.attendance {
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

.board {
  margin-bottom: 16px;
}

.stat-card {
  text-align: center;
  padding: 12px;
}

.stat-label {
  font-size: 13px;
  color: #666;
  margin-bottom: 6px;
}

.stat-value {
  font-size: 28px;
  font-weight: bold;
  color: #333;
}

.stat-value.online {
  color: #67c23a;
}

.stat-value.offline {
  color: #f56c6c;
}

.absent-block {
  margin-top: 24px;
}

.absent-block h3 {
  margin: 0 0 12px;
  font-size: 16px;
}
</style>
