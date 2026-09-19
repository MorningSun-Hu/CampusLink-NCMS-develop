<template>
  <div class="students">
    <div class="header">
      <h2 class="page-title">学生管理</h2>
      <div class="header-actions">
        <el-input v-model="keyword" placeholder="搜索学号/姓名" clearable style="width: 220px" @change="loadStudents" />
        <el-select v-model="selectedClassId" placeholder="按班级筛选" clearable style="width: 180px" @change="loadStudents">
          <el-option v-for="c in classes" :key="c.id" :label="c.name" :value="c.id" />
        </el-select>
        <el-button @click="handleDownloadTemplate">下载模板</el-button>
        <el-upload :show-file-list="false" :before-upload="handleImport" accept=".xlsx" style="display: inline-flex">
          <el-button type="warning">批量导入</el-button>
        </el-upload>
        <el-button type="success" @click="handleExport">导出 Excel</el-button>
        <el-button type="primary" @click="showCreateDialog">新增学生</el-button>
      </div>
    </div>

    <el-table :data="students" v-loading="loading" border stripe>
      <el-table-column prop="studentNo" label="学号" width="130" />
      <el-table-column prop="name" label="姓名" width="110" />
      <el-table-column prop="className" label="班级" width="130">
        <template #default="{ row }">{{ row.className || '-' }}</template>
      </el-table-column>
      <el-table-column prop="seatNo" label="座位号" width="90">
        <template #default="{ row }">{{ row.seatNo || '-' }}</template>
      </el-table-column>
      <el-table-column prop="status" label="状态" width="90">
        <template #default="{ row }">
          <el-tag :type="row.status === 'active' ? 'success' : 'info'">
            {{ row.status === 'active' ? '正常' : row.status }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column label="密码状态" width="110">
        <template #default="{ row }">
          <el-tag :type="row.passwordSet ? 'success' : 'warning'">
            {{ row.passwordSet ? '已自定义' : '初始密码' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="createdAt" label="创建时间" width="180">
        <template #default="{ row }">{{ row.createdAt || '-' }}</template>
      </el-table-column>
      <el-table-column label="操作" width="280">
        <template #default="{ row }">
          <el-button type="primary" size="small" @click="showEditDialog(row)">编辑</el-button>
          <el-button type="warning" size="small" @click="handleResetPassword(row)">重置密码</el-button>
          <el-button type="danger" size="small" @click="handleDelete(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <div class="pagination">
      <el-pagination
        v-model:current-page="page"
        :page-size="pageSize"
        :total="total"
        layout="prev, pager, next, total"
        @current-change="loadStudents"
      />
    </div>

    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑学生' : '新增学生'" width="480px">
      <el-form :model="form" label-width="80px" :rules="rules" ref="formRef">
        <el-form-item label="学号" prop="studentNo">
          <el-input v-model="form.studentNo" placeholder="留空自动生成" :disabled="isEdit" />
        </el-form-item>
        <el-form-item label="姓名" prop="name">
          <el-input v-model="form.name" placeholder="请输入姓名" />
        </el-form-item>
        <el-form-item label="班级" prop="classId">
          <el-select v-model="form.classId" placeholder="请选择班级" style="width: 100%">
            <el-option v-for="c in classes" :key="c.id" :label="c.name" :value="c.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="密码" :prop="isEdit ? '' : 'password'">
          <el-input v-model="form.password" placeholder="请输入密码（留空不修改）" show-password />
        </el-form-item>
        <el-form-item label="座位号" prop="seatNo">
          <el-select v-model="form.seatNo" placeholder="请选择已分配的座位号" filterable style="width: 100%" :disabled="!form.classId">
            <el-option v-for="seat in seatOptions" :key="seat" :label="seat" :value="seat" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSubmit" :loading="submitting">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch, onMounted } from 'vue'
import { listStudents, createStudent, updateStudent, deleteStudent, resetStudentPassword, importStudents, exportStudents, downloadStudentTemplate, type Student } from '@/api/students'
import { listClasses, listClassDevices, type ClassInfo } from '@/api/classes'
import { ElMessage, ElMessageBox } from 'element-plus'

const students = ref<Student[]>([])
const classes = ref<ClassInfo[]>([])
const loading = ref(false)
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const keyword = ref('')
const selectedClassId = ref('')

const dialogVisible = ref(false)
const isEdit = ref(false)
const submitting = ref(false)
const editingId = ref('')
const formRef = ref()

const form = reactive({
  studentNo: '',
  name: '',
  classId: '',
  password: '',
  seatNo: '',
})

const seatOptions = ref<string[]>([])

const rules = {
  name: [{ required: true, message: '请输入姓名', trigger: 'blur' }],
  classId: [{ required: true, message: '请选择班级', trigger: 'change' }],
  seatNo: [{ required: true, message: '请选择座位号', trigger: 'change' }],
}

function apiErrorMessage(error: unknown, fallback: string) {
  const data = (error as { response?: { data?: { message?: string } } })?.response?.data
  return data?.message || fallback
}

function saveBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  a.click()
  URL.revokeObjectURL(url)
}

async function loadSeats(classId: string) {
  if (!classId) {
    seatOptions.value = []
    return
  }
  try {
    const result = await listClassDevices(classId)
    const seats = (result.data || [])
      .map((d) => (d.seatNo || '').trim())
      .filter((s) => s.length > 0)
    seatOptions.value = Array.from(new Set(seats)).sort()
  } catch (error) {
    console.error('Load seats failed:', error)
    seatOptions.value = []
  }
}

watch(() => form.classId, async (classId, previous) => {
  await loadSeats(classId)
  if (previous !== undefined && !seatOptions.value.includes(form.seatNo)) {
    form.seatNo = ''
  }
})

const loadClasses = async () => {
  try {
    const result = await listClasses()
    if (result.data) classes.value = result.data
  } catch (error) {
    console.error('Load classes failed:', error)
  }
}

const loadStudents = async () => {
  loading.value = true
  try {
    const result = await listStudents({
      page: page.value,
      pageSize: pageSize.value,
      keyword: keyword.value,
      classId: selectedClassId.value || undefined,
    })
    if (result.data) {
      students.value = result.data.students
      total.value = result.data.total
    }
  } catch (error) {
    console.error('Load students failed:', error)
  } finally {
    loading.value = false
  }
}

const showCreateDialog = () => {
  isEdit.value = false
  editingId.value = ''
  form.studentNo = ''
  form.name = ''
  form.classId = selectedClassId.value || ''
  form.password = ''
  form.seatNo = ''
  dialogVisible.value = true
}

const showEditDialog = (row: Student) => {
  isEdit.value = true
  editingId.value = row.id
  form.studentNo = row.studentNo
  form.name = row.name
  form.classId = row.classId || ''
  form.password = ''
  form.seatNo = row.seatNo || ''
  dialogVisible.value = true
}

const handleSubmit = async () => {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return

  submitting.value = true
  try {
    if (isEdit.value) {
      const data: { name: string; seatNo: string; classId: string; password?: string } = {
        name: form.name,
        seatNo: form.seatNo,
        classId: form.classId,
      }
      if (form.password) data.password = form.password
      const result = await updateStudent(editingId.value, data)
      if (result.code !== 0) {
        ElMessage.error(result.message || '操作失败')
        return
      }
      ElMessage.success('更新成功')
    } else {
      const result = await createStudent({
        studentNo: form.studentNo || undefined,
        name: form.name,
        password: form.password || undefined,
        seatNo: form.seatNo,
        classId: form.classId,
      })
      if (result.code !== 0) {
        ElMessage.error(result.message || '操作失败')
        return
      }
      ElMessage.success('创建成功')
    }
    dialogVisible.value = false
    loadStudents()
  } catch (error) {
    console.error('Submit failed:', error)
    ElMessage.error(apiErrorMessage(error, '操作失败'))
  } finally {
    submitting.value = false
  }
}

const handleResetPassword = async (row: Student) => {
  try {
    const { value } = await ElMessageBox.prompt(
      `为 ${row.name}（${row.studentNo}）设置新的初始密码，留空则重置为 123456`,
      '重置密码',
      { inputPlaceholder: '123456', confirmButtonText: '确定', cancelButtonText: '取消' }
    )
    await resetStudentPassword(row.id, value || undefined)
    ElMessage.success('密码已重置为初始密码')
    loadStudents()
  } catch (error) {
    // cancelled
  }
}

const handleDelete = async (row: Student) => {
  try {
    await ElMessageBox.confirm(`确认删除学生 ${row.name}（${row.studentNo}）？`, '删除确认', { type: 'warning' })
    await deleteStudent(row.id)
    ElMessage.success('删除成功')
    loadStudents()
  } catch (error) {
    // cancelled
  }
}

const handleImport = async (file: File) => {
  try {
    const result = await importStudents(file)
    if (result.code === 0 && result.data?.success) {
      ElMessage.success(`成功导入 ${result.data.imported} 名学生`)
      loadStudents()
    } else {
      ElMessage.error(result.message || '导入失败')
    }
  } catch (error) {
    ElMessage.error(apiErrorMessage(error, '导入失败'))
  }
  return false
}

const handleDownloadTemplate = async () => {
  try {
    const blob = await downloadStudentTemplate()
    saveBlob(blob, 'students-template.xlsx')
  } catch (error) {
    ElMessage.error(apiErrorMessage(error, '下载模板失败'))
  }
}

const handleExport = async () => {
  try {
    const blob = await exportStudents()
    saveBlob(blob, 'students.xlsx')
  } catch (error) {
    ElMessage.error(apiErrorMessage(error, '导出失败'))
  }
}

onMounted(() => { loadClasses(); loadStudents() })
</script>

<style scoped>
.students { padding: 20px; }
.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.header-actions { display: flex; gap: 12px; align-items: center; }
.page-title { font-size: 24px; font-weight: 500; color: #333; }
.pagination { margin-top: 20px; display: flex; justify-content: flex-end; }
</style>
