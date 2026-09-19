<template>
  <div class="classes">
    <div class="header">
      <h2 class="page-title">班级管理</h2>
      <div class="header-actions">
        <el-button type="primary" @click="openCreate">新建班级</el-button>
      </div>
    </div>

    <el-row :gutter="16">
      <el-col :span="7">
        <el-card>
          <template #header>
            <span>班级列表</span>
          </template>
          <el-empty v-if="classes.length === 0" description="暂无班级" />
          <div
            v-for="c in classes"
            :key="c.id"
            class="class-item"
            :class="{ active: selected?.id === c.id }"
            @click="selectClass(c)"
          >
            <span class="class-name">{{ c.name }}</span>
            <span class="class-ops" @click.stop>
              <el-button link type="primary" size="small" @click="openRename(c)">重命名</el-button>
              <el-button link type="danger" size="small" @click="removeClass(c)">删除</el-button>
            </span>
          </div>
        </el-card>
      </el-col>

      <el-col :span="17">
        <el-card v-loading="detailLoading">
          <el-empty v-if="!selected" description="请选择或新建一个班级" />
          <div v-else>
            <div class="detail-header">
              <h3>{{ selected.name }}</h3>
              <div>
                <el-button size="small" @click="openAssignStudents">添加学生</el-button>
                <el-button size="small" @click="openAssignDevices">添加学生机</el-button>
                <el-button size="small" type="warning" @click="resetPasswords">批量重置密码</el-button>
              </div>
            </div>

            <el-divider content-position="left">模式切换</el-divider>
            <div class="mode-actions">
              <el-button
                v-for="m in modes"
                :key="m.value"
                :type="selectedMode === m.value ? 'primary' : 'default'"
                size="small"
                @click="changeMode(m.value)"
              >
                {{ m.label }}
              </el-button>
              <span class="hint">授课模式需班级内已有学生，切换后自动下发签到</span>
            </div>

            <el-divider content-position="left">学生（{{ classStudents.length }}）</el-divider>
            <el-table :data="classStudents" size="small" border>
              <el-table-column prop="studentNo" label="学号" width="130" />
              <el-table-column prop="name" label="姓名" width="120" />
              <el-table-column prop="seatNo" label="座位号" width="90">
                <template #default="{ row }">{{ row.seatNo || '-' }}</template>
              </el-table-column>
              <el-table-column label="密码状态">
                <template #default="{ row }">
                  <el-tag :type="row.passwordSet ? 'success' : 'warning'" size="small">
                    {{ row.passwordSet ? '已自定义' : '初始密码' }}
                  </el-tag>
                </template>
              </el-table-column>
            </el-table>

            <el-divider content-position="left">
              学生机（{{ classDevices.length }}）
              <el-button link type="primary" size="small" @click="saveSeats">保存座位号</el-button>
              <el-button link type="primary" size="small" @click="renumberSeatNos">顺序编号</el-button>
              <el-button link type="primary" size="small" @click="exportSeats">导出座位表</el-button>
              <el-button link type="primary" size="small" @click="triggerImportSeats">导入座位表</el-button>
              <el-button link type="info" size="small" @click="autoSeats">按 IP 编号</el-button>
            </el-divider>
            <input ref="seatFileInput" type="file" accept=".xlsx" class="hidden-file" @change="onImportSeats" />
            <el-table :data="classDevices" size="small" border>
              <el-table-column prop="deviceName" label="设备名" width="140" />
              <el-table-column prop="ipAddress" label="IP" width="140" />
              <el-table-column label="在线" width="80">
                <template #default="{ row }">
                  <el-tag :type="row.onlineStatus === 'online' ? 'success' : 'info'" size="small">
                    {{ row.onlineStatus === 'online' ? '在线' : '离线' }}
                  </el-tag>
                </template>
              </el-table-column>
              <el-table-column label="当前模式" width="100">
                <template #default="{ row }">{{ modeLabel(row.currentMode) }}</template>
              </el-table-column>
              <el-table-column label="座位号" width="140">
                <template #default="{ row }">
                  <el-input v-model="seatEdits[row.id]" size="small" placeholder="座位号" />
                </template>
              </el-table-column>
            </el-table>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <el-dialog v-model="nameDialogVisible" :title="editingId ? '重命名班级' : '新建班级'" width="380px">
      <el-input v-model="nameInput" placeholder="请输入班级名称" />
      <template #footer>
        <el-button @click="nameDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitting" @click="submitName">确定</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="studentDialogVisible" title="添加学生" width="520px">
      <el-select v-model="pendingStudentIds" multiple filterable placeholder="选择学生" style="width: 100%">
        <el-option
          v-for="s in assignableStudents"
          :key="s.id"
          :label="`${s.name}（${s.studentNo}）`"
          :value="s.id"
        />
      </el-select>
      <template #footer>
        <el-button @click="studentDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitting" @click="submitAssignStudents">确定</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="deviceDialogVisible" title="添加学生机" width="720px">
      <el-table
        :data="assignableDevices"
        size="small"
        border
        max-height="420"
        @selection-change="onDeviceSelection"
      >
        <el-table-column type="selection" width="48" />
        <el-table-column prop="deviceName" label="设备名" />
        <el-table-column prop="ipAddress" label="IP" width="150" />
        <el-table-column label="在线" width="90">
          <template #default="{ row }">
            <el-tag :type="row.onlineStatus === 'online' ? 'success' : 'info'" size="small">
              {{ row.onlineStatus === 'online' ? '在线' : '离线' }}
            </el-tag>
          </template>
        </el-table-column>
      </el-table>
      <template #footer>
        <el-button @click="deviceDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitting" @click="submitAssignDevices">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  listClasses, createClass, updateClass, deleteClass,
  listClassStudents, assignStudents, listClassDevices, assignDevices,
  batchSetSeats, autoAssignSeats, renumberSeats, exportClassSeats, importClassSeats,
  resetClassPasswords, switchClassMode,
  type ClassInfo, type SeatAssignment,
} from '@/api/classes'
import { listStudents, type Student } from '@/api/students'
import { listDevices, type Device } from '@/api/devices'

