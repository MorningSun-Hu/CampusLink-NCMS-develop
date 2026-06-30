<template>
  <div class="students">
    <div class="header">
      <h2 class="page-title">学生管理</h2>
      <div class="header-actions">
        <el-input v-model="keyword" placeholder="搜索学号/姓名" clearable style="width: 240px" @change="loadStudents" />
        <el-upload :show-file-list="false" :before-upload="handleImport" accept=".xlsx" style="display: inline-flex">
          <el-button type="warning">批量导入</el-button>
        </el-upload>
        <el-button type="success" @click="handleExport">导出 Excel</el-button>
        <el-button type="primary" @click="showCreateDialog">新增学生</el-button>
      </div>
    </div>

    <el-table :data="students" v-loading="loading" border stripe>
      <el-table-column prop="studentNo" label="学号" width="140" />
      <el-table-column prop="name" label="姓名" width="120" />
      <el-table-column prop="seatNo" label="座位号" width="100">
        <template #default="{ row }">{{ row.seatNo || '-' }}</template>
      </el-table-column>
      <el-table-column prop="status" label="状态" width="100">
        <template #default="{ row }">
          <el-tag :type="row.status === 'active' ? 'success' : 'info'">
            {{ row.status === 'active' ? '正常' : row.status }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="createdAt" label="创建时间" width="180">
        <template #default="{ row }">{{ row.createdAt || '-' }}</template>
      </el-table-column>
      <el-table-column label="操作" width="180">
        <template #default="{ row }">
          <el-button type="primary" size="small" @click="showEditDialog(row)">编辑</el-button>
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
          <el-input v-model="form.studentNo" placeholder="请输入学号" :disabled="isEdit" />
        </el-form-item>
        <el-form-item label="姓名" prop="name">
          <el-input v-model="form.name" placeholder="请输入姓名" />
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
import { listStudents, createStudent, updateStudent, deleteStudent, importStudents, getExportUrl, type Student } from '@/api/students'
import { ElMessage, ElMessageBox } from 'element-plus'

const students = ref<Student[]>([])
const loading = ref(false)
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const keyword = ref('')

const dialogVisible = ref(false)
const isEdit = ref(false)
const submitting = ref(false)
const editingId = ref('')
const formRef = ref()

const form = reactive({
  studentNo: '',
  name: '',
  password: '',
  seatNo: '',
})

const rules = {
  studentNo: [{ required: true, message: '请输入学号', trigger: 'blur' }],
  name: [{ required: true, message: '请输入姓名', trigger: 'blur' }],
  password: [{ required: true, message: '请输入密码', trigger: 'blur' }],
}

const loadStudents = async () => {
  loading.value = true
  try {
    const result = await listStudents({ page: page.value, pageSize: pageSize.value, keyword: keyword.value })
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
  form.password = ''
  form.seatNo = ''
  dialogVisible.value = true
}

const showEditDialog = (row: Student) => {
  isEdit.value = true
  editingId.value = row.id
  form.studentNo = row.studentNo
  form.name = row.name
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
      const data: any = { name: form.name, seatNo: form.seatNo || undefined }
      if (form.password) data.password = form.password
      await updateStudent(editingId.value, data)
      ElMessage.success('更新成功')
    } else {
      await createStudent({
        studentNo: form.studentNo,
        name: form.name,
        password: form.password,
        seatNo: form.seatNo || undefined,
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

onMounted(() => { loadStudents() })
</script>

<style scoped>
.students { padding: 20px; }
.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.header-actions { display: flex; gap: 12px; align-items: center; }
.page-title { font-size: 24px; font-weight: 500; color: #333; }
.pagination { margin-top: 20px; display: flex; justify-content: flex-end; }
</style>
