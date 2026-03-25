import { apiClient, type ApiResponse, type PaginatedResponse } from './client'

export interface McpServer {
  id: string
  name: string
  description?: string
  server_type: string
  command?: string
  args?: string
  env?: string
  url?: string
  headers?: string
  enabled: boolean
  created_at: number
  updated_at: number
}

export interface CreateMcpServerRequest {
  name: string
  description?: string
  serverType: string
  command?: string
  args?: string[]
  env?: Record<string, string>
  url?: string
  headers?: Record<string, string>
  enabled?: boolean
}

export const mcpApi = {
  getAll: (page = 1, perPage = 20) =>
    apiClient.get<ApiResponse<PaginatedResponse<McpServer>>>(`/mcp?page=${page}&per_page=${perPage}`),

  getById: (id: string) =>
    apiClient.get<ApiResponse<McpServer>>(`/mcp/${id}`),

  create: (data: CreateMcpServerRequest) =>
    apiClient.post<ApiResponse<McpServer>>('/mcp', data),

  update: (id: string, data: Partial<CreateMcpServerRequest>) =>
    apiClient.put<ApiResponse<McpServer>>(`/mcp/${id}`, data),

  delete: (id: string) =>
    apiClient.delete<ApiResponse<void>>(`/mcp/${id}`),

  toggle: (id: string) =>
    apiClient.post<ApiResponse<void>>(`/mcp/${id}/toggle`),
}