const classes = ref<ClassInfo[]>([])
const selected = ref<ClassInfo | null>(null)
const selectedMode = ref('')
const classStudents = ref<Student[]>([])
const classDevices = ref<Device[]>([])
const detailLoading = ref(false)
const seatEdits = ref<Record<string, string>>({})

const allStudents = ref<Student[]>([])
const allDevices = ref<Device[]>([])
const assignableStudents = ref<Student[]>([])
const assignableDevices = ref<Device[]>([])

const nameDialogVisible = ref(false)
const editingId = ref('')
const nameInput = ref('')
const submitting = ref(false)

const studentDialogVisible = ref(false)
const deviceDialogVisible = ref(false)
const pendingStudentIds = ref<string[]>([])
const pendingDeviceIds = ref<string[]>([])
const seatFileInput = ref<HTMLInputElement | null>(null)

const modes = [
  { value: 'open', label: '开放模式' },
  { value: 'teaching', label: '授课模式' },
  { value: 'exam', label: '考试模式' },
  { value: 'locked', label: '锁定模式' },
]

function modeLabel(mode: string) {
  return modes.find((m) => m.value === mode)?.label || mode
}

async function loadClasses() {
  const res = await listClasses()
  if (res.code === 0 && res.data) {
    classes.value = res.data
    if (selected.value) {
      selected.value = classes.value.find((c) => c.id === selected.value!.id) || null
    }
  }
}

async function selectClass(c: ClassInfo) {
  selected.value = c
  selectedMode.value = ''
  seatEdits.value = {}
  await loadDetail()
}

async function loadDetail() {
  if (!selected.value) return
  detailLoading.value = true
  try {
    const [stuRes, devRes] = await Promise.all([
      listClassStudents(selected.value.id),
      listClassDevices(selected.value.id),
    ])
    classStudents.value = stuRes.code === 0 && stuRes.data ? stuRes.data : []
    classDevices.value = devRes.code === 0 && devRes.data ? devRes.data : []
    const edits: Record<string, string> = {}
    for (const d of classDevices.value) edits[d.id] = d.seatNo || ''
    seatEdits.value = edits
  } finally {
    detailLoading.value = false
  }
}

