import { create } from 'zustand'

interface UpgradeModalOptions {
  dismissible?: boolean
  blocking?: boolean
}

interface UpgradeStore {
  isOpen: boolean
  triggerReason: string | null
  dismissible: boolean
  blocking: boolean
  openModal: (reason?: string, options?: UpgradeModalOptions) => void
  closeModal: () => void
}

export const useUpgradeStore = create<UpgradeStore>((set) => ({
  isOpen: false,
  triggerReason: null,
  dismissible: true,
  blocking: false,
  openModal: (reason, options) =>
    set({
      isOpen: true,
      triggerReason: reason || null,
      dismissible: options?.dismissible ?? true,
      blocking: options?.blocking ?? false,
    }),
  closeModal: () =>
    set({
      isOpen: false,
      triggerReason: null,
      dismissible: true,
      blocking: false,
    }),
}))
