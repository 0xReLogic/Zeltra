'use client'

import { Crown, Sparkles, Check } from 'lucide-react'
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
  const { isOpen, closeModal, triggerReason } = useUpgradeStore()

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && closeModal()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <div className="mx-auto bg-amber-100 p-3 rounded-full w-fit mb-2 dark:bg-amber-900/30">
            <Crown className="h-6 w-6 text-amber-600 dark:text-amber-500" />
          </div>
          <DialogTitle className="text-center text-xl">Upgrade to Pro</DialogTitle>
          <DialogDescription className="text-center">
            {triggerReason || "You've hit the limits of the Free plan. Upgrade to unlock unlimited access and premium features."}
          </DialogDescription>
        </DialogHeader>
        
        <div className="space-y-4 py-4">
          <div className="grid gap-2">
            <div className="flex items-center gap-2">
              <div className="h-5 w-5 rounded-full bg-green-100 flex items-center justify-center shrink-0 dark:bg-green-900/30">
                <Check className="h-3 w-3 text-green-600 dark:text-green-500" />
              </div>
              <span className="text-sm">Unlimited Transactions</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="h-5 w-5 rounded-full bg-green-100 flex items-center justify-center shrink-0 dark:bg-green-900/30">
                <Check className="h-3 w-3 text-green-600 dark:text-green-500" />
              </div>
              <span className="text-sm">Unlimited Users</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="h-5 w-5 rounded-full bg-green-100 flex items-center justify-center shrink-0 dark:bg-green-900/30">
                <Check className="h-3 w-3 text-green-600 dark:text-green-500" />
              </div>
              <span className="text-sm">Advanced AI Analysis</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="h-5 w-5 rounded-full bg-green-100 flex items-center justify-center shrink-0 dark:bg-green-900/30">
                <Check className="h-3 w-3 text-green-600 dark:text-green-500" />
              </div>
              <span className="text-sm">Priority Support</span>
            </div>
          </div>
        </div>

        <DialogFooter className="flex-col !space-x-0 !space-y-2 sm:!space-y-2">
          <Button className="w-full bg-gradient-to-r from-amber-500 to-orange-500 hover:from-amber-600 hover:to-orange-600 text-white border-0" size="lg">
            <Sparkles className="mr-2 h-4 w-4" />
            Upgrade Now
          </Button>
          <Button variant="outline" className="w-full" onClick={closeModal}>
            Maybe Later
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
