import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Plus, Pencil, Trash2, Star, StarOff, Server, Loader2 } from 'lucide-react'
import toast from 'react-hot-toast'
import { providersApi, type Provider, type CreateProviderRequest } from '../api/providers'
import { format } from 'date-fns'

// Provider Form Modal Component
interface ProviderFormModalProps {
  isOpen: boolean
  onClose: () => void
  provider?: Provider | null
  onSubmit: (data: CreateProviderRequest) => void
  isSubmitting: boolean
}

const defaultProviderTypes = [
  { id: 'openai', name: 'OpenAI', baseUrl: 'https://api.openai.com/v1' },
  { id: 'anthropic', name: 'Anthropic', baseUrl: 'https://api.anthropic.com' },
  { id: 'gemini', name: 'Google Gemini', baseUrl: 'https://generativelanguage.googleapis.com/v1beta' },
  { id: 'azure', name: 'Azure OpenAI', baseUrl: '' },
  { id: 'ollama', name: 'Ollama', baseUrl: 'http://localhost:11434' },
  { id: 'custom', name: 'Custom', baseUrl: '' },
]

function ProviderFormModal({ isOpen, onClose, provider, onSubmit, isSubmitting }: ProviderFormModalProps) {
  const [formData, setFormData] = useState<CreateProviderRequest>({
    name: '',
    providerType: 'openai',
    baseUrl: 'https://api.openai.com/v1',
    apiKey: '',
    models: [],
    category: 'third_party',
  })
  const [modelInput, setModelInput] = useState('')

  // 当 provider 变化时重置表单数据
  useEffect(() => {
    if (isOpen) {
      setFormData({
        name: provider?.name || '',
        providerType: provider?.provider_type || 'openai',
        baseUrl: provider?.base_url || 'https://api.openai.com/v1',
        apiKey: provider?.api_key || '',
        models: provider?.models ? JSON.parse(provider.models) : [],
        category: provider?.category || 'third_party',
      })
      setModelInput('')
    }
  }, [provider, isOpen])

  if (!isOpen) return null

  const handleProviderTypeChange = (typeId: string) => {
    const typeInfo = defaultProviderTypes.find(t => t.id === typeId)
    setFormData({
      ...formData,
      providerType: typeId,
      baseUrl: typeInfo?.baseUrl || '',
    })
  }

  const addModel = () => {
    if (modelInput.trim()) {
      setFormData({
        ...formData,
        models: [...(formData.models || []), { id: modelInput.trim(), name: modelInput.trim() }],
      })
      setModelInput('')
    }
  }

  const removeModel = (index: number) => {
    setFormData({
      ...formData,
      models: formData.models?.filter((_, i) => i !== index) || [],
    })
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    onSubmit(formData)
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-2xl max-h-[90vh] overflow-y-auto">
        <div className="px-6 py-4 border-b">
          <h2 className="text-xl font-semibold text-gray-900">
            {provider ? 'Edit Provider' : 'Create Provider'}
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
              placeholder="My OpenAI Provider"
            />
          </div>

          {/* Provider Type */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Provider Type *
            </label>
            <select
              value={formData.providerType}
              onChange={(e) => handleProviderTypeChange(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
            >
              {defaultProviderTypes.map((type) => (
                <option key={type.id} value={type.id}>
                  {type.name}
                </option>
              ))}
            </select>
          </div>

          {/* Base URL */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Base URL *
            </label>
            <input
              type="text"
              required
              value={formData.baseUrl}
              onChange={(e) => setFormData({ ...formData, baseUrl: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
              placeholder="https://api.openai.com/v1"
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

          {/* Category */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Category
            </label>
            <select
              value={formData.category}
              onChange={(e) => setFormData({ ...formData, category: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
            >
              <option value="official">Official</option>
              <option value="cloud_provider">Cloud Provider</option>
              <option value="aggregator">Aggregator</option>
              <option value="third_party">Third Party</option>
              <option value="custom">Custom</option>
            </select>
          </div>

          {/* Models */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Models
            </label>
            <div className="flex gap-2 mb-2">
              <input
                type="text"
                value={modelInput}
                onChange={(e) => setModelInput(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && (e.preventDefault(), addModel())}
                className="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
                placeholder="Add model ID (e.g., gpt-4)"
              />
              <button
                type="button"
                onClick={addModel}
                className="px-4 py-2 bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200"
              >
                Add
              </button>
            </div>
            <div className="flex flex-wrap gap-2">
              {formData.models?.map((model, index) => (
                <span
                  key={index}
                  className="inline-flex items-center px-2 py-1 bg-indigo-50 text-indigo-700 rounded-md text-sm"
                >
                  {model.id}
                  <button
                    type="button"
                    onClick={() => removeModel(index)}
                    className="ml-1 text-indigo-500 hover:text-indigo-700"
                  >
                    ×
                  </button>
                </span>
              ))}
            </div>
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
              {provider ? 'Update' : 'Create'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

// Delete Confirmation Modal
interface DeleteModalProps {
  isOpen: boolean
  onClose: () => void
  onConfirm: () => void
  providerName: string
  isDeleting: boolean
}

function DeleteModal({ isOpen, onClose, onConfirm, providerName, isDeleting }: DeleteModalProps) {
  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-md p-6">
        <h2 className="text-xl font-semibold text-gray-900 mb-4">Delete Provider</h2>
        <p className="text-gray-600 mb-6">
          Are you sure you want to delete "{providerName}"? This action cannot be undone.
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

// Main Providers Page
export default function Providers() {
  const queryClient = useQueryClient()
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false)
  const [editingProvider, setEditingProvider] = useState<Provider | null>(null)
  const [deletingProvider, setDeletingProvider] = useState<Provider | null>(null)
  const [page, setPage] = useState(1)
  const perPage = 10

  // Fetch providers
  const { data: response, isLoading } = useQuery<{
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
    queryKey: ['providers', page],
    queryFn: async () => {
      const result = await providersApi.getAll(page, perPage)
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

  const providers = response?.data?.data ?? []
  const pagination = response?.data

  // Create mutation
  const createMutation = useMutation({
    mutationFn: providersApi.create,
    onSuccess: () => {
      toast.success('Provider created successfully')
      setIsCreateModalOpen(false)
      queryClient.invalidateQueries({ queryKey: ['providers'] })
    },
    onError: (error: Error) => {
      toast.error(error.message || 'Failed to create provider')
    },
  })

  // Update mutation
  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Partial<CreateProviderRequest> }) =>
      providersApi.update(id, data),
    onSuccess: () => {
      toast.success('Provider updated successfully')
      setEditingProvider(null)
      queryClient.invalidateQueries({ queryKey: ['providers'] })
    },
    onError: (error: Error) => {
      toast.error(error.message || 'Failed to update provider')
    },
  })

  // Delete mutation
  const deleteMutation = useMutation({
    mutationFn: providersApi.delete,
    onSuccess: () => {
      toast.success('Provider deleted successfully')
      setDeletingProvider(null)
      queryClient.invalidateQueries({ queryKey: ['providers'] })
    },
    onError: (error: Error) => {
      toast.error(error.message || 'Failed to delete provider')
    },
  })

  // Set default mutation
  const setDefaultMutation = useMutation({
    mutationFn: providersApi.setDefault,
    onSuccess: () => {
      toast.success('Default provider set successfully')
      queryClient.invalidateQueries({ queryKey: ['providers'] })
    },
    onError: (error: Error) => {
      toast.error(error.message || 'Failed to set default provider')
    },
  })

  const handleCreate = (data: CreateProviderRequest) => {
    createMutation.mutate(data)
  }

  const handleUpdate = (data: CreateProviderRequest) => {
    if (editingProvider) {
      updateMutation.mutate({ id: editingProvider.id, data })
    }
  }

  const handleDelete = () => {
    if (deletingProvider) {
      deleteMutation.mutate(deletingProvider.id)
    }
  }

  return (
    <div>
      {/* Header */}
      <div className="flex justify-between items-center mb-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Providers</h1>
          <p className="mt-1 text-gray-600">Manage your AI providers</p>
        </div>
        <button
          onClick={() => setIsCreateModalOpen(true)}
          className="flex items-center gap-2 px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700"
        >
          <Plus className="w-4 h-4" />
          Add Provider
        </button>
      </div>

      {/* Providers Table */}
      <div className="bg-white rounded-lg shadow overflow-hidden">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Provider
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Type
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Base URL
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Models
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Status
              </th>
              <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                Actions
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {isLoading ? (
              <tr>
                <td colSpan={6} className="px-6 py-12 text-center">
                  <Loader2 className="w-8 h-8 animate-spin mx-auto text-indigo-600" />
                </td>
              </tr>
            ) : providers.length === 0 ? (
              <tr>
                <td colSpan={6} className="px-6 py-12 text-center text-gray-500">
                  <Server className="w-12 h-12 mx-auto mb-4 text-gray-300" />
                  <p>No providers configured yet</p>
                  <button
                    onClick={() => setIsCreateModalOpen(true)}
                    className="mt-2 text-indigo-600 hover:text-indigo-700"
                  >
                    Add your first provider
                  </button>
                </td>
              </tr>
            ) : (
              providers.map((provider: Provider) => (
                <tr key={provider.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex items-center">
                      {provider.is_default && (
                        <Star className="w-4 h-4 text-yellow-400 mr-2 fill-current" />
                      )}
                      <div>
                        <div className="text-sm font-medium text-gray-900">
                          {provider.name}
                        </div>
                        <div className="text-sm text-gray-500">
                          {format(new Date(provider.created_at), 'MMM d, yyyy')}
                        </div>
                      </div>
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className="px-2 py-1 text-xs font-medium bg-gray-100 text-gray-700 rounded-full capitalize">
                      {provider.provider_type}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {provider.base_url}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {provider.models ? JSON.parse(provider.models).length : 0} models
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    {provider.is_default ? (
                      <span className="px-2 py-1 text-xs font-medium bg-green-100 text-green-700 rounded-full">
                        Default
                      </span>
                    ) : (
                      <span className="px-2 py-1 text-xs font-medium bg-gray-100 text-gray-700 rounded-full">
                        Active
                      </span>
                    )}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                    <div className="flex items-center justify-end gap-2">
                      {!provider.is_default && (
                        <button
                          onClick={() => setDefaultMutation.mutate(provider.id)}
                          disabled={setDefaultMutation.isPending}
                          className="p-1 text-gray-400 hover:text-yellow-500"
                          title="Set as default"
                        >
                          <StarOff className="w-4 h-4" />
                        </button>
                      )}
                      <button
                        onClick={() => setEditingProvider(provider)}
                        className="p-1 text-gray-400 hover:text-indigo-600"
                        title="Edit"
                      >
                        <Pencil className="w-4 h-4" />
                      </button>
                      <button
                        onClick={() => setDeletingProvider(provider)}
                        className="p-1 text-gray-400 hover:text-red-600"
                        title="Delete"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>

        {/* Pagination */}
        {pagination && pagination.total_pages > 1 && (
          <div className="px-6 py-4 border-t flex items-center justify-between">
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
      </div>

      {/* Modals */}
      <ProviderFormModal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        onSubmit={handleCreate}
        isSubmitting={createMutation.isPending}
      />

      <ProviderFormModal
        isOpen={!!editingProvider}
        onClose={() => setEditingProvider(null)}
        provider={editingProvider}
        onSubmit={handleUpdate}
        isSubmitting={updateMutation.isPending}
      />

      <DeleteModal
        isOpen={!!deletingProvider}
        onClose={() => setDeletingProvider(null)}
        onConfirm={handleDelete}
        providerName={deletingProvider?.name || ''}
        isDeleting={deleteMutation.isPending}
      />
    </div>
  )
}
