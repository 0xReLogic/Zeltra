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
  currentOrgId: string | null
  setAuth: (user: User, accessToken: string, refreshToken: string) => void
  setOrg: (orgId: string) => void
  logout: () => void
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      accessToken: null,
      refreshToken: null,
      currentOrgId: null,
      setAuth: (user, accessToken, refreshToken) => 
        set({ user, accessToken, refreshToken, currentOrgId: user.organizations[0]?.id || null }),
      setOrg: (orgId) => set({ currentOrgId: orgId }),
      logout: () => set({ user: null, accessToken: null, refreshToken: null, currentOrgId: null }),
    }),
    { name: 'auth-storage' }
  )
)
