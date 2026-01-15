'use client'

import { useState } from 'react'
import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import * as z from 'zod'
import { format } from 'date-fns'
import { CalendarIcon, Loader2, ChevronsUpDown, Leaf, Scale } from 'lucide-react'

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
import { useDimensions, useDimensionValues } from '@/lib/queries/dimensions'
import { toast } from 'sonner'
import { ApiError } from '@/lib/api/client'
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
  // Advanced Fields
  exchange_rate: z.string().optional(),
  esg_enabled: z.boolean().default(false),
  carbon_impact: z.string().optional(),
  vendor_score: z.string().optional(),
})

type FormValues = z.infer<typeof formSchema>

export function CreateTransactionDialog() {
  const [open, setOpen] = useState(false)
  const [isAdvancedOpen, setIsAdvancedOpen] = useState(false)
  const createMutation = useCreateTransaction()
  const { data: accountsData } = useAccounts()
  const { data: dimensionsData } = useDimensions()

  // Ensure data is arrays - handle wrapper objects
  const accounts = accountsData?.accounts ?? []
  const dimensions = Array.isArray(dimensionsData) ? dimensionsData : []
  
  // Get dimension type IDs for DEPT and PROJ
  const deptTypeId = dimensions.find(d => d.code === 'DEPT')?.id
  const projTypeId = dimensions.find(d => d.code === 'PROJ')?.id
  
  // Fetch dimension values for each type
  const { data: deptValues } = useDimensionValues(deptTypeId)
  const { data: projValues } = useDimensionValues(projTypeId)
  
  // Extract values arrays
  const departmentOptions = Array.isArray(deptValues) ? deptValues : []
  const projectOptions = Array.isArray(projValues) ? projValues : []

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
      exchange_rate: '',
      esg_enabled: false,
      carbon_impact: '',
      vendor_score: '',
    },
  })

  function onSubmit(values: FormValues) {
    const amount = values.amount
    const currency = values.currency

    // Prepare Metadata
    let metadata: Record<string, any> | undefined = undefined
    if (values.esg_enabled) {
        metadata = {
            esg: {
                carbon_impact: parseFloat(values.carbon_impact || '0'),
                vendor_score: parseFloat(values.vendor_score || '0'),
                verified: true
            }
        }
    }

    // Collect dimension IDs
    const dims: string[] = []
    if (values.department && values.department !== 'none') dims.push(values.department)
    if (values.project && values.project !== 'none') dims.push(values.project)

    // Build entries based on transaction type
    // API expects: account_id, entry_type ("debit" | "credit"), source_amount, source_currency
    let entries: CreateEntryRequest[] = []

    const commonProps = {
        source_amount: amount,
        source_currency: currency,
        exchange_rate: values.exchange_rate, // Pass override if present
    }

    if (values.type === 'expense') {
      // Expense: Debit Expense account, Credit Asset account
      entries = [
        {
          account_id: values.contra_account_id, // Expense account
          entry_type: 'debit',
          dimensions: dims.length > 0 ? dims : undefined,
          metadata: metadata, // Attach ESG to Expense
          ...commonProps
        },
        {
          account_id: values.main_account_id, // Asset/Bank account
          entry_type: 'credit',
          ...commonProps
        },
      ]
    } else if (values.type === 'revenue') {
      // Revenue: Debit Asset account, Credit Revenue account
      entries = [
        {
          account_id: values.main_account_id, // Asset/Bank account
          entry_type: 'debit',
          ...commonProps
        },
        {
          account_id: values.contra_account_id, // Revenue account
          entry_type: 'credit',
          dimensions: dims.length > 0 ? dims : undefined,
          metadata: metadata, // Attach ESG to Revenue (Impact of sales?)
          ...commonProps
        },
      ]
    } else {
      // Transfer/Journal: Simple debit/credit
      entries = [
        {
          account_id: values.main_account_id,
          entry_type: 'debit',
          dimensions: dims.length > 0 ? dims : undefined,
          metadata: metadata,
          ...commonProps
        },
        {
          account_id: values.contra_account_id,
          entry_type: 'credit',
          metadata: metadata, // Attach to both or just one?
          ...commonProps
        },
      ]
    }

    const request = {
      type: values.type,
      transaction_date: format(values.transaction_date, 'yyyy-MM-dd'),
      description: values.description,
      entries,
      reference_number: values.reference_number || undefined,
      memo: values.memo || undefined,
      timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    } as CreateTransactionRequest

    createMutation.mutate(request, {
      onSuccess: () => {
        toast.success('Transaction created successfully')
        setOpen(false)
        form.reset()
      },
      onError: (error) => {
        // Handle specific budget dimension validation errors
        if (error instanceof ApiError && error.status === 400 && error.details?.missing_dimensions) {
          const missing = error.details.missing_dimensions.join(', ')
          toast.error(`Dimension '${missing}' is required because this account is tied to a budget.`)
          return
        }
        
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
                        {departmentOptions.map((v) => (
                          <SelectItem key={v.code} value={v.code}>
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
                        {projectOptions.map((v) => (
                          <SelectItem key={v.code} value={v.code}>
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

            <div className="w-full space-y-2 border rounded-md p-2">
              <div className="flex items-center justify-between px-2">
                <h4 className="text-sm font-semibold">Advanced Options</h4>
                <Button
                  variant="ghost" 
                  size="sm" 
                  className="w-9 p-0"
                  type="button"
                  onClick={() => setIsAdvancedOpen(!isAdvancedOpen)}
                >
                  <ChevronsUpDown className="h-4 w-4" />
                  <span className="sr-only">Toggle</span>
                </Button>
              </div>
              
              {isAdvancedOpen && (
                <div className="space-y-4 px-2 pt-2">
                   <div className="grid grid-cols-2 gap-4">
                     <FormField
                      control={form.control}
                      name="exchange_rate"
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel>Exchange Rate Override</FormLabel>
                          <FormControl>
                            <Input placeholder="1.0000" {...field} />
                          </FormControl>
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                   </div>

                   <div className="space-y-4 border-t pt-4">
                      <FormField
                        control={form.control}
                        name="esg_enabled"
                        render={({ field }) => (
                          <FormItem className="flex flex-row items-start space-x-3 space-y-0 rounded-md border p-4">
                            <FormControl>
                              <input
                                type="checkbox"
                                className="h-4 w-4"
                                checked={field.value}
                                onChange={field.onChange}
                              />
                            </FormControl>
                            <div className="space-y-1 leading-none">
                              <FormLabel>
                                Include Sentinel Intelligence (ESG Data)
                              </FormLabel>
                              <p className="text-sm text-muted-foreground">
                                Attach Carbon Impact and Vendor Score metadata.
                              </p>
                            </div>
                          </FormItem>
                        )}
                      />

                      {form.watch('esg_enabled') && (
                        <div className="grid grid-cols-2 gap-4">
                          <FormField
                            control={form.control}
                            name="carbon_impact"
                            render={({ field }) => (
                              <FormItem>
                                <FormLabel>Carbon Impact (kgCO2e)</FormLabel>
                                <FormControl>
                                  <div className="relative">
                                    <Leaf className="absolute left-2 top-2.5 h-4 w-4 text-green-600" />
                                    <Input className="pl-8" placeholder="0.00" {...field} />
                                  </div>
                                </FormControl>
                                <FormMessage />
                              </FormItem>
                            )}
                          />
                          <FormField
                            control={form.control}
                            name="vendor_score"
                            render={({ field }) => (
                              <FormItem>
                                <FormLabel>Vendor ESG Score (0-100)</FormLabel>
                                <FormControl>
                                  <div className="relative">
                                    <Scale className="absolute left-2 top-2.5 h-4 w-4 text-blue-600" />
                                    <Input className="pl-8" placeholder="85" {...field} />
                                  </div>
                                </FormControl>
                                <FormMessage />
                              </FormItem>
                            )}
                          />
                        </div>
                      )}
                   </div>
                </div>
              )}
            </div>

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
