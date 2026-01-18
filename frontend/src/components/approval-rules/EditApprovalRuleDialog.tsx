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
import { Edit } from 'lucide-react'
import { ApprovalRuleForm } from './ApprovalRuleForm'
import { useUpdateApprovalRule } from '@/lib/queries/approval-rules'
import type { ApprovalRuleFormValues } from '@/lib/validations/approval-rule'
import type { components } from '@/types/api.generated'

type ApprovalRuleResponse = components['schemas']['ApprovalRuleResponse']

interface EditApprovalRuleDialogProps {
  rule: ApprovalRuleResponse
  children?: React.ReactNode
}

export function EditApprovalRuleDialog({ rule, children }: EditApprovalRuleDialogProps) {
  const [open, setOpen] = useState(false)
  const updateMutation = useUpdateApprovalRule()

  const handleSubmit = async (values: ApprovalRuleFormValues) => {
    try {
      await updateMutation.mutateAsync({
        id: rule.id,
        data: {
          name: values.name,
          description: values.description || null,
          transaction_types: values.transaction_types,
          required_role: values.required_role,
          priority: values.priority,
          min_amount: values.min_amount || null,
          max_amount: values.max_amount || null,
          is_active: values.is_active,
        },
      })
      
      toast.success('Approval rule updated successfully')
      setOpen(false)
    } catch (error) {
      toast.error('Failed to update approval rule')
      console.error('Update approval rule error:', error)
    }
  }

  const defaultValues: Partial<ApprovalRuleFormValues> = {
    name: rule.name,
    description: rule.description,
    transaction_types: rule.transaction_types as any,
    required_role: rule.required_role as any,
    priority: rule.priority,
    min_amount: rule.min_amount,
    max_amount: rule.max_amount,
    is_active: rule.is_active,
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        {children || (
          <Button variant="ghost" size="sm">
            <Edit className="h-4 w-4" />
          </Button>
        )}
      </DialogTrigger>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Edit Approval Rule</DialogTitle>
          <DialogDescription>
            Update the approval rule settings.
          </DialogDescription>
        </DialogHeader>
        <ApprovalRuleForm
          mode="edit"
          defaultValues={defaultValues}
          onSubmit={handleSubmit}
          isSubmitting={updateMutation.isPending}
        />
      </DialogContent>
    </Dialog>
  )
}