function openCreate() {
  editingId.value = ''
  nameInput.value = ''
  nameDialogVisible.value = true
}

function openRename(c: ClassInfo) {
  editingId.value = c.id
  nameInput.value = c.name
  nameDialogVisible.value = true
}

async function submitName() {
  if (!nameInput.value.trim()) {
    ElMessage.warning('班级名称不能为空')
    return
  }
  submitting.value = true
  try {
    if (editingId.value) {
      await updateClass(editingId.value, nameInput.value.trim())
      ElMessage.success('重命名成功')
    } else {
      await createClass(nameInput.value.trim())
      ElMessage.success('创建成功')
    }
    nameDialogVisible.value = false
    await loadClasses()
  } catch (e: any) {
    ElMessage.error('操作失败：' + (e?.response?.data?.message || e?.message || ''))
  } finally {
    submitting.value = false
  }
}

async function removeClass(c: ClassInfo) {
  try {
    await ElMessageBox.confirm(`确认删除班级「${c.name}」？学生与设备将解除关联。`, '删除确认', { type: 'warning' })
    await deleteClass(c.id)
    if (selected.value?.id === c.id) selected.value = null
    ElMessage.success('删除成功')
    await loadClasses()
  } catch {
    // cancelled
  }
}

async function openAssignStudents() {
  await ensureAll()
  const inClass = new Set(classStudents.value.map((s) => s.id))
  assignableStudents.value = allStudents.value.filter((s) => !inClass.has(s.id))
  pendingStudentIds.value = []
  studentDialogVisible.value = true
}

async function openAssignDevices() {
  await ensureAll(true)
  const inClass = new Set(classDevices.value.map((d) => d.id))
  assignableDevices.value = allDevices.value.filter((d) => !inClass.has(d.id))
  pendingDeviceIds.value = []
  deviceDialogVisible.value = true
}

function onDeviceSelection(rows: Device[]) {
  pendingDeviceIds.value = rows.map((d) => d.id)
}

async function ensureAll(reloadDevices = false) {
  if (allStudents.value.length === 0) {
    const res = await listStudents({ page: 1, pageSize: 100 })
    if (res.code === 0 && res.data) allStudents.value = res.data.students
  }
  if (reloadDevices || allDevices.value.length === 0) {
    const res = await listDevices()
    if (res.code === 0 && res.data) allDevices.value = res.data
  }
}

async function submitAssignStudents() {
  if (!selected.value || pendingStudentIds.value.length === 0) {
    studentDialogVisible.value = false
    return
  }
  submitting.value = true
  try {
    const res = await assignStudents(selected.value.id, pendingStudentIds.value)
    if (res.code !== 0) {
      ElMessage.error(res.message || '添加失败')
      return
    }
    ElMessage.success('已添加学生')
    studentDialogVisible.value = false
    await loadDetail()
  } catch (e: any) {
    ElMessage.error('添加失败：' + (e?.message || ''))
  } finally {
    submitting.value = false
  }
}

async function submitAssignDevices() {
  if (!selected.value || pendingDeviceIds.value.length === 0) {
    deviceDialogVisible.value = false
    return
  }
  submitting.value = true
  try {
    const res = await assignDevices(selected.value.id, pendingDeviceIds.value)
    if (res.code !== 0) {
      ElMessage.error(res.message || '添加学生机失败')
      return
    }
    ElMessage.success('已添加学生机')
    deviceDialogVisible.value = false
    await loadDetail()
  } catch (e: any) {
    ElMessage.error('添加失败：' + (e?.response?.data?.message || e?.message || ''))
  } finally {
    submitting.value = false
  }
}

async function saveSeats() {
  if (!selected.value) return
  const seats: SeatAssignment[] = Object.entries(seatEdits.value)
    .filter(([, seat]) => seat && seat.trim())
    .map(([deviceId, seatNo]) => ({ deviceId, seatNo: seatNo.trim() }))
  try {
    const res = await batchSetSeats(selected.value.id, seats)
    if (res.code !== 0) {
      ElMessage.error(res.message || '保存失败')
      return
    }
    ElMessage.success('座位号已保存')
    await loadDetail()
  } catch (e: any) {
    ElMessage.error('保存失败：' + (e?.response?.data?.message || e?.message || ''))
  }
}

