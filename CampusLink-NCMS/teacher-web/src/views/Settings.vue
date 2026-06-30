<template>
  <div class="settings-page">
    <h2 class="page-title">系统设置</h2>

    <el-card class="setting-card">
      <template #header>
        <span class="card-title">锁屏超级密码</span>
      </template>
      <p class="card-desc">修改后所有已连接的学生端将实时更新。新密码用于 campus-lock 锁屏界面解锁。</p>
      <div class="password-row">
        <el-input
          v-model="newPassword"
          placeholder="输入新密码"
          show-password
          style="width: 280px"
          :disabled="submitting"
        />
        <el-button type="primary" @click="handleUpdate" :loading="submitting">更新密码</el-button>
      </div>
      <div v-if="statusMsg" class="status-msg" :class="{ error: statusError }">
        {{ statusMsg }}
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getLockPasswordStatus, updateLockPassword } from '@/api/settings'
import { ElMessage } from 'element-plus'

const newPassword = ref('')
const submitting = ref(false)
const statusMsg = ref('')
const statusError = ref(false)

onMounted(async () => {
  try {
    const result = await getLockPasswordStatus()
    statusMsg.value = result.data?.configured ? '当前已配置锁屏密码' : '尚未配置锁屏密码'
    statusError.value = false
  } catch {
    statusMsg.value = '获取状态失败'
    statusError.value = true
  }
})

const handleUpdate = async () => {
  if (!newPassword.value) {
    ElMessage.warning('请输入新密码')
    return
  }
  submitting.value = true
  try {
    await updateLockPassword(newPassword.value)
    ElMessage.success('密码已更新，已广播至所有学生端')
    statusMsg.value = '密码已更新'
    statusError.value = false
    newPassword.value = ''
  } catch (e: any) {
    ElMessage.error('更新失败: ' + (e?.response?.data?.message || e.message))
  } finally {
    submitting.value = false
  }
}
</script>

<style scoped>
.settings-page { padding: 20px; max-width: 600px; }
.page-title { font-size: 24px; font-weight: 500; color: #333; margin-bottom: 24px; }
.setting-card { margin-bottom: 20px; }
.card-title { font-size: 16px; font-weight: 500; }
.card-desc { color: #888; font-size: 13px; margin-bottom: 16px; }
.password-row { display: flex; gap: 12px; align-items: center; }
.status-msg { margin-top: 12px; font-size: 13px; color: #67c23a; }
.status-msg.error { color: #f56c6c; }
</style>
