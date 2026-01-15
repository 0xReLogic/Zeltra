import { create } from 'zustand'

interface UpgradeStore {
  isOpen: boolean
  triggerReason: string | null
  openModal: (reason?: string) => void
  closeModal: () => void
}

export const useUpgradeStore = create<UpgradeStore>((set) => ({
  isOpen: false,
  triggerReason: null,
  openModal: (reason) => set({ isOpen: true, triggerReason: reason || null }),
  closeModal: () => set({ isOpen: false, triggerReason: null }),
}))
