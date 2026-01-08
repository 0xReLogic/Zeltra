'use client'

import { useEffect, useState } from 'react'

export function MSWProvider({ children }: { children: React.ReactNode }) {
  const [mockingEnabled, setMockingEnabled] = useState(false)

  useEffect(() => {
    async function enableMocking() {
      if (process.env.NEXT_PUBLIC_API_MOCK === 'true') {
        if (typeof window !== 'undefined') {
          const { worker } = await import('@/mocks/browser')
          await worker.start()
        }
      }
      setMockingEnabled(true)
    }
    enableMocking()
  }, [])

  if (!mockingEnabled && process.env.NEXT_PUBLIC_API_MOCK === 'true') {
    return null
  }

  return <>{children}</>
}
