import { apiClient, type ApiResponse, type PaginatedResponse } from './client'

export interface CliTool {
  id: string
  name: string
  description?: string
  tool_type: string
  provider_id: string
  api_key: string
  api_url: string
  model: string
  enabled: boolean
  created_at: number
  updated_at: number
}

export interface CliToolType {
  id: string
  name: string
  description: string
  config_path: string
}

export interface CliToolConfigOutput {
  tool_type: string
  config_path: string
  config_content: string
  env_vars?: Array<[string, string]>
}

export interface CreateCliToolRequest {
  name: string
  description?: string
  toolType: string
  providerId: string
  apiKey: string
  apiUrl: string
  model: string
  enabled?: boolean
}

export const cliToolsApi = {
  // 获取 CLI 工具配置列表
  getAll: (page = 1, perPage = 20) =>
    apiClient.get<ApiResponse<PaginatedResponse<CliTool>>>(`/cli-tools?page=${page}&per_page=${perPage}`),

  // 获取单个 CLI 工具配置
  getById: (id: string) =>
    apiClient.get<ApiResponse<CliTool>>(`/cli-tools/${id}`),

  // 创建 CLI 工具配置
  create: (data: CreateCliToolRequest) =>
    apiClient.post<ApiResponse<CliTool>>('/cli-tools', data),

  // 更新 CLI 工具配置
  update: (id: string, data: Partial<CreateCliToolRequest>) =>
    apiClient.put<ApiResponse<CliTool>>(`/cli-tools/${id}`, data),

  // 删除 CLI 工具配置
  delete: (id: string) =>
    apiClient.delete<ApiResponse<void>>(`/cli-tools/${id}`),

  // 切换启用状态
  toggle: (id: string) =>
    apiClient.post<ApiResponse<void>>(`/cli-tools/${id}/toggle`),

  // 获取支持的 CLI 工具类型
  getTypes: () =>
    apiClient.get<ApiResponse<CliToolType[]>>('/cli-tools/types'),

  // 生成配置文件
  generateConfig: (id: string) =>
    apiClient.get<ApiResponse<CliToolConfigOutput>>(`/cli-tools/${id}/config`),

  // 应用配置到本地（写入文件）
  apply: (id: string) =>
    apiClient.post<ApiResponse<{
      message: string
      config_path: string
      backup_path: string
      env_vars?: Array<[string, string]>
    }>>(`/cli-tools/${id}/apply`),

  // 备份当前配置
  backup: (id: string) =>
    apiClient.post<ApiResponse<{
      message: string
      config_path: string
      backup_path: string
    }>>(`/cli-tools/${id}/backup`),

  // 恢复备份配置
  restore: (id: string) =>
    apiClient.post<ApiResponse<{
      message: string
      config_path: string
      backup_path: string
    }>>(`/cli-tools/${id}/restore`),
}
