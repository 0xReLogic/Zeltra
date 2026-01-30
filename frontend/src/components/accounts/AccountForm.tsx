'use client'

import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import * as z from 'zod'
import { AlertTriangle } from 'lucide-react'
import { Button } from '@/components/ui/button'
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
import { useAuthStore } from '@/lib/stores/authStore'
import { CreateAccountRequest } from '@/types/accounts'



const formSchema = z.object({
  code: z.string().min(1, 'Code is required'),
  name: z.string().min(2, 'Name must be at least 2 characters'),
  type: z.string().min(1, 'Type is required'),
  currency: z.string().min(1, 'Currency is required'),
})

type FormValues = z.infer<typeof formSchema>

interface AccountFormProps {
  mode?: 'create' | 'edit'
  defaultValues?: Partial<CreateAccountRequest>
  onSubmit: (values: CreateAccountRequest) => void
  isSubmitting?: boolean
  isLoading?: boolean
}

export function AccountForm({ mode = 'create', defaultValues, onSubmit, isSubmitting, isLoading }: AccountFormProps) {
  const currentEntityId = useAuthStore((state) => state.currentEntityId)
  
  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      code: defaultValues?.code ?? '',
      name: defaultValues?.name ?? '',
      type: defaultValues?.type ?? 'asset',
      currency: defaultValues?.currency ?? 'USD',
    },
  })

  const loading = isSubmitting || isLoading

  const handleSubmit = (values: FormValues) => {
    if (!currentEntityId) {
      return // Button should be disabled, but extra safety
    }
    // Add entity_id to the request
    onSubmit({
      ...values,
      entity_id: currentEntityId,
    } as CreateAccountRequest)
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-4">
        {/* Entity Selection Warning */}
        {!currentEntityId && (
          <div className="flex items-start gap-2 rounded-md border border-amber-500/50 bg-amber-500/10 p-3 text-sm">
            <AlertTriangle className="h-4 w-4 text-amber-500 mt-0.5 flex-shrink-0" />
            <div>
              <p className="font-medium text-amber-600">No Entity Selected</p>
              <p className="text-muted-foreground text-xs mt-1">
                Please select an entity from the sidebar before creating an account.
              </p>
            </div>
          </div>
        )}
        <FormField
          control={form.control}
          name="code"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Account Code</FormLabel>
              <FormControl>
                <Input placeholder="e.g. 1100" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="name"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Account Name</FormLabel>
              <FormControl>
                <Input placeholder="e.g. Cash in Bank" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="type"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Type</FormLabel>
              <Select onValueChange={field.onChange} defaultValue={field.value}>
                <FormControl>
                  <SelectTrigger>
                    <SelectValue placeholder="Select type" />
                  </SelectTrigger>
                </FormControl>
                <SelectContent>
                  <SelectItem value="asset">Asset</SelectItem>
                  <SelectItem value="liability">Liability</SelectItem>
                  <SelectItem value="equity">Equity</SelectItem>
                  <SelectItem value="revenue">Revenue</SelectItem>
                  <SelectItem value="expense">Expense</SelectItem>
                </SelectContent>
              </Select>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="currency"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Currency</FormLabel>
              <Select onValueChange={field.onChange} defaultValue={field.value}>
                <FormControl>
                  <SelectTrigger>
                    <SelectValue placeholder="Select currency" />
                  </SelectTrigger>
                </FormControl>
                <SelectContent>
                  <SelectItem value="USD">USD ($)</SelectItem>
                  <SelectItem value="EUR">EUR (€)</SelectItem>
                  <SelectItem value="IDR">IDR (Rp)</SelectItem>
                  <SelectItem value="SGD">SGD (S$)</SelectItem>
                </SelectContent>
              </Select>
              <FormMessage />
            </FormItem>
          )}
        />

        <div className="flex justify-end pt-4">
          <Button type="submit" disabled={loading || !currentEntityId}>
            {loading ? 'Saving...' : !currentEntityId ? 'Select Entity First' : mode === 'edit' ? 'Update Account' : 'Save Account'}
          </Button>
        </div>
      </form>
    </Form>
  )
}
