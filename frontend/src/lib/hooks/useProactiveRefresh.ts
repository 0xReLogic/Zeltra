import { useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { useAuthStore } from '../stores/authStore'
import { useRefresh } from '../queries/auth'

/**
 * Hook to proactively refresh access token before it expires.
 * Checks every minute and refreshes 5 minutes before expiry.
 */
export function useProactiveRefresh() {
  const router = useRouter()
  const accessToken = useAuthStore((state) => state.accessToken)
  const tokenExpiresAt = useAuthStore((state) => state.tokenExpiresAt)
  const refreshToken = useAuthStore((state) => state.refreshToken)
  const logout = useAuthStore((state) => state.logout)
  const { mutateAsync: refresh } = useRefresh()

  useEffect(() => {
    if (!accessToken || !tokenExpiresAt || !refreshToken) {
      console.log('⏸️ Proactive refresh skipped:', { 
        hasAccessToken: !!accessToken,
        hasExpiresAt: !!tokenExpiresAt,
        hasRefreshToken: !!refreshToken 
      })
      return
    }

    console.log('🔧 Proactive refresh effect mounted')

    const checkAndRefresh = async () => {
      const now = Date.now()
      const timeUntilExpiry = tokenExpiresAt - now
      const fiveMinutes = 5 * 60 * 1000
      const timeUntilExpiryMinutes = Math.floor(timeUntilExpiry / 60000)

      console.log(`⏰ Token check: expires in ${timeUntilExpiryMinutes} minutes`)

      // Refresh 5 minutes before expiry
      if (timeUntilExpiry > 0 && timeUntilExpiry < fiveMinutes) {
        console.log('🔄 Token expiring soon, refreshing proactively...')
        try {
          await refresh()
          console.log('✅ Token refreshed proactively')
        } catch (error) {
          console.error('❌ Proactive refresh failed:', error)
          console.log('🚪 LOGOUT TRIGGERED: Proactive refresh failed')
          logout()
          router.replace('/login')
        }
      } else if (timeUntilExpiry <= 0) {
        console.log('⚠️ Token already expired!')
        logout()
        router.replace('/login')
      }
    }

    // Check immediately
    checkAndRefresh()

    // Then check every minute
    const checkInterval = setInterval(checkAndRefresh, 60000)

    return () => {
      console.log('🔧 Proactive refresh effect unmounted')
      clearInterval(checkInterval)
    }
  }, [accessToken, tokenExpiresAt, refreshToken, refresh, logout, router])
}
