export interface McpServer {
  id: string
  name: string
  description?: string
  server_type: 'stdio' | 'http' | 'sse'
  command?: string
  args?: string[]
  env?: Record<string, string>
  url?: string
  headers?: Record<string, string>
  enabled: boolean
  created_at: number
  updated_at: number
}
