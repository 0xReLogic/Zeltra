'use client'

import React from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import * as z from 'zod'
import { format } from 'date-fns'
import { Calendar as CalendarIcon, Loader2, Coins } from 'lucide-react'
import { cn } from '@/lib/utils'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
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
import { Calendar } from '@/components/ui/calendar'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { toast } from 'sonner'

import { useAccounts } from '@/lib/queries/accounts'
import { usePayInvoice } from '@/lib/queries/transactions'
import { TransactionListItem } from '@/types/transactions'
import { Account } from '@/types/accounts'

// Schema - Zod v4 compatible
const formSchema = z.object({
  payment_account_id: z.string().min(1, 'Payment account is required'),
  amount: z.string().min(1, 'Amount is required'),
  exchange_rate: z.string().min(1, 'Exchange rate is required'),
  payment_date: z.date(),
  gain_loss_account_id: z.string().min(1, 'Gain/Loss account is required'),
  description: z.string().optional(),
})

interface PayInvoiceDialogProps {
  invoice: TransactionListItem | null
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function PayInvoiceDialog({ invoice, open, onOpenChange }: PayInvoiceDialogProps) {
  const { data: accountsData } = useAccounts()
  const payInvoice = usePayInvoice()
  // const fetchRates = useFetchLiveRates() // TODO: Implement when currency field is added to TransactionListItem

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      amount: '',
      exchange_rate: '1.0000',
      description: '',
      payment_date: new Date(),
    },
  })

  // Set default amount when invoice opens
  React.useEffect(() => {
    if (invoice && open) {
      // Assuming total_amount is available and positive string
      // Note: Backend TransactionListItem has total_amount string
      form.setValue('amount', invoice.total_amount || '0.00')
      form.setValue('description', `Payment for ${invoice.reference_number || 'invoice'}`)
      
      // Attempt to auto-set Gain/Loss account if one exists with "Exchange" or "Forex" in name
      const forexAccount = accountsData?.accounts?.find((a: { name: string; type: string }) => 
        (a.name.toLowerCase().includes('forex') || a.name.toLowerCase().includes('exchange')) && 
        (a.type.toLowerCase().includes('expense') || a.type.toLowerCase().includes('revenue'))
      )
      if (forexAccount) {
        form.setValue('gain_loss_account_id', forexAccount.id)
      }
    }
  }, [invoice, open, form, accountsData])

  const onSubmit = async (values: z.infer<typeof formSchema>) => {
    if (!invoice) return

    try {
      await payInvoice.mutateAsync({
        invoice_id: invoice.id,
        payment_account_id: values.payment_account_id,
        amount: values.amount, // Backend expects Decimal string or number? TS type says number usually for Decimal mapping but generated might be string. Checked generated: it's number or string. String is safer for precision.
        exchange_rate: values.exchange_rate,
        payment_date: format(values.payment_date, 'yyyy-MM-dd'),
        gain_loss_account_id: values.gain_loss_account_id,
        description: values.description,
        timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        // TODO: idempotency_key
      })
      
      toast.success('Invoice paid successfully')
      onOpenChange(false)
      form.reset()
    } catch (error) {
      toast.error('Failed to pay invoice')
      console.error(error)
    }
  }

  // Assets (Bank/Cash)
  const paymentAccounts = accountsData?.accounts?.filter((a: Account) => 
    ['Asset', 'Bank', 'Cash'].some(t => a.type.includes(t))
  ) || []

  // Expense/Equity/Revenue for Gain/Loss
  // Usually "Realized Gain/Loss" is an Expense or Revenue account
  const gainLossAccounts = accountsData?.accounts?.filter((a: Account) => 
    ['Expense', 'Revenue', 'Equity'].some(t => a.type.includes(t))
  ) || []

  const handleFetchRate = async () => {
    // Logic to fetch rate if we knew the currencies.
    // Invoice doesn't tell us its currency in TransactionListItem easily!
    // Wait, TransactionListItem doesn't have currency field?
    // Backend `list_transactions` response `TransactionListItem` definition:
    // pub struct TransactionListItem { id, reference_number, ... total_amount }
    // It does NOT have currency!
    // This is a missing field in `TransactionListItem` for UI to know user's currency context.
    // However, `total_amount` is usually in Functional Currency? No, usually Transaction Currency.
    // I need to update `TransactionListItem` to include `currency` to make this useful.
    
    // For now, I will let user manually input rate.
    toast.info("Please input the exchange rate manually for now.")
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Pay Invoice</DialogTitle>
          <DialogDescription>
            Record a payment for {invoice?.reference_number}. 
            Forex difference will be automatically calculated.
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="payment_account_id"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Payment Account</FormLabel>
                    <Select onValueChange={field.onChange} defaultValue={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="Select account" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {paymentAccounts.map((account) => (
                          <SelectItem key={account.id} value={account.id}>
                            {account.code} - {account.name}
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
                name="payment_date"
                render={({ field }) => (
                  <FormItem className="flex flex-col">
                    <FormLabel>Payment Date</FormLabel>
                    <Popover>
                      <PopoverTrigger asChild>
                        <FormControl>
                          <Button
                            variant={"outline"}
                            className={cn(
                              "w-full pl-3 text-left font-normal",
                              !field.value && "text-muted-foreground"
                            )}
                          >
                            {field.value ? (
                              format(field.value, "PPP")
                            ) : (
                              <span>Pick a date</span>
                            )}
                            <CalendarIcon className="ml-auto h-4 w-4 opacity-50" />
                          </Button>
                        </FormControl>
                      </PopoverTrigger>
                      <PopoverContent className="w-auto p-0" align="start">
                        <Calendar
                          mode="single"
                          selected={field.value}
                          onSelect={field.onChange}
                          disabled={(date) =>
                            date > new Date() || date < new Date("1900-01-01")
                          }
                          initialFocus
                        />
                      </PopoverContent>
                    </Popover>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
               <FormField
                control={form.control}
                name="amount"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Amount Paid</FormLabel>
                    <FormControl>
                      <Input placeholder="0.00" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <div className="flex gap-2 items-end">
                <FormField
                  control={form.control}
                  name="exchange_rate"
                  render={({ field }) => (
                    <FormItem className="flex-1">
                      <FormLabel>Exchange Rate</FormLabel>
                      <FormControl>
                        <Input placeholder="1.0000" {...field} />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <Button type="button" variant="outline" size="icon" onClick={handleFetchRate} title="Fetch Rate (Coming Soon)">
                    <Coins className="h-4 w-4" />
                </Button>
              </div>
            </div>

            <FormField
              control={form.control}
              name="gain_loss_account_id"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Realized Gain/Loss Account</FormLabel>
                   <Select onValueChange={field.onChange} defaultValue={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="Select account" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {gainLossAccounts.map((account) => (
                          <SelectItem key={account.id} value={account.id}>
                            {account.code} - {account.name}
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
                name="description"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Description</FormLabel>
                    <FormControl>
                      <Input {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                Cancel
              </Button>
              <Button type="submit" disabled={payInvoice.isPending}>
                {payInvoice.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Pay Invoice
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  )
}
