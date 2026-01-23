import { ReactNode } from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

/**
 * Creates a wrapper component for React Query testing
 * 
 * @param queryClient - The QueryClient instance to use for testing
 * @returns A wrapper component that provides the QueryClient context
 */
export function createWrapper(queryClient: QueryClient) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        {children}
      </QueryClientProvider>
    )
  }
}
