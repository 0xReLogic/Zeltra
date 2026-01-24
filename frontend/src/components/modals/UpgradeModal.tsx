'use client'

import { Crown, AlertCircle } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { useUpgradeStore } from '@/lib/stores/upgradeStore'

export function UpgradeModal() {
  const { isOpen, closeModal, triggerReason, dismissible, blocking } = useUpgradeStore()

  // For blocking modals (subscription expired), prevent closing
  const handleOpenChange = (open: boolean) => {
    if (!open && dismissible) {
      closeModal()
    }
    // If not dismissible, do nothing (can't close)
  }

  return (
    <Dialog open={isOpen} onOpenChange={handleOpenChange}>
      <DialogContent
        className={`sm:max-w-md ${blocking ? 'border-red-200 dark:border-red-900' : ''}`}
        showCloseButton={dismissible}
        // Prevent closing on escape or backdrop click if not dismissible
        onEscapeKeyDown={(e) => !dismissible && e.preventDefault()}
        onPointerDownOutside={(e) => !dismissible && e.preventDefault()}
      >
        <DialogHeader>
          <div
            className={`mx-auto p-3 rounded-full w-fit mb-2 ${
              blocking
                ? 'bg-red-100 dark:bg-red-900/30'
                : 'bg-amber-100 dark:bg-amber-900/30'
            }`}
          >
            {blocking ? (
              <AlertCircle className="h-6 w-6 text-red-600 dark:text-red-500" />
            ) : (
              <Crown className="h-6 w-6 text-amber-600 dark:text-amber-500" />
            )}
          </div>
          <DialogTitle className="text-center text-xl">
            {blocking ? 'Subscription Required' : 'Upgrade to Pro'}
          </DialogTitle>
          <DialogDescription className="text-center">
            {triggerReason ||
              (blocking
                ? 'Your subscription has expired. Please upgrade to continue using Zeltra.'
                : "You've hit the limits of your current plan. Upgrade to unlock unlimited access and premium features.")}
          </DialogDescription>
        </DialogHeader>

        <DialogFooter className="flex flex-col gap-2 sm:flex-col">
          <Button
            className={`w-full text-white border-0 ${
              blocking
                ? 'bg-gradient-to-r from-green-500 to-emerald-500 hover:from-green-600 hover:to-emerald-600'
                : 'bg-gradient-to-r from-amber-500 to-orange-500 hover:from-amber-600 hover:to-orange-600'
            }`}
            size="lg"
          >
            {blocking ? 'Upgrade to Continue' : 'Upgrade Now'}
          </Button>
          {dismissible && (
            <Button variant="outline" className="w-full" onClick={closeModal}>
              Maybe Later
            </Button>
          )}
          {blocking && (
            <p className="text-xs text-center text-muted-foreground w-full">
              You must upgrade to continue using Zeltra
            </p>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
