import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/login',
      name: 'Login',
      component: () => import('@/views/Login.vue'),
    },
    {
      path: '/',
      component: () => import('@/components/Layout.vue'),
      redirect: '/dashboard',
      children: [
        {
          path: 'dashboard',
          name: 'Dashboard',
          component: () => import('@/views/Dashboard.vue'),
        },
        {
          path: 'devices',
          name: 'Devices',
          component: () => import('@/views/Devices.vue'),
        },
        {
          path: 'monitor',
          name: 'Monitor',
          component: () => import('@/views/Monitor.vue'),
        },
        {
          path: 'attendance',
          name: 'Attendance',
          component: () => import('@/views/Attendance.vue'),
        },
        {
          path: 'alerts',
          name: 'Alerts',
          component: () => import('@/views/Alerts.vue'),
        },
        {
          path: 'hardware',
          name: 'Hardware',
          component: () => import('@/views/Hardware.vue'),
        },
        {
          path: 'logs',
          name: 'Logs',
          component: () => import('@/views/Logs.vue'),
        },
        {
          path: 'students',
          name: 'Students',
          component: () => import('@/views/Students.vue'),
        },
        {
          path: 'classes',
          name: 'Classes',
          component: () => import('@/views/Classes.vue'),
        },
        {
          path: 'settings',
          name: 'Settings',
          component: () => import('@/views/Settings.vue'),
        },
        {
          path: 'repairs',
          name: 'Repairs',
          component: () => import('@/views/Repairs.vue'),
        },
        {
          path: 'device-whitelist',
          name: 'DeviceWhitelist',
          component: () => import('@/views/DeviceWhitelist.vue'),
        },
        {
          path: 'network-accounts',
          name: 'NetworkAccounts',
          component: () => import('@/views/NetworkAccounts.vue'),
        },
      ],
    },
  ],
})

export default router
