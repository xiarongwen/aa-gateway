import { apiClient, type ApiResponse } from './client'

export interface ProxyConfig {
  enabled: boolean
  listenAddress: string
  listenPort: number
  enableAuth: boolean
  enableLogging: boolean
  config?: ProxyAdvancedConfig
}

export interface ProxyAdvancedConfig {
  requestTimeoutSecs?: number
  connectTimeoutSecs?: number
  maxConnections?: number
  enableLoadBalance?: boolean
  enableFailover?: boolean
  failoverThreshold?: number
  circuitBreaker?: CircuitBreakerConfig
}

export interface CircuitBreakerConfig {
  failureRateThreshold: number
  slowCallRateThreshold: number
  slowCallDurationThresholdMs: number
  permittedCallsInHalfOpenState: number
  waitDurationInOpenStateMs: number
  slidingWindowSize: number
}

export interface ProxyStatus {
  running: boolean
  address: string
  port: number
  uptimeSeconds: number
  totalRequests: number
  activeConnections: number
}

export const proxyApi = {
  getConfig: () =>
    apiClient.get<ApiResponse<ProxyConfig>>('/proxy/config'),

  updateConfig: (data: Partial<ProxyConfig>) =>
    apiClient.put<ApiResponse<ProxyConfig>>('/proxy/config', data),

  getStatus: () =>
    apiClient.get<ApiResponse<ProxyStatus>>('/proxy/status'),

  start: () =>
    apiClient.post<ApiResponse<void>>('/proxy/start'),

  stop: () =>
    apiClient.post<ApiResponse<void>>('/proxy/stop'),
}
