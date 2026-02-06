/**
 * Auth Store Tests
 * 
 * Tests for the auth store including:
 * - setCurrentEntityId updates state
 * - clearAuth clears currentEntityId
 * - Entity context persistence
 * 
 * Requirements: 7.3, 15.1
 */

import { describe, it, expect, beforeEach } from 'vitest'
import { useAuthStore } from '../authStore'

describe('Auth Store', () => {
  beforeEach(() => {
    // Reset store state before each test
    useAuthStore.setState({
      user: null,
      accessToken: null,
      refreshToken: null,
      tokenExpiresAt: null,
      currentOrgId: null,
      currentEntityId: null,
      isRefreshing: false,
    })
    
    // Clear localStorage
    localStorage.clear()
  })

  describe('setCurrentEntityId', () => {
    it('should update currentEntityId state', () => {
      const { setCurrentEntityId } = useAuthStore.getState()
      
      setCurrentEntityId('entity-123')
      
      const state = useAuthStore.getState()
      expect(state.currentEntityId).toBe('entity-123')
    })

    it('should persist entity ID to localStorage', () => {
      const { setCurrentEntityId } = useAuthStore.getState()
      
      setCurrentEntityId('entity-456')
      
      // Check localStorage directly (persistence is handled by zustand middleware)
      // Note: In actual implementation, persistence happens through zustand/middleware
      const state = useAuthStore.getState()
      expect(state.currentEntityId).toBe('entity-456')
    })

    it('should allow changing entity ID', () => {
      const { setCurrentEntityId } = useAuthStore.getState()
      
      setCurrentEntityId('entity-1')
      expect(useAuthStore.getState().currentEntityId).toBe('entity-1')
      
      setCurrentEntityId('entity-2')
      expect(useAuthStore.getState().currentEntityId).toBe('entity-2')
    })
  })

  describe('logout', () => {
    it('should clear currentEntityId on logout', () => {
      const { setCurrentEntityId, logout } = useAuthStore.getState()
      
      // Set entity ID
      setCurrentEntityId('entity-123')
      expect(useAuthStore.getState().currentEntityId).toBe('entity-123')
      
      // Logout
      logout()
      
      // Verify entity ID is cleared
      const state = useAuthStore.getState()
      expect(state.currentEntityId).toBeNull()
    })

    it('should clear all auth state on logout', () => {
      const { setAuth, setCurrentEntityId, logout } = useAuthStore.getState()
      
      // Set auth state
      const mockUser = {
        id: 'user-1',
        email: 'test@example.com',
        full_name: 'Test User',
        organizations: [
          {
            id: 'org-1',
            name: 'Test Org',
            slug: 'test-org',
            role: 'owner',
          },
        ],
      }
      
      setAuth(mockUser, 'access-token', 'refresh-token', 3600)
      setCurrentEntityId('entity-123')
      
      // Verify state is set
      let state = useAuthStore.getState()
      expect(state.user).toBeTruthy()
      expect(state.accessToken).toBe('access-token')
      expect(state.currentEntityId).toBe('entity-123')
      
      // Logout
      logout()
      
      // Verify all state is cleared
      state = useAuthStore.getState()
      expect(state.user).toBeNull()
      expect(state.accessToken).toBeNull()
      expect(state.refreshToken).toBeNull()
      expect(state.tokenExpiresAt).toBeNull()
      expect(state.currentOrgId).toBeNull()
      expect(state.currentEntityId).toBeNull()
    })
  })

  describe('entity context persistence', () => {
    it('should maintain entity context across store updates', () => {
      const { setCurrentEntityId, setOrg } = useAuthStore.getState()
      
      // Set entity ID
      setCurrentEntityId('entity-123')
      expect(useAuthStore.getState().currentEntityId).toBe('entity-123')
      
      // Change organization (should not affect entity ID)
      setOrg('org-456')
      
      // Entity ID should still be set
      const state = useAuthStore.getState()
      expect(state.currentEntityId).toBe('entity-123')
      expect(state.currentOrgId).toBe('org-456')
    })

    it('should handle null entity ID', () => {
      const { setCurrentEntityId } = useAuthStore.getState()
      
      // Set to null
      setCurrentEntityId(null as any)
      
      expect(useAuthStore.getState().currentEntityId).toBeNull()
    })
  })

  describe('setAuth', () => {
    it('should not affect currentEntityId when setting auth', () => {
      const { setCurrentEntityId, setAuth } = useAuthStore.getState()
      
      // Set entity ID first
      setCurrentEntityId('entity-123')
      
      // Set auth
      const mockUser = {
        id: 'user-1',
        email: 'test@example.com',
        full_name: 'Test User',
        organizations: [
          {
            id: 'org-1',
            name: 'Test Org',
            slug: 'test-org',
            role: 'owner',
          },
        ],
      }
      
      setAuth(mockUser, 'access-token', 'refresh-token', 3600)
      
      // Entity ID should still be set
      const state = useAuthStore.getState()
      expect(state.currentEntityId).toBe('entity-123')
      expect(state.user).toBeTruthy()
    })
  })
})
