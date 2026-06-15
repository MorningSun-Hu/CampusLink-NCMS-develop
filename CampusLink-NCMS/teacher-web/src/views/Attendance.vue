<template>
  <div class="attendance">
    <div class="header">
      <h2 class="page-title">签到管理</h2>
      <div class="header-actions">
        <el-button type="primary" @click="showRetroactiveDialog = true">补签</el-button>
        <el-button @click="loadData">刷新</el-button>
      </div>
    </div>

    <div class="filters">
      <el-select v-model="filterRecordType" placeholder="记录类型" clearable @change="loadAttendanceList">
        <el-option label="签到" value="check_in" />
        <el-option label="签退" value="check_out" />
      </el-select>
      <el-date-picker
        v-model="filterDate"
        type="date"
        placeholder="选择日期"
        value-format="YYYY-MM-DD"
        clearable
        @change="loadAttendanceList"
      />
    </div>

    <el-table :data="records" v-loading="loading" border stripe>
      <el-table-column prop="device_id" label="设备ID" width="200" />
      <el-table-column prop="record_type" label="记录类型">
        <template #default="{ row }">
          <el-tag :type="row.record_type === 'check_in' ? 'success' : 'warning'">
            {{ row.record_type === 'check_in' ? '签到' : '签退' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="time_type" label="时段">
        <template #default="{ row }">
          <el-tag :type="timeTypeTag(row.time_type)">{{ timeTypeLabel(row.time_type) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="timestamp" label="时间" width="180" />
      <el-table-column prop="created_at" label="创建时间" width="180" />
    </el-table>

    <el-divider />

    <div class="statistics">
      <h3>签到统计</h3>
      <el-date-picker
        v-model="statsDate"
        type="date"
        placeholder="选择统计日期"
        value-format="YYYY-MM-DD"
        @change="loadStatistics"
      />
      <el-table :data="statistics" border stripe style="margin-top: 16px">
        <el-table-column prop="date" label="日期" />
        <el-table-column prop="total" label="总数" />
        <el-table-column prop="present" label="正常">
          <template #default="{ row }">
            <el-tag type="success">{{ row.present }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="absent" label="缺勤">
          <template #default="{ row }">
            <el-tag type="danger">{{ row.absent }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="late" label="迟到">
          <template #default="{ row }">
            <el-tag type="warning">{{ row.late }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="leave_early" label="早退">
          <template #default="{ row }">
            <el-tag type="info">{{ row.leave_early }}</el-tag>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <el-dialog v-model="showRetroactiveDialog" title="补签" width="400px">
      <el-form :model="retroactiveForm" label-width="80px">
        <el-form-item label="设备ID">
          <el-input v-model="retroactiveForm.device_id" placeholder="输入设备ID" />
        </el-form-item>
        <el-form-item label="记录类型">
          <el-select v-model="retroactiveForm.record_type" style="width: 100%">
            <el-option label="签到" value="check_in" />
            <el-option label="签退" value="check_out" />
          </el-select>
        </el-form-item>
        <el-form-item label="时间">
          <el-date-picker
            v-model="retroactiveForm.timestamp"
            type="datetime"
            placeholder="选择补签时间"
            value-format="x"
            style="width: 100%"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showRetroactiveDialog = false">取消</el-button>
        <el-button type="primary" @click="doRetroactive" :loading="retroactiveLoading">确认补签</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { getAttendanceList, getAttendanceStatistics, retroactive } from '@/api/attendance'
import type { AttendanceRecord, AttendanceStatistics } from '@/api/types'

const records = ref<AttendanceRecord[]>([])
const statistics = ref<AttendanceStatistics[]>([])
const loading = ref(false)
const filterRecordType = ref('')
const filterDate = ref('')
const statsDate = ref('')

const showRetroactiveDialog = ref(false)
const retroactiveLoading = ref(false)
const retroactiveForm = ref({
  device_id: '',
  record_type: 'check_in',
  timestamp: '',
})

async function loadAttendanceList() {
  loading.value = true
  try {
    const params: Record<string, string> = {}
    if (filterRecordType.value) params.record_type = filterRecordType.value
    if (filterDate.value) params.date = filterDate.value
    const res = await getAttendanceList(params)
    if (res.code === 0 && res.data) {
      records.value = res.data
    }
  } catch (e: any) {
    ElMessage.error('获取签到记录失败: ' + (e?.message || '未知错误'))
  } finally {
    loading.value = false
  }
}

async function loadStatistics() {
  try {
    const res = await getAttendanceStatistics(statsDate.value || undefined)
    if (res.code === 0 && res.data) {
      statistics.value = res.data
    }
  } catch (e: any) {
    ElMessage.error('获取统计失败: ' + (e?.message || '未知错误'))
  }
}

async function doRetroactive() {
  if (!retroactiveForm.value.device_id) {
    ElMessage.warning('请输入设备ID')
    return
  }
  if (!retroactiveForm.value.timestamp) {
    ElMessage.warning('请选择时间')
    return
  }
  retroactiveLoading.value = true
  try {
    const res = await retroactive({
      device_id: retroactiveForm.value.device_id,
      record_type: retroactiveForm.value.record_type,
      timestamp: Number(retroactiveForm.value.timestamp) / 1000,
    })
    if (res.code === 0) {
      ElMessage.success('补签成功')
      showRetroactiveDialog.value = false
      loadAttendanceList()
    } else {
      ElMessage.error(res.message || '补签失败')
    }
  } catch (e: any) {
    ElMessage.error('补签失败: ' + (e?.message || '未知错误'))
  } finally {
    retroactiveLoading.value = false
  }
}

function loadData() {
  loadAttendanceList()
  loadStatistics()
}

function timeTypeTag(type: string) {
  const map: Record<string, string> = {
    on_time: 'success',
    late: 'warning',
    leave_early: 'info',
  }
  return map[type] || 'info'
}

function timeTypeLabel(type: string) {
  const map: Record<string, string> = {
    on_time: '准时',
    late: '迟到',
    leave_early: '早退',
  }
  return map[type] || type
}

onMounted(() => {
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
  margin-bottom: 20px;
}

.page-title {
  font-size: 20px;
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.filters {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
}

.statistics {
  margin-top: 16px;
}

.statistics h3 {
  margin-bottom: 12px;
}
</style>
