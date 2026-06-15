import axios from 'axios'
import type { ApiResponse, PhotoRecord } from './types'

const api = axios.create({
  baseURL: '/api',
  timeout: 30000,
})

export async function uploadPhoto(file: File, inspectionId?: string) {
  const formData = new FormData()
  formData.append('photo', file)
  if (inspectionId) {
    formData.append('inspection_id', inspectionId)
  }
  const response = await api.post<ApiResponse<PhotoRecord>>('/photos/upload', formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
  })
  return response.data
}

export async function getPhotoList(params?: { inspection_id?: string }) {
  const response = await api.get<ApiResponse<PhotoRecord[]>>('/photos', { params })
  return response.data
}

export function getPhotoUrl(photoId: string) {
  return `/api/photos/${photoId}`
}
