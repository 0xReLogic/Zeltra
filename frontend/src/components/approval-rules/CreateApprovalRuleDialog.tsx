'use client'

import { useState } from 'react'
import { toast } from 'sonner'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Plus } from 'lucide-react'
import { ApprovalRuleForm } from './ApprovalRuleForm'
import { useCreateApprovalRule } from '@/lib/queries/approval-rules'
import type { ApprovalRuleFormValues } from '@/lib/validations/approval-rule'

interface CreateApprovalRuleDialogProps {
  children?: React.ReactNode
}

export function CreateApprovalRuleDialog({ children }: CreateApprovalRuleDialogProps) {
  const [open, setOpen] = useState(false)
  const createMutation = useCreateApprovalRule()

  const handleSubmit = async (values: ApprovalRuleFormValues) => {
    try {
      await createMutation.mutateAsync({
        name: values.name,
        description: values.description || null,
        transaction_types: values.transaction_types,
        required_role: values.required_role,
        priority: values.priority,
        min_amount: values.min_amount || null,
        max_amount: values.max_amount || null,
      })
      
      toast.success('Approval rule created successfully')
      setOpen(false)
    } catch (error) {
      toast.error('Failed to create approval rule')
      console.error('Create approval rule error:', error)
    }
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        {children || (
          <Button>
            <Plus className="h-4 w-4 mr-2" />
            Create Rule
          </Button>
        )}
      </DialogTrigger>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Create Approval Rule</DialogTitle>
          <DialogDescription>
            Create a new approval rule to define when transactions require approval.
          </DialogDescription>
        </DialogHeader>
        <ApprovalRuleForm
          mode="create"
          onSubmit={handleSubmit}
          isSubmitting={createMutation.isPending}
        />
      </DialogContent>
    </Dialog>
  )
}