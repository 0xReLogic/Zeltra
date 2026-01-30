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
  currentEntityId: string | null  // NEW: Current selected entity
  isRefreshing: boolean
  setAuth: (user: User, accessToken: string, refreshToken: string, expiresIn: number) => void
  setTokens: (accessToken: string, refreshToken: string, expiresIn: number) => void
  setOrg: (orgId: string) => void
  setCurrentEntityId: (entityId: string) => void  // NEW: Set current entity
  addOrganization: (org: { id: string; name: string; slug: string; role: string }) => void
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
      currentEntityId: null,  // NEW: Initialize entity context
      isRefreshing: false,
      
      setAuth: (user, accessToken, refreshToken, expiresIn) => {
        // Store actual expiry time - proactive refresh is handled by dashboard layout
        const tokenExpiresAt = Date.now() + (expiresIn * 1000)
        const expiresInMinutes = Math.floor(expiresIn / 60)
        console.log(`🔐 setAuth called: user=${user.email}, expires in ${expiresInMinutes} minutes`)
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
        const expiresInMinutes = Math.floor(expiresIn / 60)
        console.log(`🔑 setTokens called: expires in ${expiresInMinutes} minutes`)
        set({ accessToken, refreshToken, tokenExpiresAt })
      },
      
      setOrg: (orgId) => set({ currentOrgId: orgId }),
      
      setCurrentEntityId: (entityId) => {
        console.log(`🏢 Setting current entity: ${entityId}`)
        set({ currentEntityId: entityId })
      },
      
      addOrganization: (org) => {
        const { user } = get()
        if (!user) {
          console.warn('⚠️ Cannot add organization: no user logged in')
          return
        }
        
        // Check if organization already exists
        const exists = user.organizations.some(o => o.id === org.id)
        if (exists) {
          console.log(`ℹ️ Organization ${org.name} already exists in user.organizations`)
          return
        }
        
        // Add new organization to user's organizations array
        const updatedUser = {
          ...user,
          organizations: [...user.organizations, org]
        }
        console.log(`✅ Added organization ${org.name} to user.organizations (total: ${updatedUser.organizations.length})`)
        set({ user: updatedUser })
      },
      
      logout: () => {
        console.log('🚪 LOGOUT CALLED - Clearing auth state')
        console.trace('Logout stack trace:')
        set({ 
          user: null, 
          accessToken: null, 
          refreshToken: null, 
          tokenExpiresAt: null,
          currentOrgId: null,
          currentEntityId: null  // NEW: Clear entity context on logout
        })
      },
      
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
        currentEntityId: state.currentEntityId,  // NEW: Persist entity context
      }),
      onRehydrateStorage: () => {
        console.log('💧 Zustand: Starting hydration from localStorage')
        return (state, error) => {
          if (error) {
            console.error('❌ Zustand: Hydration error:', error)
          } else {
            console.log('✅ Zustand: Hydration complete', {
              hasUser: !!state?.user,
              hasAccessToken: !!state?.accessToken,
              hasRefreshToken: !!state?.refreshToken,
            })
          }
        }
      },
    }
  )
)
