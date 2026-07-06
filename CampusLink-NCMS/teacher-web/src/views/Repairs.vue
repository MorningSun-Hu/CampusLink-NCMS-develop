<template>
  <div class="repairs">
    <div class="header">
      <h2 class="page-title">维修工单</h2>
      <div class="header-actions">
        <el-select v-model="statusFilter" placeholder="状态筛选" clearable @change="loadRepairs" style="width: 140px">
          <el-option label="全部" value="all" />
          <el-option label="待处理" value="pending" />
          <el-option label="处理中" value="processing" />
          <el-option label="已完成" value="completed" />
        </el-select>
        <el-button type="primary" @click="showCreateDialog">新建工单</el-button>
      </div>
    </div>

    <el-table :data="orders" v-loading="loading" border stripe>
      <el-table-column prop="deviceName" label="设备名称" width="140" />
      <el-table-column prop="issueType" label="问题类型" width="100">
        <template #default="{ row }">
          <el-tag size="small">{{ row.issueType === 'hardware' ? '硬件' : row.issueType === 'software' ? '软件' : row.issueType }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="description" label="描述" min-width="200" show-overflow-tooltip />
      <el-table-column prop="reporter" label="报修人" width="100" />
      <el-table-column prop="status" label="状态" width="100">
        <template #default="{ row }">
          <el-tag :type="statusType(row.status)">{{ statusLabel(row.status) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="assignedTo" label="指派人" width="100" />
      <el-table-column prop="createdAt" label="创建时间" width="170" />
      <el-table-column label="操作" width="200">
        <template #default="{ row }">
          <el-button v-if="row.status === 'pending'" type="success" size="small" @click="handleStatus(row, 'processing')">接单</el-button>
          <el-button v-if="row.status === 'processing'" type="warning" size="small" @click="handleStatus(row, 'completed')">完成</el-button>
          <el-button type="danger" size="small" @click="handleDelete(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <div class="pagination">
      <el-pagination v-model:current-page="page" :page-size="pageSize" :total="total" layout="prev, pager, next, total" @current-change="loadRepairs" />
    </div>

    <el-dialog v-model="dialogVisible" title="新建工单" width="480px">
      <el-form :model="form" label-width="80px">
        <el-form-item label="设备名称" required>
          <el-input v-model="form.deviceName" placeholder="例如: PC-01" />
        </el-form-item>
        <el-form-item label="问题类型" required>
          <el-select v-model="form.issueType" style="width: 100%">
            <el-option label="硬件故障" value="hardware" />
            <el-option label="软件问题" value="software" />
            <el-option label="网络问题" value="network" />
            <el-option label="其他" value="other" />
          </el-select>
        </el-form-item>
        <el-form-item label="报修人" required>
          <el-input v-model="form.reporter" placeholder="请输入报修人" />
        </el-form-item>
        <el-form-item label="描述" required>
          <el-input v-model="form.description" type="textarea" :rows="3" placeholder="请输入问题描述" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleCreate">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { listRepairs, createRepair, updateRepair, deleteRepair, type RepairOrder } from '@/api/repairs'
import { ElMessage, ElMessageBox } from 'element-plus'

const orders = ref<RepairOrder[]>([])
const loading = ref(false)
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const statusFilter = ref('')

const dialogVisible = ref(false)
const form = reactive({
  deviceId: '',
  deviceName: '',
  reporter: '',
  issueType: 'hardware',
  description: '',
})

const statusType = (s: string) => s === 'pending' ? 'danger' : s === 'processing' ? 'warning' : 'success'
const statusLabel = (s: string) => s === 'pending' ? '待处理' : s === 'processing' ? '处理中' : '已完成'

const loadRepairs = async () => {
  loading.value = true
  try {
    const result = await listRepairs({ page: page.value, pageSize: pageSize.value, status: statusFilter.value || undefined })
    if (result.data) {
      orders.value = result.data.orders
      total.value = result.data.total
    }
  } catch (e) { console.error(e) }
  finally { loading.value = false }
}

const showCreateDialog = () => {
  form.deviceName = ''
  form.reporter = ''
  form.issueType = 'hardware'
  form.description = ''
  dialogVisible.value = true
}

const handleCreate = async () => {
  if (!form.deviceName || !form.reporter || !form.description) {
    ElMessage.warning('请填写必填字段')
    return
  }
  try {
    await createRepair(form)
    ElMessage.success('工单创建成功')
    dialogVisible.value = false
    loadRepairs()
  } catch (e) { ElMessage.error('创建失败') }
}

const handleStatus = async (row: RepairOrder, status: string) => {
  try {
    await updateRepair(row.id, { status })
    ElMessage.success('状态更新成功')
    loadRepairs()
  } catch (e) { ElMessage.error('更新失败') }
}

const handleDelete = async (row: RepairOrder) => {
  try {
    await ElMessageBox.confirm('确认删除该工单？', '删除确认', { type: 'warning' })
    await deleteRepair(row.id)
    ElMessage.success('删除成功')
    loadRepairs()
  } catch (e) { /* cancelled */ }
}

onMounted(() => { loadRepairs() })
</script>

<style scoped>
.repairs { padding: 20px; }
.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.header-actions { display: flex; gap: 12px; align-items: center; }
.page-title { font-size: 24px; font-weight: 500; color: #333; }
.pagination { margin-top: 20px; display: flex; justify-content: flex-end; }
</style>
