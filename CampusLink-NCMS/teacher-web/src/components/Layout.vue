<template>
  <div class="layout">
    <el-container>
      <el-aside width="220px">
        <div class="logo">CampusLink</div>
        <el-menu
          :default-active="activeMenu"
          background-color="#1a1a2e"
          text-color="#b8b8d0"
          active-text-color="#409EFF"
          router
        >
          <el-menu-item index="/dashboard">
            <span>仪表盘</span>
          </el-menu-item>
          <el-menu-item index="/devices">
            <span>设备管理</span>
          </el-menu-item>
          <el-menu-item index="/monitor">
            <span>监控总览</span>
          </el-menu-item>
        </el-menu>
      </el-aside>
      <el-container>
        <el-header>
          <div class="header-content">
            <span class="system-name">网络教室管理系统</span>
            <span class="user-info">管理员</span>
          </div>
        </el-header>
        <el-main>
          <router-view />
        </el-main>
      </el-container>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const activeMenu = computed(() => route.path as string)
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

.el-main {
  background-color: #f5f7fa;
  padding: 20px;
}
</style>
