'use client'

import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { 
  approvalRuleSchema, 
  type ApprovalRuleFormValues,
  TRANSACTION_TYPES,
  ROLES 
} from '@/lib/validations/approval-rule'

interface ApprovalRuleFormProps {
  mode?: 'create' | 'edit'
  defaultValues?: Partial<ApprovalRuleFormValues>
  onSubmit: (values: ApprovalRuleFormValues) => void
  isSubmitting?: boolean
}

export function ApprovalRuleForm({ 
  mode = 'create', 
  defaultValues, 
  onSubmit, 
  isSubmitting 
}: ApprovalRuleFormProps) {
  const form = useForm<ApprovalRuleFormValues>({
    resolver: zodResolver(approvalRuleSchema),
    defaultValues: {
      name: defaultValues?.name ?? '',
      description: defaultValues?.description ?? '',
      transaction_types: defaultValues?.transaction_types ?? [],
      required_role: defaultValues?.required_role ?? 'approver',
      priority: defaultValues?.priority ?? 1,
      min_amount: defaultValues?.min_amount ?? '',
      max_amount: defaultValues?.max_amount ?? '',
      is_active: defaultValues?.is_active ?? true,
    },
  })

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
        <FormField
          control={form.control}
          name="name"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Rule Name</FormLabel>
              <FormControl>
                <Input 
                  placeholder="e.g. High Value Bills" 
                  {...field} 
                  aria-describedby="name-description"
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="description"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Description (Optional)</FormLabel>
              <FormControl>
                <Textarea 
                  placeholder="Describe when this rule applies..."
                  className="resize-none"
                  {...field}
                  value={field.value ?? ''}
                  aria-describedby="description-description"
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="transaction_types"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Transaction Types</FormLabel>
              <div className="grid grid-cols-2 gap-2 mt-2">
                {TRANSACTION_TYPES.map((type) => (
                  <FormItem
                    key={type}
                    className="flex flex-row items-start space-x-3 space-y-0"
                  >
                    <FormControl>
                      <Checkbox
                        checked={field.value?.includes(type)}
                        onCheckedChange={(checked) => {
                          const currentTypes = field.value || []
                          if (checked) {
                            field.onChange([...currentTypes, type])
                          } else {
                            field.onChange(currentTypes.filter((t) => t !== type))
                          }
                        }}
                        aria-describedby={`${type}-description`}
                      />
                    </FormControl>
                    <FormLabel className="text-sm font-normal capitalize">
                      {type.replace('_', ' ')}
                    </FormLabel>
                  </FormItem>
                ))}
              </div>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="required_role"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Required Approver Role</FormLabel>
              <Select onValueChange={field.onChange} defaultValue={field.value}>
                <FormControl>
                  <SelectTrigger aria-describedby="role-description">
                    <SelectValue placeholder="Select required role" />
                  </SelectTrigger>
                </FormControl>
                <SelectContent>
                  {ROLES.map((role) => (
                    <SelectItem key={role} value={role}>
                      <span className="capitalize">{role}</span>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="priority"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Priority (1-100)</FormLabel>
              <FormControl>
                <Input 
                  type="number"
                  min={1}
                  max={100}
                  placeholder="1"
                  {...field}
                  onChange={(e) => field.onChange(parseInt(e.target.value) || 1)}
                  aria-describedby="priority-description"
                />
              </FormControl>
              <p className="text-xs text-muted-foreground">
                Lower numbers = higher priority
              </p>
              <FormMessage />
            </FormItem>
          )}
        />

        <div className="grid grid-cols-2 gap-4">
          <FormField
            control={form.control}
            name="min_amount"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Minimum Amount (Optional)</FormLabel>
                <FormControl>
                  <Input 
                    placeholder="0.00"
                    {...field}
                    value={field.value ?? ''}
                    aria-describedby="min-amount-description"
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="max_amount"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Maximum Amount (Optional)</FormLabel>
                <FormControl>
                  <Input 
                    placeholder="999999.99"
                    {...field}
                    value={field.value ?? ''}
                    aria-describedby="max-amount-description"
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
        </div>

        <FormField
          control={form.control}
          name="is_active"
          render={({ field }) => (
            <FormItem className="flex flex-row items-start space-x-3 space-y-0">
              <FormControl>
                <Checkbox
                  checked={field.value}
                  onCheckedChange={field.onChange}
                  aria-describedby="active-description"
                />
              </FormControl>
              <div className="space-y-1 leading-none">
                <FormLabel>Active Rule</FormLabel>
                <p className="text-xs text-muted-foreground">
                  Only active rules will be applied to transactions
                </p>
              </div>
            </FormItem>
          )}
        />

        <div className="flex justify-end pt-4">
          <Button type="submit" disabled={isSubmitting}>
            {isSubmitting ? 'Saving...' : mode === 'edit' ? 'Update Rule' : 'Create Rule'}
          </Button>
        </div>
      </form>
    </Form>
  )
}