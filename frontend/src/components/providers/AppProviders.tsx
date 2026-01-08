'use client'

import { QueryClientProvider } from '@tanstack/react-query'
import { queryClient } from '@/lib/api/client'
import { MSWProvider } from '@/components/MSWProvider'

export function AppProviders({ children }: { children: React.ReactNode }) {
  return (
    <QueryClientProvider client={queryClient}>
      <MSWProvider>
        {children}
      </MSWProvider>
    </QueryClientProvider>
  )
}
