<template>
  <div class="layout">
    <el-container>
      <el-aside width="220px">
        <div class="logo">CampusLink</div>
        <div class="sidebar-menu">
          <router-link to="/dashboard" custom v-slot="{ navigate, isActive }">
            <div class="menu-item" :class="{ active: isActive }" @click="navigate">
              <span>仪表盘</span>
            </div>
          </router-link>
          <router-link to="/devices" custom v-slot="{ navigate, isActive }">
            <div class="menu-item" :class="{ active: isActive }" @click="navigate">
              <span>设备管理</span>
            </div>
          </router-link>
          <router-link to="/monitor" custom v-slot="{ navigate, isActive }">
            <div class="menu-item" :class="{ active: isActive }" @click="navigate">
              <span>监控总览</span>
            </div>
          </router-link>
          <router-link to="/attendance" custom v-slot="{ navigate, isActive }">
            <div class="menu-item" :class="{ active: isActive }" @click="navigate">
              <span>签到管理</span>
            </div>
          </router-link>
          <router-link to="/usage" custom v-slot="{ navigate, isActive }">
            <div class="menu-item" :class="{ active: isActive }" @click="navigate">
              <span>设备使用</span>
            </div>
          </router-link>
          <router-link to="/alerts" custom v-slot="{ navigate, isActive }">
            <div class="menu-item" :class="{ active: isActive }" @click="navigate">
              <span>检查告警</span>
            </div>
          </router-link>
          <router-link to="/hardware" custom v-slot="{ navigate, isActive }">
            <div class="menu-item" :class="{ active: isActive }" @click="navigate">
              <span>硬件快照</span>
            </div>
          </router-link>
          <router-link to="/logs" custom v-slot="{ navigate, isActive }">
            <div class="menu-item" :class="{ active: isActive }" @click="navigate">
              <span>日志中心</span>
            </div>
          </router-link>
          <router-link to="/students" custom v-slot="{ navigate, isActive }">
            <div class="menu-item" :class="{ active: isActive }" @click="navigate">
              <span>学生管理</span>
            </div>
          </router-link>
          <router-link to="/classes" custom v-slot="{ navigate, isActive }">
            <div class="menu-item" :class="{ active: isActive }" @click="navigate">
              <span>班级管理</span>
            </div>
          </router-link>
          <router-link to="/settings" custom v-slot="{ navigate, isActive }">
            <div class="menu-item" :class="{ active: isActive }" @click="navigate">
              <span>系统设置</span>
            </div>
          </router-link>
          <router-link to="/repairs" custom v-slot="{ navigate, isActive }">
            <div class="menu-item" :class="{ active: isActive }" @click="navigate">
              <span>维修工单</span>
            </div>
          </router-link>
          <router-link to="/device-whitelist" custom v-slot="{ navigate, isActive }">
            <div class="menu-item" :class="{ active: isActive }" @click="navigate">
              <span>设备白名单</span>
            </div>
          </router-link>
          <router-link to="/network-accounts" custom v-slot="{ navigate, isActive }">
            <div class="menu-item" :class="{ active: isActive }" @click="navigate">
              <span>网络认证</span>
            </div>
          </router-link>
          <router-link v-if="isAdmin" to="/users" custom v-slot="{ navigate, isActive }">
            <div class="menu-item" :class="{ active: isActive }" @click="navigate">
              <span>用户管理</span>
            </div>
          </router-link>
        </div>
      </el-aside>
      <el-container>
        <el-header>
          <div class="header-content">
            <span class="system-name">网络教室管理系统</span>
            <div class="user-area">
              <span class="user-info">{{ currentDisplayName }}</span>
              <el-button link type="primary" @click="passwordVisible = true">修改密码</el-button>
              <el-button link type="primary" @click="handleLogout">退出</el-button>
            </div>
          </div>
        </el-header>
        <el-main>
          <router-view />
        </el-main>
      </el-container>
    </el-container>

    <el-dialog v-model="passwordVisible" title="修改密码" width="420px">
      <el-form ref="passwordRef" :model="passwordForm" :rules="passwordRules" label-width="90px">
        <el-form-item label="当前密码" prop="oldPassword">
          <el-input v-model="passwordForm.oldPassword" type="password" show-password />
        </el-form-item>
        <el-form-item label="新密码" prop="newPassword">
          <el-input v-model="passwordForm.newPassword" type="password" show-password placeholder="不少于 4 位" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="passwordVisible = false">取消</el-button>
        <el-button type="primary" :loading="passwordSaving" @click="handleChangePassword">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useWebSocket } from '@/composables/useWebSocket'
import { changeOwnPassword } from '@/api/users'

const { connect, disconnect } = useWebSocket()
const router = useRouter()

const currentUser = (() => {
  try {
    const raw = localStorage.getItem('user')
    return raw ? JSON.parse(raw) : {}
  } catch {
    return {}
  }
})()

const isAdmin = computed(() => currentUser.role === 'admin')
const currentDisplayName = computed(() => currentUser.displayName || currentUser.username || '用户')

const passwordVisible = ref(false)
const passwordSaving = ref(false)
const passwordRef = ref()
const passwordForm = reactive({
  oldPassword: '',
  newPassword: '',
})
const passwordRules = {
  oldPassword: [{ required: true, message: '请输入当前密码', trigger: 'blur' }],
  newPassword: [
    { required: true, message: '请输入新密码', trigger: 'blur' },
    { min: 4, message: '新密码不少于 4 位', trigger: 'blur' },
  ],
}

async function handleChangePassword() {
  const valid = await passwordRef.value?.validate().catch(() => false)
  if (!valid) return

  passwordSaving.value = true
  try {
    const result = await changeOwnPassword(passwordForm.oldPassword, passwordForm.newPassword)
    if (result.code !== 0) {
      ElMessage.error(result.message || '修改失败')
      return
    }
    ElMessage.success('密码已修改')
    passwordVisible.value = false
    passwordForm.oldPassword = ''
    passwordForm.newPassword = ''
  } catch (error) {
    const data = (error as { response?: { data?: { message?: string } } })?.response?.data
    ElMessage.error(data?.message || '修改失败')
  } finally {
    passwordSaving.value = false
  }
}

function handleLogout() {
  disconnect()
  localStorage.removeItem('token')
  localStorage.removeItem('user')
  router.push('/login')
}

onMounted(() => {
  connect()
})
</script>

<style scoped>
.layout {
  height: 100%;
}

.el-container {
  height: 100%;
}

.el-aside {
  background-color: #1a1a2e;
  color: #fff;
  overflow-y: auto;
}

.sidebar-menu {
  padding: 8px 0;
}

.menu-item {
  height: 56px;
  line-height: 56px;
  padding: 0 20px;
  color: #b8b8d0;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s;
}

.menu-item:hover {
  color: #fff;
  background-color: rgba(255, 255, 255, 0.05);
}

.menu-item.active {
  color: #409EFF;
  background-color: rgba(64, 158, 255, 0.1);
}

.logo {
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  font-weight: bold;
  color: #fff;
  border-bottom: 1px solid #2d2d44;
}

.el-menu {
  border-right: none;
}

.el-header {
  background-color: #fff;
  border-bottom: 1px solid #e0e0e0;
  display: flex;
  align-items: center;
  padding: 0 20px;
}

.header-content {
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.system-name {
  font-size: 18px;
  font-weight: 500;
}

.user-info {
  color: #666;
}

.user-area {
  display: flex;
  align-items: center;
  gap: 12px;
}

.el-main {
  background-color: #f5f7fa;
  padding: 20px;
}
</style>
