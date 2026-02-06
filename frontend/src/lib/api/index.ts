/**
 * API client wrapper
 * Provides a clean interface for making API requests
 */

import { apiClient } from './client'

export const api = {
  get: <T>(endpoint: string) => apiClient<T>(endpoint, { method: 'GET' }),
  
  post: <T>(endpoint: string, data?: unknown) =>
    apiClient<T>(endpoint, {
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined,
    }),
  
  patch: <T>(endpoint: string, data?: unknown) =>
    apiClient<T>(endpoint, {
      method: 'PATCH',
      body: data ? JSON.stringify(data) : undefined,
    }),
  
  delete: <T>(endpoint: string) =>
    apiClient<T>(endpoint, { method: 'DELETE' }),
}

export { apiClient } from './client'