async function autoSeats() {
  if (!selected.value) return
  try {
    const res = await autoAssignSeats(selected.value.id)
    if (res.code === 0 && res.data) {
      ElMessage.success(`自动分配 ${res.data.assigned.length} 个座位`)
      await loadDetail()
    } else {
      ElMessage.error(res.message || '自动分配失败')
    }
  } catch (e: any) {
    ElMessage.error('自动分配失败：' + (e?.message || ''))
  }
}

function saveBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  a.click()
  URL.revokeObjectURL(url)
}

async function renumberSeatNos() {
  if (!selected.value) return
  try {
    const res = await renumberSeats(selected.value.id)
    if (res.code !== 0) {
      ElMessage.error(res.message || '顺序编号失败')
      return
    }
    ElMessage.success(`已按列表顺序编号 ${res.data?.length ?? 0} 台学生机`)
    await loadDetail()
  } catch (e: any) {
    ElMessage.error('顺序编号失败：' + (e?.message || ''))
  }
}

async function exportSeats() {
  if (!selected.value) return
  try {
    const blob = await exportClassSeats(selected.value.id)
    saveBlob(blob, `${selected.value.name}-seats.xlsx`)
  } catch (e: any) {
    ElMessage.error('导出失败：' + (e?.message || ''))
  }
}

function triggerImportSeats() {
  if (seatFileInput.value) {
    seatFileInput.value.value = ''
    seatFileInput.value.click()
  }
}

async function onImportSeats(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file || !selected.value) return
  try {
    const res = await importClassSeats(selected.value.id, file)
    if (res.code !== 0) {
      ElMessage.error(res.message || '导入失败')
      return
    }
    ElMessage.success(`已导入 ${res.data?.imported ?? 0} 条座位号`)
    await loadDetail()
  } catch (e: any) {
    ElMessage.error('导入失败：' + (e?.response?.data?.message || e?.message || ''))
  }
}

async function resetPasswords() {
  if (!selected.value) return
  try {
    const { value } = await ElMessageBox.prompt(
      '为班级内所有学生设置初始密码，留空则重置为 123456',
      '批量重置密码',
      { inputPlaceholder: '123456', confirmButtonText: '确定', cancelButtonText: '取消' }
    )
    await resetClassPasswords(selected.value.id, value || undefined)
    ElMessage.success('密码已重置')
    await loadDetail()
  } catch {
    // cancelled
  }
}

async function changeMode(mode: string) {
  if (!selected.value) return
  try {
    const res = await switchClassMode(selected.value.id, mode)
    if (res.code !== 0) {
      ElMessage.error(res.message || '切换失败')
      return
    }
    selectedMode.value = mode
    const teachingCount = res.data?.teachingCount ?? 0
    const lockedCount = res.data?.lockedCount ?? 0
    if (mode === 'teaching') {
      ElMessage.success(`授课模式完成：${teachingCount} 台进入授课，${lockedCount} 台进入锁定`)
    } else {
      ElMessage.success(`已切换到${modeLabel(mode)}`)
    }
    await loadDetail()
  } catch (e: any) {
    ElMessage.error(e?.response?.data?.message || e?.message || '切换失败')
  }
}

onMounted(() => {
  loadClasses()
})
</script>

<style scoped>
.classes { padding: 20px; }
.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.page-title { font-size: 24px; font-weight: 500; color: #333; }
.class-item {
  display: flex; justify-content: space-between; align-items: center;
  padding: 10px 12px; border-radius: 4px; cursor: pointer; margin-bottom: 6px;
}
.class-item:hover { background: #f5f7fa; }
.class-item.active { background: #ecf5ff; }
.class-name { font-weight: 500; }
.detail-header { display: flex; justify-content: space-between; align-items: center; }
.mode-actions { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
 .hint { color: #909399; font-size: 12px; margin-left: 8px; }
 .hidden-file { display: none; }
</style>
