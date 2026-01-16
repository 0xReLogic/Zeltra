import { useState, useCallback } from 'react'
import { useForm } from 'react-hook-form'
import * as z from 'zod'
import { format, startOfDay, endOfDay } from 'date-fns'
import { CalendarIcon, Loader2, ChevronsUpDown, Leaf, Scale, AlertTriangle } from 'lucide-react'

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
import { useFiscalPeriods } from '@/lib/queries/fiscal'
import { toast } from 'sonner'
import { ApiError } from '@/lib/api/client'
import { useOrganization } from '@/lib/queries/organizations'
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
  // Dynamic dimensions: TypeID -> ValueID
  dimensionValues: z.record(z.string(), z.string()).optional(),
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
  const { data: valuesData } = useDimensionValues() // Fetch all values
  const { data: organization } = useOrganization()
  const { data: fiscalPeriodsData } = useFiscalPeriods()

  // Custom Zod Resolver to bypass version mismatch
  const resolver = useCallback(async (values: FormValues) => {
    try {
      const result = await formSchema.safeParseAsync(values)
     
      if (result.success) {
        return {
          values: result.data,
          errors: {},
        }
      }

      // ZodError structure mismatch workaround
      const errors = (result.error as unknown as { errors: Array<{ path: string[]; code: string; message: string }> }).errors.reduce(
        (all: Record<string, unknown>, current) => {
           const path = current.path.join('.')
           return {
             ...all,
             [path]: {
                type: current.code,
                message: current.message,
             }
           }
        },
        {}
      )

      return {
        values: {},
        errors,
      }
    } catch {
        return { values: {}, errors: {} }
    }
  }, [])

  const form = useForm<FormValues>({
    resolver, // Use custom resolver
    defaultValues: {
      type: 'expense',
      transaction_date: new Date(),
      reference_number: '',
      description: '',
      memo: '',
      amount: '',
      main_account_id: '',
      contra_account_id: '',
      // Valid record initialization
      dimensionValues: {}, 
      currency: 'USD',
      exchange_rate: '',
      esg_enabled: false,
      carbon_impact: '',
      vendor_score: '',
    },
  })

  // Ensure data is arrays - handle wrapper objects
  const accounts = accountsData?.accounts ?? []
  const dimensions = dimensionsData?.dimension_types || []
  const allValues = valuesData?.dimension_values || []
  const fiscalPeriods = fiscalPeriodsData || []
  
  // Tier-aware exchange rate check
  const baseCurrency = organization?.base_currency || 'USD'
  const isStarterTier = organization?.subscription_tier?.toLowerCase() === 'starter'

  // Fiscal period validation
  const transactionDate = form.watch('transaction_date')
  const activePeriod = fiscalPeriods.find(p => {
    if (!transactionDate || !p.start_date || !p.end_date) return false
    const start = new Date(p.start_date)
    const end = new Date(p.end_date)
    const tx = startOfDay(transactionDate)
    return (tx >= startOfDay(start) && tx <= endOfDay(end))
  })
  
  const isPeriodOpen = activePeriod?.status === 'OPEN'
  const noPeriodFound = transactionDate && !activePeriod
  const isPeriodClosed = activePeriod && activePeriod.status !== 'OPEN'

  function onSubmit(values: FormValues) {
    const amount = values.amount
    const currency = values.currency

    // Prepare Metadata
    let metadata: Record<string, unknown> | undefined = undefined
    if (values.esg_enabled) {
        metadata = {
            esg: {
                carbon_impact: parseFloat(values.carbon_impact || '0'),
                vendor_score: parseFloat(values.vendor_score || '0'),
                verified: true
            }
        }
    }

    // Collect dynamic dimension IDs (flatten from map)
    const dims: string[] = values.dimensionValues 
        ? (Object.values(values.dimensionValues) as string[]).filter(v => v !== 'none') 
        : []

    // Build entries based on transaction type
    let entries: CreateEntryRequest[] = []

    const commonProps = {
        source_amount: amount,
        source_currency: currency,
        exchange_rate: values.exchange_rate || undefined,
    }

    if (values.type === 'expense') {
      entries = [
        {
          account_id: values.contra_account_id,
          entry_type: 'debit',
          dimensions: dims.length > 0 ? dims : undefined,
          metadata: metadata,
          ...commonProps
        },
        {
          account_id: values.main_account_id,
          entry_type: 'credit',
          ...commonProps
        },
      ]
    } else if (values.type === 'revenue') {
      entries = [
        {
          account_id: values.main_account_id,
          entry_type: 'debit',
          ...commonProps
        },
        {
          account_id: values.contra_account_id,
          entry_type: 'credit',
          dimensions: dims.length > 0 ? dims : undefined,
          metadata: metadata,
          ...commonProps
        },
      ]
    } else {
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
          metadata: metadata,
          ...commonProps
        },
      ]
    }

    const request = {
      type: values.type,
      transaction_date: values.transaction_date instanceof Date && !isNaN(values.transaction_date.getTime())
        ? format(values.transaction_date, 'yyyy-MM-dd')
        : format(new Date(), 'yyyy-MM-dd'),
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
        if (error instanceof ApiError && error.status === 400) {
          const code = error.code
          if (code === 'no_fiscal_period') {
            toast.error('No open fiscal period found for this date. Transaction cannot be posted.')
            return
          }
          if (code === 'period_closed') {
            toast.error('The fiscal period for this date is closed. No further posting allowed.')
            return
          }
          if (error.details?.missing_dimensions) {
            const missing = error.details.missing_dimensions.join(', ')
            toast.error(`Dimension '${missing}' is required because this account is tied to a budget.`)
            return
          }
        }
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

            {/* Fiscal Period Validation Messages */}
            {noPeriodFound && (
              <div className="flex items-start gap-2 rounded-md border border-amber-500/50 bg-amber-500/10 p-3 text-sm">
                <AlertTriangle className="h-4 w-4 text-amber-500 mt-0.5 flex-shrink-0" />
                <div>
                  <p className="font-medium text-amber-600">No Fiscal Period Found</p>
                  <p className="text-muted-foreground text-xs mt-1">
                    There is no fiscal period defined for this date. Please create one in Settings.
                  </p>
                </div>
              </div>
            )}

            {isPeriodClosed && (
              <div className="flex items-start gap-2 rounded-md border border-red-500/50 bg-red-500/10 p-3 text-sm">
                <AlertTriangle className="h-4 w-4 text-red-500 mt-0.5 flex-shrink-0" />
                <div>
                  <p className="font-medium text-red-600">Fiscal Period Restricted</p>
                  <p className="text-muted-foreground text-xs mt-1">
                    The fiscal period for this date is {activePeriod?.status?.toLowerCase()}. Posting is not allowed.
                  </p>
                </div>
              </div>
            )}

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

            {/* Dynamic Dimensions Section */}
            {dimensions.length > 0 && (
              <div className="grid grid-cols-2 gap-4">
                {dimensions.map((dim) => {
                  const options = allValues.filter(v => {
                    // eslint-disable-next-line @typescript-eslint/no-explicit-any
                    const val = v as any
                    const matchType = val.dimension_type_id === dim.id || val.dimension_type?.id === dim.id
                    const isActive = val.is_active !== false && val.active !== false
                    return matchType && isActive
                  })
                  
                  return (
                    <FormField
                      key={dim.id}
                      control={form.control}
                      name={`dimensionValues.${dim.id}`}
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel>{dim.name} {dim.is_required ? '*' : '(Optional)'}</FormLabel>
                          <Select 
                            onValueChange={field.onChange} 
                            value={(field.value || 'none') as string}
                          >
                            <FormControl>
                              <SelectTrigger>
                                <SelectValue placeholder={`Select ${dim.name}`} />
                              </SelectTrigger>
                            </FormControl>
                            <SelectContent>
                              <SelectItem value="none">None</SelectItem>
                              {options.map((v) => (
                                <SelectItem key={v.code} value={v.id}>
                                  {v.name}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                  )
                })}
              </div>
            )}

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
                          <FormLabel>
                            Exchange Rate {isStarterTier && form.watch('currency') !== baseCurrency ? '(Required)' : 'Override'}
                          </FormLabel>
                          <FormControl>
                            <Input placeholder="1.0000" {...field} />
                          </FormControl>
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                   </div>
                   
                   {/* STARTER tier warning for non-base currencies */}
                   {isStarterTier && form.watch('currency') !== baseCurrency && (
                     <div className="flex items-start gap-2 rounded-md border border-amber-500/50 bg-amber-500/10 p-3 text-sm">
                       <AlertTriangle className="h-4 w-4 text-amber-500 mt-0.5 flex-shrink-0" />
                       <div>
                         <p className="font-medium text-amber-600">Manual Exchange Rate Required</p>
                         <p className="text-muted-foreground text-xs mt-1">
                           Your Starter plan requires manual exchange rate input for {form.watch('currency')} → {baseCurrency}. 
                           Upgrade to Growth for auto-sync rates.
                         </p>
                       </div>
                     </div>
                   )}

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
              <Button type="submit" disabled={createMutation.isPending || !isPeriodOpen}>
                {createMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                {!isPeriodOpen && transactionDate ? 'Period Restricted' : 'Create Transaction'}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  )
}
