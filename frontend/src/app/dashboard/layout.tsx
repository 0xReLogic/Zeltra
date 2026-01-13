'use client'

import React, { useEffect, useState } from 'react'
import { useRouter } from 'next/navigation'
import { Sidebar } from '@/components/layout/Sidebar'
import { Header } from '@/components/layout/Header'
import { useAuthStore } from '@/lib/stores/authStore'
import { Loader2 } from 'lucide-react'

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const router = useRouter()
  const accessToken = useAuthStore((state) => state.accessToken)
  const user = useAuthStore((state) => state.user)
  const [isChecking, setIsChecking] = useState(true)

  useEffect(() => {
    // Wait for hydration to complete before checking auth
    const checkAuth = () => {
      const state = useAuthStore.getState()
      if (!state.accessToken || !state.user) {
        router.replace('/login')
      } else {
        setIsChecking(false)
      }
    }

    // Small delay to ensure Zustand has hydrated from localStorage
    const timer = setTimeout(checkAuth, 100)
    return () => clearTimeout(timer)
  }, [router])

  // Also check on state changes (e.g., after logout)
  useEffect(() => {
    if (!isChecking && (!accessToken || !user)) {
      router.replace('/login')
    }
  }, [accessToken, user, isChecking, router])

  // Show loading while checking auth
  if (isChecking) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-muted/40">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  // Don't render dashboard if not authenticated
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
    </div>
  )
}
