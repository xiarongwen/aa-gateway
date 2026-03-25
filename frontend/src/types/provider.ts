export interface Provider {
  id: string
  name: string
  provider_type: string
  base_url: string
  api_key: string
  models: string
  config?: string
  category?: string
  is_default: boolean
  created_at: number
  updated_at: number
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

export type ProviderType = 'openai' | 'anthropic' | 'gemini' | 'azure' | 'ollama' | 'custom'

export type ProviderCategory = 'official' | 'cloud_provider' | 'aggregator' | 'third_party' | 'custom'
