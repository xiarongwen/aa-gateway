import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Plus, Pencil, Trash2, Terminal, Download, CheckCircle, FileCode, Loader2, Copy, Undo2 } from 'lucide-react'
import toast from 'react-hot-toast'
import { cliToolsApi, type CliTool, type CliToolType, type CreateCliToolRequest, type CliToolConfigOutput } from '../api/cliTools'
import { providersApi, type Provider } from '../api/providers'
import { format } from 'date-fns'

// CLI 工具表单模态框
interface CliToolFormModalProps {
  isOpen: boolean
  onClose: () => void
  cliTool?: CliTool | null
  onSubmit: (data: CreateCliToolRequest) => void
  isSubmitting: boolean
}

function CliToolFormModal({ isOpen, onClose, cliTool, onSubmit, isSubmitting }: CliToolFormModalProps) {
  const [formData, setFormData] = useState<CreateCliToolRequest>({
    name: '',
    description: '',
    toolType: 'claude_code',
    providerId: '',
    apiKey: '',
    apiUrl: '',
    model: '',
    enabled: true,
  })

  // 获取 CLI 工具类型列表
  const { data: typesData } = useQuery<{
    success: boolean
    data?: CliToolType[]
    error?: string
  }>({
    queryKey: ['cliToolTypes'],
    queryFn: async () => {
      const result = await cliToolsApi.getTypes()
      return result as unknown as {
        success: boolean
        data?: CliToolType[]
        error?: string
      }
    },
  })

  // 获取 Provider 列表
  const { data: providersData } = useQuery<{
    success: boolean
    data?: {
      data: Provider[]
      total: number
      page: number
      per_page: number
      total_pages: number
    }
    error?: string
  }>({
    queryKey: ['providers', 1],
    queryFn: async () => {
      const result = await providersApi.getAll(1, 100)
      return result as unknown as {
        success: boolean
        data?: {
          data: Provider[]
          total: number
          page: number
          per_page: number
          total_pages: number
        }
        error?: string
      }
    },
  })

  const cliToolTypes = typesData?.data || []
  const providers = providersData?.data?.data || []

  useEffect(() => {
    if (isOpen) {
      setFormData({
        name: cliTool?.name || '',
        description: cliTool?.description || '',
        toolType: cliTool?.tool_type || 'claude_code',
        providerId: cliTool?.provider_id || '',
        apiKey: cliTool?.api_key || '',
        apiUrl: cliTool?.api_url || '',
        model: cliTool?.model || '',
        enabled: cliTool?.enabled ?? true,
      })
    }
  }, [cliTool, isOpen])

  const handleToolTypeChange = (toolType: string) => {
    // 根据工具类型设置默认的 API URL
    let defaultUrl = ''
    switch (toolType) {
      case 'claude_code':
        defaultUrl = ''
        break
      case 'codex':
        defaultUrl = 'https://api.openai.com/v1'
        break
      case 'gemini_cli':
        defaultUrl = 'https://generativelanguage.googleapis.com/v1beta'
        break
      default:
        defaultUrl = ''
    }
    setFormData({ ...formData, toolType, apiUrl: defaultUrl })
  }

  const handleProviderChange = (providerId: string) => {
    const provider = providers.find((p: Provider) => p.id === providerId)
    if (provider) {
      setFormData({
        ...formData,
        providerId,
        apiUrl: provider.base_url,
        apiKey: provider.api_key,
      })
    }
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    onSubmit(formData)
  }

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-2xl max-h-[90vh] overflow-y-auto">
        <div className="px-6 py-4 border-b">
          <h2 className="text-xl font-semibold text-gray-900">
            {cliTool ? 'Edit CLI Tool Config' : 'Create CLI Tool Config'}
          </h2>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          {/* Name */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Name *
            </label>
            <input
              type="text"
              required
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
              placeholder="My Claude Code Config"
            />
          </div>

          {/* Description */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Description
            </label>
            <input
              type="text"
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
              placeholder="Configuration for Claude Code with custom provider"
            />
          </div>

          {/* CLI Tool Type */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              CLI Tool Type *
            </label>
            <select
              value={formData.toolType}
              onChange={(e) => handleToolTypeChange(e.target.value)}
              disabled={!!cliTool}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:bg-gray-100"
            >
              {cliToolTypes.map((type: CliToolType) => (
                <option key={type.id} value={type.id}>
                  {type.name}
                </option>
              ))}
            </select>
            {cliTool && (
              <p className="text-xs text-gray-500 mt-1">Tool type cannot be changed after creation</p>
            )}
          </div>

          {/* Provider */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Provider *
            </label>
            <select
              value={formData.providerId}
              onChange={(e) => handleProviderChange(e.target.value)}
              required
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
            >
              <option value="">Select a provider</option>
              {providers.map((provider: Provider) => (
                <option key={provider.id} value={provider.id}>
                  {provider.name} ({provider.provider_type})
                </option>
              ))}
            </select>
          </div>

          {/* API URL */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              API URL *
            </label>
            <input
              type="text"
              required
              value={formData.apiUrl}
              onChange={(e) => setFormData({ ...formData, apiUrl: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
              placeholder="https://api.anthropic.com"
            />
          </div>

          {/* API Key */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              API Key *
            </label>
            <input
              type="password"
              required
              value={formData.apiKey}
              onChange={(e) => setFormData({ ...formData, apiKey: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
              placeholder="sk-..."
            />
          </div>

          {/* Model */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Model *
            </label>
            <input
              type="text"
              required
              value={formData.model}
              onChange={(e) => setFormData({ ...formData, model: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
              placeholder="claude-3-opus-20240229"
            />
          </div>

          {/* Enabled */}
          <div className="flex items-center">
            <input
              type="checkbox"
              id="enabled"
              checked={formData.enabled}
              onChange={(e) => setFormData({ ...formData, enabled: e.target.checked })}
              className="h-4 w-4 text-indigo-600 border-gray-300 rounded focus:ring-indigo-500"
            />
            <label htmlFor="enabled" className="ml-2 text-sm text-gray-700">
              Enable this configuration
            </label>
          </div>

          {/* Actions */}
          <div className="flex justify-end gap-3 pt-4 border-t">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-md"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSubmitting}
              className="px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 disabled:opacity-50 flex items-center gap-2"
            >
              {isSubmitting && <Loader2 className="w-4 h-4 animate-spin" />}
              {cliTool ? 'Update' : 'Create'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

// 配置文件预览模态框
interface ConfigPreviewModalProps {
  isOpen: boolean
  onClose: () => void
  config: CliToolConfigOutput | null
}

function ConfigPreviewModal({ isOpen, onClose, config }: ConfigPreviewModalProps) {
  const [copied, setCopied] = useState(false)

  if (!isOpen || !config) return null

  const handleCopy = () => {
    navigator.clipboard.writeText(config.config_content)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const handleDownload = () => {
    const blob = new Blob([config.config_content], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = config.config_path.split('/').pop() || 'config.json'
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-3xl max-h-[90vh] overflow-hidden">
        <div className="px-6 py-4 border-b flex justify-between items-center">
          <h2 className="text-xl font-semibold text-gray-900">
            Configuration File
          </h2>
          <div className="flex gap-2">
            <button
              onClick={handleCopy}
              className="flex items-center gap-1 px-3 py-1.5 text-sm bg-gray-100 hover:bg-gray-200 rounded-md"
            >
              {copied ? <CheckCircle className="w-4 h-4 text-green-600" /> : <Copy className="w-4 h-4" />}
              {copied ? 'Copied!' : 'Copy'}
            </button>
            <button
              onClick={handleDownload}
              className="flex items-center gap-1 px-3 py-1.5 text-sm bg-indigo-100 text-indigo-700 hover:bg-indigo-200 rounded-md"
            >
              <Download className="w-4 h-4" />
              Download
            </button>
          </div>
        </div>

        <div className="p-6 overflow-y-auto max-h-[60vh]">
          <div className="mb-4 p-3 bg-gray-50 rounded-md">
            <p className="text-sm text-gray-600">
              <span className="font-medium">Config Path:</span> {config.config_path}
            </p>
            {config.env_vars && config.env_vars.length > 0 && (
              <div className="mt-2">
                <p className="text-sm font-medium text-gray-600">Environment Variables:</p>
                <ul className="mt-1 text-sm text-gray-500">
                  {config.env_vars.map(([key, value]) => (
                    <li key={key} className="font-mono">
                      export {key}={value}
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>

          <pre className="bg-gray-900 text-gray-100 p-4 rounded-md overflow-x-auto text-sm font-mono">
            {config.config_content}
          </pre>
        </div>

        <div className="px-6 py-4 border-t flex justify-end">
          <button
            onClick={onClose}
            className="px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-md"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  )
}

// 删除确认模态框
interface DeleteModalProps {
  isOpen: boolean
  onClose: () => void
  onConfirm: () => void
  toolName: string
  isDeleting: boolean
}

function DeleteModal({ isOpen, onClose, onConfirm, toolName, isDeleting }: DeleteModalProps) {
  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-md p-6">
        <h2 className="text-xl font-semibold text-gray-900 mb-4">Delete Configuration</h2>
        <p className="text-gray-600 mb-6">
          Are you sure you want to delete "{toolName}"? This action cannot be undone.
        </p>
        <div className="flex justify-end gap-3">
          <button
            onClick={onClose}
            className="px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-md"
          >
            Cancel
          </button>
          <button
            onClick={onConfirm}
            disabled={isDeleting}
            className="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 disabled:opacity-50 flex items-center gap-2"
          >
            {isDeleting && <Loader2 className="w-4 h-4 animate-spin" />}
            Delete
          </button>
        </div>
      </div>
    </div>
  )
}

// 主页面
export default function CliTools() {
  const queryClient = useQueryClient()
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false)
  const [editingTool, setEditingTool] = useState<CliTool | null>(null)
  const [deletingTool, setDeletingTool] = useState<CliTool | null>(null)
  const [previewConfig, setPreviewConfig] = useState<CliToolConfigOutput | null>(null)
  const [page, setPage] = useState(1)
  const perPage = 10

  // 获取 CLI 工具配置列表
  const { data: response, isLoading } = useQuery<{
    success: boolean
    data?: {
      data: CliTool[]
      total: number
      page: number
      per_page: number
      total_pages: number
    }
    error?: string
  }>({
    queryKey: ['cliTools', page],
    queryFn: async () => {
      const result = await cliToolsApi.getAll(page, perPage)
      return result as unknown as {
        success: boolean
        data?: {
          data: CliTool[]
          total: number
          page: number
          per_page: number
          total_pages: number
        }
        error?: string
      }
    },
  })

  const tools = response?.data?.data ?? []
  const pagination = response?.data

  // 创建
  const createMutation = useMutation({
    mutationFn: cliToolsApi.create,
    onSuccess: () => {
      toast.success('CLI tool configuration created')
      setIsCreateModalOpen(false)
      queryClient.invalidateQueries({ queryKey: ['cliTools'] })
    },
    onError: (error: Error) => {
      toast.error(error.message || 'Failed to create configuration')
    },
  })

  // 更新
  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Partial<CreateCliToolRequest> }) =>
      cliToolsApi.update(id, data),
    onSuccess: () => {
      toast.success('CLI tool configuration updated')
      setEditingTool(null)
      queryClient.invalidateQueries({ queryKey: ['cliTools'] })
    },
    onError: (error: Error) => {
      toast.error(error.message || 'Failed to update configuration')
    },
  })

  // 删除
  const deleteMutation = useMutation({
    mutationFn: cliToolsApi.delete,
    onSuccess: () => {
      toast.success('CLI tool configuration deleted')
      setDeletingTool(null)
      queryClient.invalidateQueries({ queryKey: ['cliTools'] })
    },
    onError: (error: Error) => {
      toast.error(error.message || 'Failed to delete configuration')
    },
  })

  // 应用配置到本地
  const applyMutation = useMutation({
    mutationFn: cliToolsApi.apply,
    onSuccess: (data) => {
      const result = (data as unknown as { data?: { message: string; config_path: string; backup_path?: string } }).data
      toast.success(result?.message || 'Configuration applied successfully')
      queryClient.invalidateQueries({ queryKey: ['cliTools'] })
    },
    onError: (error: Error) => {
      toast.error(error.message || 'Failed to apply configuration')
    },
  })

  // 生成配置文件
  const generateConfigMutation = useMutation({
    mutationFn: cliToolsApi.generateConfig,
    onSuccess: (data) => {
      const configData = (data as unknown as { data?: CliToolConfigOutput }).data
      if (configData) {
        setPreviewConfig(configData)
      }
    },
    onError: (error: Error) => {
      toast.error(error.message || 'Failed to generate configuration')
    },
  })

  // 备份当前配置
  const backupMutation = useMutation({
    mutationFn: cliToolsApi.backup,
    onSuccess: (data) => {
      const result = (data as unknown as { data?: { message: string; config_path: string; backup_path: string } }).data
      toast.success(result?.message || 'Configuration backed up successfully')
    },
    onError: (error: Error) => {
      toast.error(error.message || 'Failed to backup configuration')
    },
  })

  // 恢复备份
  const restoreMutation = useMutation({
    mutationFn: cliToolsApi.restore,
    onSuccess: (data) => {
      const result = (data as unknown as { data?: { message: string; config_path: string } }).data
      toast.success(result?.message || 'Configuration restored successfully')
    },
    onError: (error: Error) => {
      toast.error(error.message || 'Failed to restore configuration')
    },
  })

  const handleCreate = (data: CreateCliToolRequest) => {
    createMutation.mutate(data)
  }

  const handleUpdate = (data: CreateCliToolRequest) => {
    if (editingTool) {
      updateMutation.mutate({ id: editingTool.id, data })
    }
  }

  const handleDelete = () => {
    if (deletingTool) {
      deleteMutation.mutate(deletingTool.id)
    }
  }

  const getToolTypeDisplayName = (toolType: string) => {
    const names: Record<string, string> = {
      claude_code: 'Claude Code',
      codex: 'Codex',
      gemini_cli: 'Gemini CLI',
      opencode: 'OpenCode',
      openclaw: 'OpenClaw',
    }
    return names[toolType] || toolType
  }

  return (
    <div>
      {/* Header */}
      <div className="flex justify-between items-center mb-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">CLI Tools</h1>
          <p className="mt-1 text-gray-600">
            Manage CLI tool configurations for Claude Code, Codex, Gemini CLI, and more
          </p>
        </div>
        <button
          onClick={() => setIsCreateModalOpen(true)}
          className="flex items-center gap-2 px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700"
        >
          <Plus className="w-4 h-4" />
          Add Configuration
        </button>
      </div>

      {/* Tools Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {isLoading ? (
          <div className="col-span-full flex justify-center py-12">
            <Loader2 className="w-8 h-8 animate-spin text-indigo-600" />
          </div>
        ) : tools.length === 0 ? (
          <div className="col-span-full text-center py-12 text-gray-500">
            <Terminal className="w-12 h-12 mx-auto mb-4 text-gray-300" />
            <p>No CLI tool configurations yet</p>
            <button
              onClick={() => setIsCreateModalOpen(true)}
              className="mt-2 text-indigo-600 hover:text-indigo-700"
            >
              Create your first configuration
            </button>
          </div>
        ) : (
          tools.map((tool: CliTool) => (
            <div
              key={tool.id}
              className={`bg-white rounded-lg shadow border-2 ${
                tool.enabled ? 'border-green-200' : 'border-gray-200'
              }`}
            >
              <div className="p-5">
                <div className="flex items-start justify-between mb-3">
                  <div className="flex items-center gap-2">
                    <Terminal className="w-5 h-5 text-indigo-600" />
                    <h3 className="font-semibold text-gray-900">{tool.name}</h3>
                  </div>
                  {tool.enabled && (
                    <span className="flex items-center gap-1 text-xs font-medium text-green-600 bg-green-100 px-2 py-1 rounded-full">
                      <CheckCircle className="w-3 h-3" />
                      Active
                    </span>
                  )}
                </div>

                <p className="text-sm text-gray-500 mb-3">{tool.description || 'No description'}</p>

                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-gray-500">Tool:</span>
                    <span className="font-medium">{getToolTypeDisplayName(tool.tool_type)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-500">Model:</span>
                    <span className="font-medium text-gray-700 truncate max-w-[150px]">{tool.model}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-500">API URL:</span>
                    <span className="font-medium text-gray-700 truncate max-w-[150px]">{tool.api_url}</span>
                  </div>
                </div>

                <div className="mt-4 pt-4 border-t flex items-center justify-between">
                  <span className="text-xs text-gray-400">
                    Updated {format(new Date(tool.updated_at), 'MMM d, yyyy')}
                  </span>
                  <div className="flex items-center gap-1">
                    {/* 备份按钮 - 备份当前配置 */}
                    <button
                      onClick={() => backupMutation.mutate(tool.id)}
                      disabled={backupMutation.isPending}
                      className="p-1.5 text-gray-400 hover:text-blue-600"
                      title="Backup current config"
                    >
                      {backupMutation.isPending ? <Loader2 className="w-4 h-4 animate-spin" /> : <Download className="w-4 h-4" />}
                    </button>
                    {/* 恢复备份按钮 */}
                    <button
                      onClick={() => restoreMutation.mutate(tool.id)}
                      disabled={restoreMutation.isPending}
                      className="p-1.5 text-gray-400 hover:text-orange-600"
                      title="Restore from backup"
                    >
                      {restoreMutation.isPending ? <Loader2 className="w-4 h-4 animate-spin" /> : <Undo2 className="w-4 h-4" />}
                    </button>
                    {!tool.enabled && (
                      <button
                        onClick={() => applyMutation.mutate(tool.id)}
                        disabled={applyMutation.isPending}
                        className="p-1.5 text-gray-400 hover:text-green-600"
                        title="Apply to local"
                      >
                        {applyMutation.isPending ? <Loader2 className="w-4 h-4 animate-spin" /> : <CheckCircle className="w-4 h-4" />}
                      </button>
                    )}
                    <button
                      onClick={() => generateConfigMutation.mutate(tool.id)}
                      disabled={generateConfigMutation.isPending}
                      className="p-1.5 text-gray-400 hover:text-indigo-600"
                      title="View Config"
                    >
                      <FileCode className="w-4 h-4" />
                    </button>
                    <button
                      onClick={() => setEditingTool(tool)}
                      className="p-1.5 text-gray-400 hover:text-indigo-600"
                      title="Edit"
                    >
                      <Pencil className="w-4 h-4" />
                    </button>
                    <button
                      onClick={() => setDeletingTool(tool)}
                      className="p-1.5 text-gray-400 hover:text-red-600"
                      title="Delete"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Pagination */}
      {pagination && pagination.total_pages > 1 && (
        <div className="mt-6 flex items-center justify-between">
          <div className="text-sm text-gray-500">
            Showing {(pagination.page - 1) * pagination.per_page + 1} to{' '}
            {Math.min(pagination.page * pagination.per_page, pagination.total)} of{' '}
            {pagination.total} results
          </div>
          <div className="flex gap-2">
            <button
              onClick={() => setPage(p => Math.max(1, p - 1))}
              disabled={page === 1}
              className="px-3 py-1 text-sm border rounded-md disabled:opacity-50 hover:bg-gray-50"
            >
              Previous
            </button>
            <button
              onClick={() => setPage(p => Math.min(pagination.total_pages, p + 1))}
              disabled={page === pagination.total_pages}
              className="px-3 py-1 text-sm border rounded-md disabled:opacity-50 hover:bg-gray-50"
            >
              Next
            </button>
          </div>
        </div>
      )}

      {/* Modals */}
      <CliToolFormModal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        onSubmit={handleCreate}
        isSubmitting={createMutation.isPending}
      />

      <CliToolFormModal
        isOpen={!!editingTool}
        onClose={() => setEditingTool(null)}
        cliTool={editingTool}
        onSubmit={handleUpdate}
        isSubmitting={updateMutation.isPending}
      />

      <DeleteModal
        isOpen={!!deletingTool}
        onClose={() => setDeletingTool(null)}
        onConfirm={handleDelete}
        toolName={deletingTool?.name || ''}
        isDeleting={deleteMutation.isPending}
      />

      <ConfigPreviewModal
        isOpen={!!previewConfig}
        onClose={() => setPreviewConfig(null)}
        config={previewConfig}
      />
    </div>
  )
}
