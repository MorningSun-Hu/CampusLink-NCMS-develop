<template>
  <div class="whitelist">
    <div class="header">
      <h2 class="page-title">设备白名单</h2>
      <div class="header-actions">
        <el-upload :show-file-list="false" :before-upload="handleImport" accept=".xlsx" style="display: inline-flex">
          <el-button type="warning">批量导入</el-button>
        </el-upload>
      </div>
    </div>

    <el-table :data="entries" v-loading="loading" border stripe>
      <el-table-column prop="deviceCode" label="设备编码" width="160" />
      <el-table-column prop="deviceName" label="设备名称" width="160" />
      <el-table-column prop="macAddress" label="MAC 地址" width="180" />
      <el-table-column prop="status" label="状态" width="100">
        <template #default="{ row }">
          <el-tag :type="row.status === 'approved' ? 'success' : 'warning'">
            {{ row.status === 'approved' ? '已批准' : '待审批' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="createdAt" label="创建时间" width="170" />
      <el-table-column label="操作" width="180">
        <template #default="{ row }">
          <el-button v-if="row.status === 'pending'" type="success" size="small" @click="handleApprove(row)">批准</el-button>
          <el-button type="danger" size="small" @click="handleDelete(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import axios from 'axios'
import { ElMessage, ElMessageBox } from 'element-plus'

interface WhitelistEntry {
  id: string
  deviceCode: string
  deviceName: string
  macAddress: string
  status: string
  createdAt: string
}

const entries = ref<WhitelistEntry[]>([])
const loading = ref(false)

const loadEntries = async () => {
  loading.value = true
  try {
    const token = localStorage.getItem('token')
    const resp = await axios.get('/api/devices/whitelist', { headers: { Authorization: `Bearer ${token}` } })
    if (resp.data.data) {
      entries.value = resp.data.data
    }
  } catch (e) { console.error(e) }
  finally { loading.value = false }
}

const handleImport = async (file: File) => {
  try {
    const formData = new FormData()
    formData.append('file', file)
    const token = localStorage.getItem('token')
    const resp = await axios.post('/api/devices/whitelist/import', formData, {
      headers: { Authorization: `Bearer ${token}`, 'Content-Type': 'multipart/form-data' },
    })
    if (resp.data.data?.imported) {
      ElMessage.success(`成功导入 ${resp.data.data.imported} 条记录`)
      loadEntries()
    }
  } catch (e) { ElMessage.error('导入失败') }
  return false
}

const handleApprove = async (row: WhitelistEntry) => {
  try {
    const token = localStorage.getItem('token')
    await axios.post(`/api/devices/whitelist/${row.id}/approve`, {}, { headers: { Authorization: `Bearer ${token}` } })
    ElMessage.success('批准成功')
    loadEntries()
  } catch (e) { ElMessage.error('批准失败') }
}

const handleDelete = async (row: WhitelistEntry) => {
  try {
    await ElMessageBox.confirm(`确认删除设备 ${row.deviceCode}？`, '删除确认', { type: 'warning' })
    const token = localStorage.getItem('token')
    await axios.delete(`/api/devices/whitelist/${row.id}`, { headers: { Authorization: `Bearer ${token}` } })
    ElMessage.success('删除成功')
    loadEntries()
  } catch (e) { /* cancelled */ }
}

onMounted(() => { loadEntries() })
</script>

<style scoped>
.whitelist { padding: 20px; }
.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.header-actions { display: flex; gap: 12px; align-items: center; }
.page-title { font-size: 24px; font-weight: 500; color: #333; }
</style>
