<template>
  <div class="students">
    <div class="header">
      <h2 class="page-title">学生管理</h2>
      <div class="header-actions">
        <el-input v-model="keyword" placeholder="搜索学号/姓名" clearable style="width: 220px" @change="loadStudents" />
        <el-select v-model="selectedClassId" placeholder="按班级筛选" clearable style="width: 180px" @change="loadStudents">
          <el-option v-for="c in classes" :key="c.id" :label="c.name" :value="c.id" />
        </el-select>
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
        <el-form-item label="班级">
          <el-select v-model="form.classId" placeholder="选填" clearable style="width: 100%">
            <el-option v-for="c in classes" :key="c.id" :label="c.name" :value="c.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="密码" :prop="isEdit ? '' : 'password'">
          <el-input v-model="form.password" placeholder="请输入密码（留空不修改）" show-password />
        </el-form-item>
        <el-form-item label="座位号">
          <el-input v-model="form.seatNo" placeholder="选填" />
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
import { ref, reactive, onMounted } from 'vue'
import { listStudents, createStudent, updateStudent, deleteStudent, resetStudentPassword, importStudents, getExportUrl, type Student } from '@/api/students'
import { listClasses, type ClassInfo } from '@/api/classes'
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

const rules = {
  name: [{ required: true, message: '请输入姓名', trigger: 'blur' }],
}

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
      const data: any = { name: form.name, seatNo: form.seatNo || undefined, classId: form.classId || undefined }
      if (form.password) data.password = form.password
      await updateStudent(editingId.value, data)
      ElMessage.success('更新成功')
    } else {
      await createStudent({
        studentNo: form.studentNo || undefined,
        name: form.name,
        password: form.password || undefined,
        seatNo: form.seatNo || undefined,
        classId: form.classId || undefined,
      })
      ElMessage.success('创建成功')
    }
    dialogVisible.value = false
    loadStudents()
  } catch (error) {
    console.error('Submit failed:', error)
    ElMessage.error('操作失败')
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
    if (result.data?.success) {
      ElMessage.success(`成功导入 ${result.data.imported} 名学生`)
      loadStudents()
    } else {
      ElMessage.error('导入失败')
    }
  } catch (error) {
    ElMessage.error('导入失败')
  }
  return false
}

const handleExport = () => {
  window.open(getExportUrl(), '_blank')
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
