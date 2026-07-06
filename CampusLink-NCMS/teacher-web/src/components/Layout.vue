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
        </div>
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

.el-main {
  background-color: #f5f7fa;
  padding: 20px;
}
</style>
