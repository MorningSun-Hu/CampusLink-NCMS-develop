<template>
  <div class="network-accounts-page">
    <div class="header">
      <h2 class="page-title">网络认证账号</h2>
      <el-button type="primary" @click="showCreateDialog">添加账号</el-button>
    </div>

    <el-table :data="accounts" v-loading="loading" border stripe>
      <el-table-column prop="accountName" label="账号名" />
      <el-table-column prop="deviceId" label="关联设备">
        <template #default="{ row }">
          {{ row.deviceId || '-' }}
        </template>
      </el-table-column>
      <el-table-column prop="loginUrl" label="登录地址">
        <template #default="{ row }">
          <span style="word-break:break-all">{{ row.loginUrl || '-' }}</span>
        </template>
      </el-table-column>
      <el-table-column prop="autoLogin" label="自动登录" width="100">
        <template #default="{ row }">
          <el-tag :type="row.autoLogin ? 'success' : 'info'">{{ row.autoLogin ? '是' : '否' }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="autoLogout" label="自动注销" width="100">
        <template #default="{ row }">
          <el-tag :type="row.autoLogout ? 'warning' : 'info'">{{ row.autoLogout ? '是' : '否' }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="status" label="状态" width="80">
        <template #default="{ row }">
          <el-tag :type="row.status === 'active' ? 'success' : 'danger'">
            {{ row.status === 'active' ? '启用' : '禁用' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="180">
        <template #default="{ row }">
          <el-button size="small" @click="showEditDialog(row)">编辑</el-button>
          <el-button size="small" type="danger" @click="handleDelete(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑账号' : '添加账号'" width="500px">
      <el-form :model="form" label-width="100px">
        <el-form-item label="账号名" required>
          <el-input v-model="form.account_name" placeholder="输入外网认证账号" />
        </el-form-item>
        <el-form-item label="密码" required>
          <el-input v-model="form.password" placeholder="输入密码" show-password />
        </el-form-item>
        <el-form-item label="关联设备">
          <el-input v-model="form.device_id" placeholder="设备ID (可选)" />
        </el-form-item>
        <el-form-item label="登录地址">
          <el-input v-model="form.login_url" placeholder="自动登录URL (可选)" />
        </el-form-item>
        <el-form-item label="注销地址">
          <el-input v-model="form.logout_url" placeholder="自动注销URL (可选)" />
        </el-form-item>
        <el-form-item label="自动登录">
          <el-switch v-model="form.auto_login" />
        </el-form-item>
        <el-form-item label="自动注销">
          <el-switch v-model="form.auto_logout" />
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="form.description" type="textarea" placeholder="备注信息" />
        </el-form-item>
        <el-form-item v-if="isEdit" label="状态">
          <el-select v-model="form.status" style="width:100%">
            <el-option label="启用" value="active" />
            <el-option label="禁用" value="disabled" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSave" :loading="saving">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  listNetworkAccounts, createNetworkAccount, updateNetworkAccount, deleteNetworkAccount,
  type NetworkAccount, type CreateNetworkAccountParams, type UpdateNetworkAccountParams,
} from '@/api/networkAccounts'

const accounts = ref<NetworkAccount[]>([])
const loading = ref(false)
const dialogVisible = ref(false)
const isEdit = ref(false)
const editingId = ref('')
const saving = ref(false)

const form = ref<CreateNetworkAccountParams & { status: string }>({
  account_name: '',
  password: '',
  device_id: '',
  login_url: '',
  logout_url: '',
  auto_login: false,
  auto_logout: false,
  description: '',
  status: 'active',
})

async function loadAccounts() {
  loading.value = true
  try {
    const res = await listNetworkAccounts()
    if (res.data) accounts.value = res.data
  } catch (e: any) {
    ElMessage.error('加载失败: ' + (e?.message || '未知错误'))
  } finally {
    loading.value = false
  }
}

function showCreateDialog() {
  isEdit.value = false
  editingId.value = ''
  form.value = { account_name: '', password: '', device_id: '', login_url: '', logout_url: '', auto_login: false, auto_logout: false, description: '', status: 'active' }
  dialogVisible.value = true
}

function showEditDialog(row: NetworkAccount) {
  isEdit.value = true
  editingId.value = row.id
  form.value = {
    account_name: row.accountName,
    password: '',
    device_id: row.deviceId || '',
    login_url: row.loginUrl || '',
    logout_url: row.logoutUrl || '',
    auto_login: row.autoLogin === 1,
    auto_logout: row.autoLogout === 1,
    description: row.description || '',
    status: row.status,
  }
  dialogVisible.value = true
}

async function handleSave() {
  if (!form.value.account_name) { ElMessage.warning('请输入账号名'); return }
  saving.value = true
  try {
    if (isEdit.value) {
      const updateParams: UpdateNetworkAccountParams = {
        account_name: form.value.account_name,
        device_id: form.value.device_id || undefined,
        login_url: form.value.login_url || undefined,
        logout_url: form.value.logout_url || undefined,
        auto_login: form.value.auto_login,
        auto_logout: form.value.auto_logout,
        description: form.value.description || undefined,
        status: form.value.status,
      }
      if (form.value.password) updateParams.password = form.value.password
      await updateNetworkAccount(editingId.value, updateParams)
      ElMessage.success('更新成功')
    } else {
      if (!form.value.password) { ElMessage.warning('请输入密码'); return }
      await createNetworkAccount({
        account_name: form.value.account_name,
        password: form.value.password,
        device_id: form.value.device_id || undefined,
        login_url: form.value.login_url || undefined,
        logout_url: form.value.logout_url || undefined,
        auto_login: form.value.auto_login,
        auto_logout: form.value.auto_logout,
        description: form.value.description || undefined,
      })
      ElMessage.success('创建成功')
    }
    dialogVisible.value = false
    loadAccounts()
  } catch (e: any) {
    ElMessage.error('保存失败: ' + (e?.response?.data?.message || e?.message || '未知错误'))
  } finally {
    saving.value = false
  }
}

async function handleDelete(row: NetworkAccount) {
  try {
    await ElMessageBox.confirm('确认删除该网络账号？', '删除确认', { type: 'warning' })
  } catch { return }
  try {
    await deleteNetworkAccount(row.id)
    ElMessage.success('删除成功')
    loadAccounts()
  } catch (e: any) {
    ElMessage.error('删除失败: ' + (e?.message || '未知错误'))
  }
}

onMounted(() => { loadAccounts() })
</script>

<style scoped>
.network-accounts-page { background: #fff; border-radius: 4px; padding: 20px; }
.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.page-title { font-size: 20px; margin: 0; }
</style>
