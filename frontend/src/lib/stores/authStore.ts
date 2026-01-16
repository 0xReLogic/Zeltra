import { create } from 'zustand'
import { persist } from 'zustand/middleware'

interface User {
  id: string
  email: string
  full_name: string
  organizations: Array<{
    id: string
    name: string
    slug: string
    role: string
  }>
}

interface AuthState {
  user: User | null
  accessToken: string | null
  refreshToken: string | null
  tokenExpiresAt: number | null
  currentOrgId: string | null
  isRefreshing: boolean
  setAuth: (user: User, accessToken: string, refreshToken: string, expiresIn: number) => void
  setTokens: (accessToken: string, refreshToken: string, expiresIn: number) => void
  setOrg: (orgId: string) => void
  logout: () => void
  isTokenExpired: () => boolean
  setRefreshing: (isRefreshing: boolean) => void
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      accessToken: null,
      refreshToken: null,
      tokenExpiresAt: null,
      currentOrgId: null,
      isRefreshing: false,
      
      setAuth: (user, accessToken, refreshToken, expiresIn) => {
        // Store actual expiry time - proactive refresh is handled by dashboard layout
        const tokenExpiresAt = Date.now() + (expiresIn * 1000)
        set({ 
          user, 
          accessToken, 
          refreshToken, 
          tokenExpiresAt,
          currentOrgId: user.organizations[0]?.id || null 
        })
      },
      
      setTokens: (accessToken, refreshToken, expiresIn) => {
        const tokenExpiresAt = Date.now() + (expiresIn * 1000)
        set({ accessToken, refreshToken, tokenExpiresAt })
      },
      
      setOrg: (orgId) => set({ currentOrgId: orgId }),
      
      logout: () => set({ 
        user: null, 
        accessToken: null, 
        refreshToken: null, 
        tokenExpiresAt: null,
        currentOrgId: null 
      }),
      
      isTokenExpired: () => {
        const { tokenExpiresAt } = get()
        if (!tokenExpiresAt) return true
        return Date.now() >= tokenExpiresAt
      },
      
      setRefreshing: (isRefreshing) => set({ isRefreshing }),
    }),
    { 
      name: 'auth-storage',
      partialize: (state) => ({
        user: state.user,
        accessToken: state.accessToken,
        refreshToken: state.refreshToken,
        tokenExpiresAt: state.tokenExpiresAt,
        currentOrgId: state.currentOrgId,
      })
    }
  )
)
