<template>
  <div class="logs-page">
    <div class="header">
      <h2 class="page-title">日志中心</h2>
      <div class="header-actions">
        <el-button type="primary" @click="exportLogs">导出 CSV</el-button>
        <el-button type="success" :loading="collecting" @click="collectLogs">收集学生机日志</el-button>
        <el-button @click="refreshAll">刷新</el-button>
      </div>
    </div>

    <div class="filters">
      <el-select v-model="logType" placeholder="日志类型" clearable @change="loadLogs">
        <el-option label="注册" value="register" />
        <el-option label="签到" value="attendance" />
        <el-option label="检查" value="inspection" />
        <el-option label="告警" value="alert" />
        <el-option label="模式切换" value="mode_switch" />
      </el-select>
      <el-input v-model="deviceId" placeholder="设备ID筛选" clearable @keyup.enter="loadLogs" style="width: 200px" />
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :total="total"
        :page-sizes="[10, 20, 50]"
        layout="total, sizes, prev, pager, next"
        @size-change="loadLogs"
        @current-change="loadLogs"
        small
      />
    </div>

    <el-table :data="logs" v-loading="loading" border stripe>
      <el-table-column prop="log_type" label="类型" width="100">
        <template #default="{ row }">
          <el-tag size="small">{{ logTypeLabel(row.log_type) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="device_id" label="设备ID" width="200" />
      <el-table-column prop="operator" label="操作人" width="120" />
      <el-table-column prop="action" label="操作" />
      <el-table-column prop="detail" label="详情">
        <template #default="{ row }">
          {{ row.detail || '-' }}
        </template>
      </el-table-column>
      <el-table-column prop="created_at" label="时间" width="180" />
    </el-table>

    <h3 class="section-title">学生机运行日志</h3>
    <el-table :data="agentFiles" v-loading="agentLoading" border stripe>
      <el-table-column prop="device_id" label="设备ID" width="280" />
      <el-table-column prop="file_name" label="文件" />
      <el-table-column prop="size" label="大小" width="120">
        <template #default="{ row }">{{ formatSize(row.size) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="120">
        <template #default="{ row }">
          <el-button link type="primary" @click="downloadFile(row)">下载</el-button>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { queryLogs, exportLogsUrl, collectStudentLogs, listAgentLogs, downloadAgentLog } from '@/api/logs'
import type { LogEntry, AgentLogFileInfo } from '@/api/logs'

const logs = ref<LogEntry[]>([])
const loading = ref(false)
const logType = ref('')
const deviceId = ref('')
const currentPage = ref(1)
const pageSize = ref(20)
const total = ref(0)
const collecting = ref(false)
const agentLoading = ref(false)
const agentFiles = ref<AgentLogFileInfo[]>([])

async function loadLogs() {
  loading.value = true
  try {
    const params: Record<string, string | number> = {
      page: currentPage.value,
      page_size: pageSize.value,
    }
    if (logType.value) params.log_type = logType.value
    if (deviceId.value) params.device_id = deviceId.value
    const res = await queryLogs(params)
    if (res.code === 0 && res.data) {
      logs.value = res.data.items
      total.value = res.data.total
    }
  } catch (e: any) {
    ElMessage.error('查询日志失败: ' + (e?.message || '未知错误'))
  } finally {
    loading.value = false
  }
}

function logTypeLabel(type: string | null) {
  const map: Record<string, string> = {
    register: '注册',
    attendance: '签到',
    inspection: '检查',
    alert: '告警',
    mode_switch: '模式',
  }
  return map[type || ''] || type || '-'
}

function exportLogs() {
  const params: Record<string, string> = {}
  if (logType.value) params.log_type = logType.value
  if (deviceId.value) params.device_id = deviceId.value
  window.open(exportLogsUrl(params), '_blank')
}

async function loadAgentFiles() {
  agentLoading.value = true
  try {
    const res = await listAgentLogs()
    if (res.code === 0 && res.data) {
      agentFiles.value = res.data
    }
  } catch (e: any) {
    ElMessage.error('查询学生机日志失败: ' + (e?.message || '未知错误'))
  } finally {
    agentLoading.value = false
  }
}

async function collectLogs() {
  collecting.value = true
  try {
    const res = await collectStudentLogs()
    if (res.code === 0) {
      ElMessage.success(res.data || '已向在线学生机下发收集指令')
      window.setTimeout(() => {
        loadAgentFiles()
      }, 2500)
    } else {
      ElMessage.error(res.message || '收集失败')
    }
  } catch (e: any) {
    ElMessage.error('收集学生机日志失败: ' + (e?.message || '未知错误'))
  } finally {
    collecting.value = false
  }
}

function formatSize(size: number) {
  if (size < 1024) return size + ' B'
  if (size < 1024 * 1024) return (size / 1024).toFixed(1) + ' KB'
  return (size / 1024 / 1024).toFixed(1) + ' MB'
}

async function downloadFile(row: AgentLogFileInfo) {
  try {
    await downloadAgentLog(row.device_id, row.file_name)
  } catch (e: any) {
    ElMessage.error('下载失败: ' + (e?.message || '未知错误'))
  }
}

function refreshAll() {
  loadLogs()
  loadAgentFiles()
}

onMounted(() => {
  loadLogs()
  loadAgentFiles()
})
</script>

<style scoped>
.logs-page {
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
  align-items: center;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.section-title {
  font-size: 16px;
  margin: 24px 0 12px;
}
</style>
