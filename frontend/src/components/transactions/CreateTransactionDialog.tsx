'use client'

import { useState } from 'react'
import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import * as z from 'zod'
import { format } from 'date-fns'
import { CalendarIcon, Loader2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
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
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover'
import { Calendar } from '@/components/ui/calendar'
import { cn } from '@/lib/utils'
import { useCreateTransaction } from '@/lib/queries/transactions'
import { useAccounts } from '@/lib/queries/accounts'
import { useDimensions } from '@/lib/queries/dimensions'
import { toast } from 'sonner'
import type { CreateTransactionRequest, CreateEntryRequest } from '@/types/transactions'

const formSchema = z.object({
  type: z.enum(['expense', 'revenue', 'transfer', 'journal']),
  transaction_date: z.date(),
  reference_number: z.string().optional(),
  description: z.string().min(1, 'Description is required'),
  memo: z.string().optional(),
  amount: z.string().refine((val) => !isNaN(parseFloat(val)) && parseFloat(val) > 0, {
    message: 'Amount must be a positive number',
  }),
  main_account_id: z.string().min(1, 'Account is required'),
  contra_account_id: z.string().min(1, 'Category/Contra account is required'),
  department: z.string().optional(),
  project: z.string().optional(),
  currency: z.string(),
})

type FormValues = z.infer<typeof formSchema>

export function CreateTransactionDialog() {
  const [open, setOpen] = useState(false)
  const createMutation = useCreateTransaction()
  const { data: accountsData } = useAccounts()
  const { data: dimensionsData } = useDimensions()

  // Ensure data is arrays - handle wrapper objects
  const accounts = accountsData?.accounts ?? []
  const dimensions = Array.isArray(dimensionsData) ? dimensionsData : []

  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      type: 'expense',
      transaction_date: new Date(),
      reference_number: '',
      description: '',
      memo: '',
      amount: '',
      main_account_id: '',
      contra_account_id: '',
      department: '',
      project: '',
      currency: 'USD',
    },
  })

  function onSubmit(values: FormValues) {
    const amount = values.amount
    const currency = values.currency

    // Collect dimension IDs
    const dims: string[] = []
    if (values.department && values.department !== 'none') dims.push(values.department)
    if (values.project && values.project !== 'none') dims.push(values.project)

    // Build entries based on transaction type
    // API expects: account_id, entry_type ("debit" | "credit"), source_amount, source_currency
    let entries: CreateEntryRequest[] = []

    if (values.type === 'expense') {
      // Expense: Debit Expense account, Credit Asset account
      entries = [
        {
          account_id: values.contra_account_id, // Expense account
          entry_type: 'debit',
          source_amount: amount,
          source_currency: currency,
          dimensions: dims.length > 0 ? dims : undefined,
        },
        {
          account_id: values.main_account_id, // Asset/Bank account
          entry_type: 'credit',
          source_amount: amount,
          source_currency: currency,
        },
      ]
    } else if (values.type === 'revenue') {
      // Revenue: Debit Asset account, Credit Revenue account
      entries = [
        {
          account_id: values.main_account_id, // Asset/Bank account
          entry_type: 'debit',
          source_amount: amount,
          source_currency: currency,
        },
        {
          account_id: values.contra_account_id, // Revenue account
          entry_type: 'credit',
          source_amount: amount,
          source_currency: currency,
          dimensions: dims.length > 0 ? dims : undefined,
        },
      ]
    } else {
      // Transfer/Journal: Simple debit/credit
      entries = [
        {
          account_id: values.main_account_id,
          entry_type: 'debit',
          source_amount: amount,
          source_currency: currency,
          dimensions: dims.length > 0 ? dims : undefined,
        },
        {
          account_id: values.contra_account_id,
          entry_type: 'credit',
          source_amount: amount,
          source_currency: currency,
        },
      ]
    }

    const request: CreateTransactionRequest = {
      type: values.type,
      transaction_date: format(values.transaction_date, 'yyyy-MM-dd'),
      description: values.description,
      entries,
      reference_number: values.reference_number || undefined,
      memo: values.memo || undefined,
    }

    createMutation.mutate(request, {
      onSuccess: () => {
        toast.success('Transaction created successfully')
        setOpen(false)
        form.reset()
      },
      onError: (error) => {
        // Error toast is already shown by apiClient
        console.error('Failed to create transaction:', error)
      },
    })
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button>Create Transaction</Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[500px] max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Create Transaction</DialogTitle>
          <DialogDescription>
            Record a new transaction. Entries will be automatically generated.
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
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
                        <SelectItem value="expense">Expense</SelectItem>
                        <SelectItem value="revenue">Revenue</SelectItem>
                        <SelectItem value="transfer">Transfer</SelectItem>
                        <SelectItem value="journal">Journal</SelectItem>
                      </SelectContent>
                    </Select>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="transaction_date"
                render={({ field }) => (
                  <FormItem className="flex flex-col mt-2.5">
                    <FormLabel>Date</FormLabel>
                    <Popover>
                      <PopoverTrigger asChild>
                        <FormControl>
                          <Button
                            variant="outline"
                            className={cn(
                              'w-full pl-3 text-left font-normal',
                              !field.value && 'text-muted-foreground'
                            )}
                          >
                            {field.value ? format(field.value, 'PPP') : <span>Pick a date</span>}
                            <CalendarIcon className="ml-auto h-4 w-4 opacity-50" />
                          </Button>
                        </FormControl>
                      </PopoverTrigger>
                      <PopoverContent className="w-auto p-0" align="start">
                        <Calendar
                          mode="single"
                          selected={field.value}
                          onSelect={field.onChange}
                          disabled={(date) => date > new Date() || date < new Date('1900-01-01')}
                          initialFocus
                        />
                      </PopoverContent>
                    </Popover>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <FormField
              control={form.control}
              name="reference_number"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Reference Number (Optional)</FormLabel>
                  <FormControl>
                    <Input placeholder="REF-001" {...field} />
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
                  <FormLabel>Description</FormLabel>
                  <FormControl>
                    <Input placeholder="e.g. Monthly rent payment" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            {/* Dimensions Section */}
            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="department"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Department (Optional)</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="Select Dept" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        <SelectItem value="none">None</SelectItem>
                        {dimensions
                          .find((d) => d.code === 'DEPT')
                          ?.values?.map((v) => (
                            <SelectItem key={v.id} value={v.id}>
                              {v.name}
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
                name="project"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Project (Optional)</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="Select Project" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        <SelectItem value="none">None</SelectItem>
                        {dimensions
                          .find((d) => d.code === 'PROJ')
                          ?.values?.map((v) => (
                            <SelectItem key={v.id} value={v.id}>
                              {v.name}
                            </SelectItem>
                          ))}
                      </SelectContent>
                    </Select>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="main_account_id"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Paid From / To</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="Select account" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {accounts
                          .filter((a) => a.type === 'asset' || a.type === 'liability')
                          .map((acc) => (
                            <SelectItem key={acc.id} value={acc.id}>
                              {acc.code} - {acc.name}
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
                name="contra_account_id"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Category (Account)</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="Select category" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {accounts
                          .filter((a) => a.type === 'expense' || a.type === 'revenue')
                          .map((acc) => (
                            <SelectItem key={acc.id} value={acc.id}>
                              {acc.code} - {acc.name}
                            </SelectItem>
                          ))}
                      </SelectContent>
                    </Select>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="currency"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Currency</FormLabel>
                    <Select onValueChange={field.onChange} defaultValue={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="Select" />
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

              <FormField
                control={form.control}
                name="amount"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Amount</FormLabel>
                    <FormControl>
                      <Input type="number" step="0.01" placeholder="0.00" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <FormField
              control={form.control}
              name="memo"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Memo (Optional)</FormLabel>
                  <FormControl>
                    <Input placeholder="Additional notes..." {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <DialogFooter>
              <Button type="submit" disabled={createMutation.isPending}>
                {createMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Create Transaction
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  )
}
