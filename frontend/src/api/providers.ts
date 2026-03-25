import { apiClient, type ApiResponse, type PaginatedResponse } from './client'

export interface Provider {
  id: string
  name: string
  provider_type: string
  base_url: string
  api_key: string
  models: string // JSON string
  config?: string
  category?: string
  is_default: boolean
  created_at: number
  updated_at: number
}

export interface CreateProviderRequest {
  name: string
  providerType: string
  baseUrl: string
  apiKey: string
  models: ModelConfig[]
  config?: Record<string, unknown>
  category?: string
}

export interface ModelConfig {
  id: string
  name: string
  context_window?: number
  max_tokens?: number
  input_cost_per_1k?: number
  output_cost_per_1k?: number
  capabilities?: string[]
}

export const providersApi = {
  getAll: (page = 1, perPage = 20) =>
    apiClient.get<ApiResponse<PaginatedResponse<Provider>>>(`/providers?page=${page}&per_page=${perPage}`),

  getById: (id: string) =>
    apiClient.get<ApiResponse<Provider>>(`/providers/${id}`),

  create: (data: CreateProviderRequest) =>
    apiClient.post<ApiResponse<Provider>>('/providers', data),

  update: (id: string, data: Partial<CreateProviderRequest>) =>
    apiClient.put<ApiResponse<Provider>>(`/providers/${id}`, data),

  delete: (id: string) =>
    apiClient.delete<ApiResponse<void>>(`/providers/${id}`),

  setDefault: (id: string) =>
    apiClient.post<ApiResponse<void>>(`/providers/${id}/default`),

  getTypes: () =>
    apiClient.get<ApiResponse<Array<{ id: string; name: string; description: string }>>>('/providers/types'),
}
