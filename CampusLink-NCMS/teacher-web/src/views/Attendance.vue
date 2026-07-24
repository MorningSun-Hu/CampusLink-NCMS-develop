<template>
  <div class="attendance">
    <div class="header">
      <h2 class="page-title">签到管理</h2>
      <div class="header-actions">
        <el-button type="primary" @click="showRetroactiveDialog = true">补签</el-button>
        <el-button @click="loadData">刷新</el-button>
      </div>
    </div>

    <el-table :data="records" v-loading="loading" border stripe>
      <el-table-column prop="device_id" label="设备ID" width="200" />
      <el-table-column prop="student_id" label="学生ID" width="120" />
      <el-table-column prop="check_in_time" label="签到时间" width="180" />
      <el-table-column prop="check_out_time" label="签退时间" width="180">
        <template #default="{ row }">
          {{ row.check_out_time || '-' }}
        </template>
      </el-table-column>
      <el-table-column prop="status" label="状态" width="100">
        <template #default="{ row }">
          <el-tag :type="statusTagType(row.status)">{{ statusLabel(row.status) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="remarks" label="备注">
        <template #default="{ row }">
          {{ row.remarks || '-' }}
        </template>
      </el-table-column>
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
      <el-row :gutter="16" style="margin-top: 16px">
        <el-col :span="4">
          <el-card class="stat-card">
            <div class="stat-label">总数</div>
            <div class="stat-value">{{ stats.total ?? 0 }}</div>
          </el-card>
        </el-col>
        <el-col :span="4">
          <el-card class="stat-card">
            <div class="stat-label">正常</div>
            <div class="stat-value online">{{ stats.present ?? 0 }}</div>
          </el-card>
        </el-col>
        <el-col :span="4">
          <el-card class="stat-card">
            <div class="stat-label">迟到</div>
            <div class="stat-value warning">{{ stats.late ?? 0 }}</div>
          </el-card>
        </el-col>
        <el-col :span="4">
          <el-card class="stat-card">
            <div class="stat-label">缺勤</div>
            <div class="stat-value offline">{{ stats.absent ?? 0 }}</div>
          </el-card>
        </el-col>
        <el-col :span="4">
          <el-card class="stat-card">
            <div class="stat-label">请假</div>
            <div class="stat-value info">{{ stats.leave ?? 0 }}</div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <el-dialog v-model="showRetroactiveDialog" title="补签" width="400px" destroy-on-close>
      <el-form :model="retroactiveForm" label-width="80px">
        <el-form-item label="学生ID">
          <el-input v-model="retroactiveForm.student_id" placeholder="输入学生ID" />
        </el-form-item>
        <el-form-item label="设备ID">
          <el-input v-model="retroactiveForm.device_id" placeholder="输入设备ID" />
        </el-form-item>
        <el-form-item label="签到时间">
          <el-date-picker
            v-model="retroactiveForm.check_in_time"
            type="datetime"
            placeholder="选择签到时间"
            value-format="YYYY-MM-DD HH:mm:ss"
            style="width: 100%"
          />
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="retroactiveForm.remarks" placeholder="备注信息" />
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
const stats = ref<Partial<AttendanceStatistics>>({})
const loading = ref(false)
const statsDate = ref('')

const showRetroactiveDialog = ref(false)
const retroactiveLoading = ref(false)
const retroactiveForm = ref({
  student_id: '',
  device_id: '',
  check_in_time: '',
  remarks: '',
})

async function loadAttendanceList() {
  loading.value = true
  try {
    const res = await getAttendanceList()
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
      stats.value = res.data
    }
  } catch (e: any) {
    ElMessage.error('获取统计失败: ' + (e?.message || '未知错误'))
  }
}

async function doRetroactive() {
  if (!retroactiveForm.value.student_id) {
    ElMessage.warning('请输入学生ID')
    return
  }
  if (!retroactiveForm.value.device_id) {
    ElMessage.warning('请输入设备ID')
    return
  }
  if (!retroactiveForm.value.check_in_time) {
    ElMessage.warning('请选择时间')
    return
  }
  retroactiveLoading.value = true
  try {
    const res = await retroactive({
      student_id: retroactiveForm.value.student_id,
      device_id: retroactiveForm.value.device_id,
      check_in_time: retroactiveForm.value.check_in_time,
      remarks: retroactiveForm.value.remarks || undefined,
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
    present: '正常',
    late: '迟到',
    absent: '缺勤',
    leave: '请假',
  }
  return map[status] || status
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

.statistics {
  margin-top: 16px;
}

.statistics h3 {
  margin-bottom: 12px;
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

.stat-value.warning {
  color: #e6a23c;
}

.stat-value.info {
  color: #909399;
}
</style>
