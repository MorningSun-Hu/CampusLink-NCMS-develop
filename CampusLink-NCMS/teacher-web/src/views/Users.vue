<template>
  <div class="users">
    <div class="header">
      <h2 class="page-title">用户管理</h2>
      <div class="header-actions">
        <el-button type="primary" @click="showCreateDialog">新增教师</el-button>
      </div>
    </div>

    <el-table :data="users" v-loading="loading" border stripe>
      <el-table-column prop="username" label="用户名" width="160" />
      <el-table-column prop="displayName" label="显示名" width="140">
        <template #default="{ row }">{{ row.displayName || '-' }}</template>
      </el-table-column>
      <el-table-column prop="role" label="角色" width="100">
        <template #default="{ row }">
          <el-tag :type="row.role === 'admin' ? 'danger' : 'success'">
            {{ row.role === 'admin' ? '管理员' : '教师' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="status" label="状态" width="90">
        <template #default="{ row }">
          <el-tag :type="row.status === 'active' ? 'success' : 'info'">
            {{ row.status === 'active' ? '正常' : '已停用' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="lastLoginAt" label="最近登录" width="180">
        <template #default="{ row }">{{ row.lastLoginAt || '-' }}</template>
      </el-table-column>
      <el-table-column prop="createdAt" label="创建时间" width="180">
        <template #default="{ row }">{{ row.createdAt || '-' }}</template>
      </el-table-column>
      <el-table-column label="操作" width="300">
        <template #default="{ row }">
          <el-button type="primary" size="small" :disabled="row.role === 'admin'" @click="showEditDialog(row)">编辑</el-button>
          <el-button type="warning" size="small" :disabled="row.role === 'admin'" @click="handleResetPassword(row)">重置密码</el-button>
          <el-button type="danger" size="small" :disabled="row.role === 'admin'" @click="handleDelete(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑教师' : '新增教师'" width="460px">
      <el-form ref="formRef" :model="form" :rules="rules" label-width="90px">
        <el-form-item label="用户名" prop="username">
          <el-input v-model="form.username" :disabled="isEdit" placeholder="登录账号" />
        </el-form-item>
        <el-form-item v-if="!isEdit" label="初始密码" prop="password">
          <el-input v-model="form.password" placeholder="不少于 4 位" show-password />
        </el-form-item>
        <el-form-item label="显示名" prop="displayName">
          <el-input v-model="form.displayName" placeholder="教师姓名" />
        </el-form-item>
        <el-form-item v-if="isEdit" label="状态" prop="status">
          <el-select v-model="form.status" style="width: 100%">
            <el-option label="正常" value="active" />
            <el-option label="停用" value="disabled" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="handleSubmit">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  listUsers, createUser, updateUser, deleteUser, resetUserPassword,
  type AdminUser,
} from '@/api/users'

const users = ref<AdminUser[]>([])
const loading = ref(false)
const dialogVisible = ref(false)
const isEdit = ref(false)
const editingId = ref('')
const saving = ref(false)
const formRef = ref()

const form = reactive({
  username: '',
  password: '',
  displayName: '',
  status: 'active',
})

const rules = {
  username: [{ required: true, message: '请输入用户名', trigger: 'blur' }],
  password: [{ required: true, message: '请输入初始密码', trigger: 'blur' }, { min: 4, message: '密码不少于 4 位', trigger: 'blur' }],
}

function apiErrorMessage(error: unknown, fallback: string) {
  const data = (error as { response?: { data?: { message?: string } } })?.response?.data
  return data?.message || fallback
}

async function loadUsers() {
  loading.value = true
  try {
    const result = await listUsers()
    if (result.data) users.value = result.data
  } catch (error) {
    ElMessage.error(apiErrorMessage(error, '加载账号失败'))
  } finally {
    loading.value = false
  }
}

function showCreateDialog() {
  isEdit.value = false
  editingId.value = ''
  form.username = ''
  form.password = ''
  form.displayName = ''
  form.status = 'active'
  dialogVisible.value = true
}

function showEditDialog(row: AdminUser) {
  isEdit.value = true
  editingId.value = row.id
  form.username = row.username
  form.displayName = row.displayName
  form.status = row.status
  dialogVisible.value = true
}

async function handleSubmit() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return

  saving.value = true
  try {
    if (isEdit.value) {
      const result = await updateUser(editingId.value, {
        displayName: form.displayName,
        status: form.status,
      })
      if (result.code !== 0) {
        ElMessage.error(result.message || '操作失败')
        return
      }
      ElMessage.success('更新成功')
    } else {
      const result = await createUser({
        username: form.username,
        password: form.password,
        displayName: form.displayName,
      })
      if (result.code !== 0) {
        ElMessage.error(result.message || '操作失败')
        return
      }
      ElMessage.success('创建成功')
    }
    dialogVisible.value = false
    loadUsers()
  } catch (error) {
    ElMessage.error(apiErrorMessage(error, '操作失败'))
  } finally {
    saving.value = false
  }
}

async function handleResetPassword(row: AdminUser) {
  try {
    const { value } = await ElMessageBox.prompt(`为 ${row.username} 设置新密码`, '重置密码', {
      inputType: 'password',
      inputPlaceholder: '不少于 4 位',
      inputValidator: (v: string) => (v && v.length >= 4) || '密码不少于 4 位',
    })
    const result = await resetUserPassword(row.id, value)
    if (result.code !== 0) {
      ElMessage.error(result.message || '重置失败')
      return
    }
    ElMessage.success('密码已重置')
  } catch (error) {
    if (error === 'cancel' || error === 'close') return
    ElMessage.error(apiErrorMessage(error, '重置失败'))
  }
}

async function handleDelete(row: AdminUser) {
  try {
    await ElMessageBox.confirm(`确认删除教师账号 ${row.username}？`, '删除账号', { type: 'warning' })
  } catch {
    return
  }
  try {
    const result = await deleteUser(row.id)
    if (result.code !== 0) {
      ElMessage.error(result.message || '删除失败')
      return
    }
    ElMessage.success('已删除')
    loadUsers()
  } catch (error) {
    ElMessage.error(apiErrorMessage(error, '删除失败'))
  }
}

onMounted(loadUsers)
</script>

<style scoped>
.users {
  padding: 4px;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.page-title {
  margin: 0;
  font-size: 20px;
  color: #333;
}
</style>
