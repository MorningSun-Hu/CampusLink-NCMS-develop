<template>
  <div class="alerts-page">
    <div class="header">
      <h2 class="page-title">检查与告警</h2>
      <el-button @click="loadData">刷新</el-button>
    </div>

    <el-tabs v-model="activeTab">
      <el-tab-pane label="检查记录" name="inspections">
        <div class="filters">
          <el-select v-model="inspectionFilter.type" placeholder="检查类型" clearable @change="loadInspections">
            <el-option label="卫生检查" value="hygiene" />
            <el-option label="设备检查" value="equipment" />
            <el-option label="键盘检查" value="keyboard" />
            <el-option label="鼠标检查" value="mouse" />
          </el-select>
        </div>
        <el-table :data="inspections" v-loading="inspLoading" border stripe>
          <el-table-column prop="device_id" label="设备ID" width="200" />
          <el-table-column prop="inspection_type" label="检查类型">
            <template #default="{ row }">
              <el-tag>{{ inspectionTypeLabel(row.inspection_type) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="item_name" label="项目" />
          <el-table-column prop="status" label="状态">
            <template #default="{ row }">
              <el-tag :type="row.status === 'normal' ? 'success' : 'danger'">
                {{ row.status === 'normal' ? '正常' : '异常' }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="is_abnormal" label="是否异常" width="100">
            <template #default="{ row }">
              <el-tag :type="row.is_abnormal ? 'danger' : 'success'">
                {{ row.is_abnormal ? '是' : '否' }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="description" label="描述">
            <template #default="{ row }">
              {{ row.description || '-' }}
            </template>
          </el-table-column>
          <el-table-column prop="created_at" label="时间" width="180" />
        </el-table>
      </el-tab-pane>

      <el-tab-pane label="告警列表" name="alerts">
        <div class="filters">
          <el-select v-model="alertFilter.resolved" placeholder="处理状态" clearable @change="loadAlerts">
            <el-option label="未处理" value="false" />
            <el-option label="已处理" value="true" />
          </el-select>
        </div>
        <el-table :data="alerts" v-loading="alertLoading" border stripe>
          <el-table-column prop="device_id" label="设备ID" width="200" />
          <el-table-column prop="alert_type" label="告警类型">
            <template #default="{ row }">
              <el-tag type="warning">{{ row.alert_type }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="description" label="描述" />
          <el-table-column prop="resolved" label="处理状态" width="100">
            <template #default="{ row }">
              <el-tag :type="row.resolved ? 'success' : 'danger'">
                {{ row.resolved ? '已处理' : '未处理' }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="created_at" label="创建时间" width="180" />
          <el-table-column label="操作" width="160">
            <template #default="{ row }">
              <el-button
                v-if="!row.resolved"
                type="primary"
                size="small"
                @click="handleResolve(row)"
              >
                处理
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>

      <el-tab-pane label="图片管理" name="photos">
        <div class="photo-upload-area">
          <el-upload
            :auto-upload="true"
            :http-request="customUpload"
            :show-file-list="true"
            accept="image/*"
          >
            <el-button type="primary">上传图片</el-button>
          </el-upload>
        </div>
        <div class="photo-grid" v-if="photos.length > 0">
          <div v-for="photo in photos" :key="photo.id" class="photo-card">
            <img :src="getPhotoUrl(photo.id)" :alt="photo.original_name" class="photo-img" />
            <div class="photo-info">
              <span class="photo-name">{{ photo.original_name }}</span>
              <span class="photo-size">{{ formatFileSize(photo.file_size) }}</span>
            </div>
          </div>
        </div>
        <el-empty v-else description="暂无图片" />
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { UploadRequestOptions } from 'element-plus'
import { getInspectionList } from '@/api/inspection'
import { getAlertList, resolveAlert } from '@/api/inspection'
import { uploadPhoto, getPhotoList, getPhotoUrl } from '@/api/photos'
import type { InspectionRecord, AlertRecord, PhotoRecord } from '@/api/types'

const activeTab = ref('inspections')

const inspections = ref<InspectionRecord[]>([])
const inspLoading = ref(false)
const inspectionFilter = ref({ type: '' })

const alerts = ref<AlertRecord[]>([])
const alertLoading = ref(false)
const alertFilter = ref({ resolved: '' })

const photos = ref<PhotoRecord[]>([])

async function loadInspections() {
  inspLoading.value = true
  try {
    const params: Record<string, string> = {}
    if (inspectionFilter.value.type) params.inspection_type = inspectionFilter.value.type
    const res = await getInspectionList(params)
    if (res.code === 0 && res.data) {
      inspections.value = res.data
    }
  } catch (e: any) {
    ElMessage.error('获取检查记录失败: ' + (e?.message || '未知错误'))
  } finally {
    inspLoading.value = false
  }
}

async function loadAlerts() {
  alertLoading.value = true
  try {
    const params: Record<string, string> = {}
    if (alertFilter.value.resolved) params.resolved = alertFilter.value.resolved
    const res = await getAlertList(params)
    if (res.code === 0 && res.data) {
      alerts.value = res.data
    }
  } catch (e: any) {
    ElMessage.error('获取告警列表失败: ' + (e?.message || '未知错误'))
  } finally {
    alertLoading.value = false
  }
}

async function loadPhotos() {
  try {
    const res = await getPhotoList()
    if (res.code === 0 && res.data) {
      photos.value = res.data
    }
  } catch (e: any) {
    ElMessage.error('获取图片列表失败: ' + (e?.message || '未知错误'))
  }
}

async function handleResolve(row: AlertRecord) {
  try {
    await ElMessageBox.confirm('确认处理该告警？', '处理告警', { type: 'warning' })
  } catch {
    return
  }
  try {
    const res = await resolveAlert(row.id)
    if (res.code === 0) {
      ElMessage.success('告警已处理')
      loadAlerts()
    } else {
      ElMessage.error(res.message || '处理失败')
    }
  } catch (e: any) {
    ElMessage.error('处理失败: ' + (e?.message || '未知错误'))
  }
}

async function customUpload(options: UploadRequestOptions) {
  try {
    const res = await uploadPhoto(options.file as File)
    if (res.code === 0) {
      ElMessage.success('上传成功')
      loadPhotos()
    } else {
      ElMessage.error(res.message || '上传失败')
    }
  } catch (e: any) {
    ElMessage.error('上传失败: ' + (e?.message || '未知错误'))
  }
}

function loadData() {
  loadInspections()
  loadAlerts()
  loadPhotos()
}

function inspectionTypeLabel(type: string) {
  const map: Record<string, string> = {
    hygiene: '卫生检查',
    equipment: '设备检查',
    keyboard: '键盘检查',
    mouse: '鼠标检查',
  }
  return map[type] || type
}

async function compressImage(file: File, maxWidth = 1200, quality = 0.7): Promise<Blob> {
  if (!file.type.startsWith('image/')) return file
  return new Promise((resolve) => {
    const img = new Image()
    img.onload = () => {
      let w = img.width, h = img.height
      if (w > maxWidth) {
        h = Math.round(h * maxWidth / w)
        w = maxWidth
      }
      const canvas = document.createElement('canvas')
      canvas.width = w
      canvas.height = h
      const ctx = canvas.getContext('2d')!
      ctx.drawImage(img, 0, 0, w, h)
      canvas.toBlob((blob) => resolve(blob || file), file.type, quality)
    }
    img.onerror = () => resolve(file)
    img.src = URL.createObjectURL(file)
  })
}

async function customUpload(options: UploadRequestOptions) {
  try {
    const compressed = await compressImage(options.file as File)
    const res = await uploadPhoto(new File([compressed], (options.file as File).name, { type: (options.file as File).type }))
    if (res.code === 0) {
      ElMessage.success('上传成功')
      loadPhotos()
    } else {
      ElMessage.error(res.message || '上传失败')
    }
  } catch (e: any) {
    ElMessage.error('上传失败: ' + (e?.message || '未知错误'))
  }
}
  return map[type] || type
}

function formatFileSize(bytes: number) {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

onMounted(() => {
  loadData()
})
</script>

<style scoped>
.alerts-page {
  background: #fff;
  border-radius: 4px;
  padding: 20px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.page-title {
  font-size: 20px;
  margin: 0;
}

.filters {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
}

.photo-upload-area {
  margin-bottom: 20px;
  padding: 16px;
  border: 1px dashed #d9d9d9;
  border-radius: 4px;
  text-align: center;
}

.photo-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 16px;
  margin-top: 16px;
}

.photo-card {
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  overflow: hidden;
}

.photo-img {
  width: 100%;
  height: 160px;
  object-fit: cover;
  display: block;
}

.photo-info {
  padding: 8px 12px;
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: #666;
}

.photo-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 120px;
}
</style>
