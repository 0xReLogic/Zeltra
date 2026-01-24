'use client'

import React, { useEffect, useState } from 'react'
import { useRouter } from 'next/navigation'
import { Sidebar } from '@/components/layout/Sidebar'
import { Header } from '@/components/layout/Header'
import { UpgradeModal } from '@/components/modals/UpgradeModal'
import { useAuthStore } from '@/lib/stores/authStore'
import { useProactiveRefresh } from '@/lib/hooks/useProactiveRefresh'
import { Loader2 } from 'lucide-react'

/**
 * Custom hook to properly wait for Zustand persist hydration.
 * This ensures we don't read stale state before localStorage is loaded.
 */
/**
 * Custom hook to properly wait for Zustand persist hydration.
 * This ensures we don't read stale state before localStorage is loaded.
 * 
 * IMPORTANT: We can't rely on hasHydrated() alone because it returns true
 * BEFORE the state is actually updated. We need to wait for the actual
 * state to be populated from localStorage.
 */
function useHydration() {
  const [hydrated, setHydrated] = useState(() => {
    // Skip on server
    if (typeof window === 'undefined') {
      console.log('🌊 useHydration: SSR mode, not hydrated')
      return false
    }

    // Check if persist middleware exists
    if (!useAuthStore.persist) {
      console.log('🌊 useHydration: No persist middleware, marking as hydrated')
      return true
    }

    // Check if already hydrated
    const isHydrated = useAuthStore.persist.hasHydrated()
    if (isHydrated) {
      console.log('🌊 useHydration: Already hydrated on mount, state:', {
        hasAccessToken: !!useAuthStore.getState().accessToken,
        hasUser: !!useAuthStore.getState().user,
      })
    }
    return isHydrated
  })
  
  const accessToken = useAuthStore((state) => state.accessToken)
  const user = useAuthStore((state) => state.user)

  useEffect(() => {
    // Skip if already hydrated or on server
    if (hydrated || typeof window === 'undefined') {
      return
    }

    // Check if persist middleware exists
    if (!useAuthStore.persist) {
      return
    }

    console.log('🌊 useHydration: Waiting for hydration...')
    
    // Wait for hydration to complete
    const unsub = useAuthStore.persist.onFinishHydration(() => {
      console.log('🌊 useHydration: Hydration finished, state:', {
        hasAccessToken: !!useAuthStore.getState().accessToken,
        hasUser: !!useAuthStore.getState().user,
      })
      setHydrated(true)
    })

    return () => {
      console.log('🌊 useHydration: Cleanup')
      unsub?.()
    }
  }, [hydrated])

  // Log when state changes after hydration
  useEffect(() => {
    if (hydrated) {
      console.log('🌊 useHydration: State after hydration:', {
        hasAccessToken: !!accessToken,
        hasUser: !!user,
      })
    }
  }, [hydrated, accessToken, user])

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
  
  // Use proactive refresh hook to automatically refresh tokens before expiry
  useProactiveRefresh()

  useEffect(() => {
    // Only check auth after hydration is complete
    if (!isHydrated) {
      return
    }

    // Add a small delay to ensure state is fully populated after hydration
    // This prevents race condition where hasHydrated() returns true but state is still null
    const timeoutId = setTimeout(() => {
      if (!accessToken || !user) {
        console.log('🚫 Auth check failed, redirecting to login:', { hasAccessToken: !!accessToken, hasUser: !!user })
        router.replace('/login')
      } else if (user.organizations.length === 0) {
        console.log('🏢 User has no organizations, redirecting to onboarding')
        router.replace('/onboarding/create-organization')
      } else {
        console.log('✅ Auth check passed:', { hasAccessToken: !!accessToken, hasUser: !!user })
      }
    }, 100) // 100ms delay to let state settle

    return () => clearTimeout(timeoutId)
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
