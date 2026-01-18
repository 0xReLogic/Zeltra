'use client'

import { useState } from 'react'
import { toast } from 'sonner'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { Trash2 } from 'lucide-react'
import { useDeleteApprovalRule } from '@/lib/queries/approval-rules'
import type { components } from '@/types/api.generated'

type ApprovalRuleResponse = components['schemas']['ApprovalRuleResponse']

interface DeleteApprovalRuleDialogProps {
  rule: ApprovalRuleResponse
  children?: React.ReactNode
}

export function DeleteApprovalRuleDialog({ rule, children }: DeleteApprovalRuleDialogProps) {
  const [open, setOpen] = useState(false)
  const deleteMutation = useDeleteApprovalRule()

  const handleDelete = async () => {
    try {
      await deleteMutation.mutateAsync(rule.id)
      toast.success('Approval rule deleted successfully')
      setOpen(false)
    } catch (error) {
      toast.error('Failed to delete approval rule')
      console.error('Delete approval rule error:', error)
    }
  }

  return (
    <AlertDialog open={open} onOpenChange={setOpen}>
      <AlertDialogTrigger asChild>
        {children || (
          <Button variant="ghost" size="sm">
            <Trash2 className="h-4 w-4" />
          </Button>
        )}
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete Approval Rule</AlertDialogTitle>
          <AlertDialogDescription>
            Are you sure you want to delete the approval rule "{rule.name}"? 
            This action cannot be undone and may affect existing approval workflows.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            onClick={handleDelete}
            disabled={deleteMutation.isPending}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
          >
            {deleteMutation.isPending ? 'Deleting...' : 'Delete Rule'}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}