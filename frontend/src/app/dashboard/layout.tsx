'use client'

import React, { useEffect, useState } from 'react'
import { useRouter } from 'next/navigation'
import { Sidebar } from '@/components/layout/Sidebar'
import { Header } from '@/components/layout/Header'
import { UpgradeModal } from '@/components/modals/UpgradeModal'
import { useAuthStore } from '@/lib/stores/authStore'
import { Loader2 } from 'lucide-react'

/**
 * Custom hook to properly wait for Zustand persist hydration.
 * This ensures we don't read stale state before localStorage is loaded.
 */
function useHydration() {
  const [hydrated, setHydrated] = useState(false)

  useEffect(() => {
    // Check if already hydrated
    if (useAuthStore.persist?.hasHydrated()) {
      setHydrated(true)
      return
    }

    // Wait for hydration to complete
    const unsub = useAuthStore.persist?.onFinishHydration(() => {
      setHydrated(true)
    })

    return () => unsub?.()
  }, [])

  return hydrated
}

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const router = useRouter()
  const isHydrated = useHydration()
  
  // Only read auth state AFTER hydration is complete
  // This prevents reading stale null values before localStorage is loaded
  const accessToken = useAuthStore((state) => state.accessToken)
  const user = useAuthStore((state) => state.user)

  useEffect(() => {
    // Only check auth after hydration is complete
    if (isHydrated && (!accessToken || !user)) {
      router.replace('/login')
    }
  }, [isHydrated, accessToken, user, router])

  // Show loading while waiting for hydration
  if (!isHydrated) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-muted/40">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  // Don't render dashboard if not authenticated (after hydration)
  if (!accessToken || !user) {
    return null
  }

  return (
    <div className="min-h-screen bg-muted/40 font-sans">
      <Sidebar />
      <div className="flex flex-col md:pl-64">
        <Header />
        <main className="flex-1 py-16 px-6">
          <div className="mx-auto w-full max-w-6xl py-6">
            {children}
          </div>
        </main>
      </div>
      <UpgradeModal />
    </div>
  )
}
