<template>
  <div class="dashboard">
    <h2 class="page-title">仪表盘</h2>
    <el-row :gutter="20">
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-label">学生总数</div>
          <div class="stat-value">{{ stats.studentCount || 0 }}</div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-label">已注册设备数</div>
          <div class="stat-value">{{ stats.registeredDeviceCount || 0 }}</div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-label">在线设备数</div>
          <div class="stat-value online">{{ stats.onlineDeviceCount || 0 }}</div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-label">离线设备数</div>
          <div class="stat-value offline">{{ stats.offlineDeviceCount || 0 }}</div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getDashboardOverview } from '@/api/dashboard'

const stats = ref({
  studentCount: 0,
  registeredDeviceCount: 0,
  onlineDeviceCount: 0,
  offlineDeviceCount: 0,
})

const loadStats = async () => {
  try {
    const result = await getDashboardOverview()
    if (result.data) {
      stats.value = result.data
    }
  } catch (error) {
    console.error('Load dashboard failed:', error)
  }
}

onMounted(() => {
  loadStats()
})
</script>

<style scoped>
.dashboard {
  padding: 20px;
}

.page-title {
  font-size: 24px;
  font-weight: 500;
  margin-bottom: 20px;
  color: #333;
}

.stat-card {
  text-align: center;
  padding: 20px;
}

.stat-label {
  font-size: 14px;
  color: #666;
  margin-bottom: 10px;
}

.stat-value {
  font-size: 32px;
  font-weight: bold;
  color: #333;
}

.stat-value.online {
  color: #67c23a;
}

.stat-value.offline {
  color: #f56c6c;
}
</style>
